const API_BASE_URL = 'http://localhost:3000';

const STRESS_NAMES = [
    "Alice", "Bob", "Charlie", "David", "Eve", "Frank", "Grace", "Heidi", "Ivan", "Judy",
    "Karl", "Linda", "Mike", "Nancy", "Oscar", "Peggy", "Quinn", "Rose", "Sam", "Ted",
    "Ursula", "Victor", "Wendy", "Xander", "Yvonne", "Zelda", "Arthur", "Beatrice", "Conrad", "Diana",
    "Eric", "Flora", "George", "Hilda", "Isaac", "Julia", "Kevin", "Lois", "Mark", "Nora",
    "Oliver", "Paula", "Quentin", "Ruth", "Seth", "Tara", "Ulysses", "Vera", "Walter", "Xena"
];

// Global State
let charts = {};
let tpsHistory = Array(30).fill(0);
let requestCount = 0;
let lastThroughputTime = Date.now();
let stressTestRunning = false;
let stressTestInterval = null;
let adminToken = null;
let currentView = 'regular';
let historyPage = 1;
const HISTORY_COUNT = 15;
let dateRangeMin = null; // Date object: earliest known timestamp
let dateRangeMax = null; // Date object: latest known timestamp

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    initCharts();
    setupEvents();
    loadBoardConfig();

    // High-frequency UI loop
    setInterval(updateLoop, 250);
    // History auto-refresh
    setInterval(fetchHistory, 3000);
});

function initCharts() {
    const commonOptions = {
        responsive: true,
        maintainAspectRatio: false,
        plugins: { legend: { display: false } },
        scales: {
            x: { display: false },
            y: { beginAtZero: true, grid: { color: '#f5f5f5', borderDash: [2, 2] }, ticks: { font: { size: 9 } } }
        },
        animation: { duration: 200 }
    };

    // Request Rate Timeline
    charts.timeline = new Chart(document.getElementById('timelineChart'), {
        type: 'line',
        data: {
            labels: Array(30).fill(''),
            datasets: [{
                borderColor: '#28a745', borderWidth: 1.5, pointRadius: 0, fill: true,
                backgroundColor: 'rgba(40, 167, 69, 0.05)', data: tpsHistory, tension: 0.3
            }]
        },
        options: commonOptions
    });

    // Distribution Histogram
    charts.distribution = new Chart(document.getElementById('distributionChart'), {
        type: 'bar',
        data: {
            labels: ['0', '200', '400', '600', '800', '1k+'],
            datasets: [{ backgroundColor: '#6f42c1', borderRadius: 2, data: [0, 0, 0, 0, 0, 0] }]
        },
        options: { 
            ...commonOptions, 
            scales: { 
                ...commonOptions.scales, 
                x: { display: true, ticks: { font: { size: 9 } } } 
            } 
        }
    });
}

function setupEvents() {
    document.getElementById('addEntryForm').addEventListener('submit', handleAdd);
    document.getElementById('startStress').addEventListener('click', startStressTest);
    document.getElementById('stopStress').addEventListener('click', stopStressTest);
    document.getElementById('toggleViewBtn').addEventListener('click', toggleView);
    document.getElementById('adminAuthForm').addEventListener('submit', handleAdminAuth);
    document.getElementById('updateTitleForm').addEventListener('submit', handleUpdateTitle);
    document.getElementById('updateSortForm').addEventListener('submit', handleUpdateSort);
    document.getElementById('logoutAdmin').addEventListener('click', handleAdminLogout);
    document.getElementById('historyPrev').addEventListener('click', () => { historyPage = Math.max(1, historyPage - 1); fetchHistory(); });
    document.getElementById('historyNext').addEventListener('click', () => { historyPage++; fetchHistory(); });
    document.getElementById('leaderboardLimit').addEventListener('change', fetchLeaderboard);

    // Stress slider numeric readout
    const stressSlider = document.getElementById('stressIntensity');
    stressSlider.addEventListener('input', () => {
        document.getElementById('stressValue').textContent = stressSlider.value;
    });

    // Date range dual slider
    const rangeMin = document.getElementById('rangeMin');
    const rangeMax = document.getElementById('rangeMax');
    let rangeTimeout;
    const onRangeChange = () => {
        enforceRangeConstraint();
        updateRangeTrack();
        updateRangeLabels();
        clearTimeout(rangeTimeout);
        rangeTimeout = setTimeout(() => { historyPage = 1; fetchHistory(); }, 300);
    };
    rangeMin.addEventListener('input', onRangeChange);
    rangeMax.addEventListener('input', onRangeChange);
    document.getElementById('resetDateRange').addEventListener('click', () => {
        rangeMin.value = 0;
        rangeMax.value = 1000;
        updateRangeTrack();
        updateRangeLabels();
        historyPage = 1;
        fetchHistory();
    });

    let searchTimeout;
    document.getElementById('historyUser').addEventListener('input', () => {
        clearTimeout(searchTimeout);
        searchTimeout = setTimeout(() => { historyPage = 1; fetchHistory(); }, 500);
    });
}

async function updateLoop() {
    await Promise.all([
        fetchLeaderboard(),
        fetchStats(),
        fetchPerformance()
    ]);
    updateThroughput();
}

async function fetchLeaderboard() {
    const limit = document.getElementById('leaderboardLimit').value;
    try {
        const res = await fetch(`${API_BASE_URL}/leaderboard/json/${limit}`);
        if (!res.ok) return;
        const data = await res.json();
        
        const el = document.getElementById('leaderboard');
        if (data.length === 0) {
            el.innerHTML = '<div style="text-align:center;padding:40px;color:#999;">SYSTEM_IDLE: No data points found</div>';
            return;
        }

        el.innerHTML = data.map((item, i) => `
            <div class="leaderboard-item">
                <span class="rank">${(i + 1).toString().padStart(2, '0')}</span>
                <span class="username">${esc(item.uploader)}</span>
                <span class="timestamp">${formatTime(item.created_at)}</span>
                <span class="score">${fmt(item.value)}</span>
            </div>
        `).join('');

        updateDistribution(data);
    } catch (e) { console.error('Leaderboard error:', e); }
}

function updateDistribution(data) {
    const buckets = [0, 0, 0, 0, 0, 0];
    data.forEach(item => {
        const val = item.value;
        if (val < 200) buckets[0]++;
        else if (val < 400) buckets[1]++;
        else if (val < 600) buckets[2]++;
        else if (val < 800) buckets[3]++;
        else if (val < 1000) buckets[4]++;
        else buckets[5]++;
    });
    charts.distribution.data.datasets[0].data = buckets;
    charts.distribution.update('none');
}

async function fetchStats() {
    try {
        const res = await fetch(`${API_BASE_URL}/info`);
        if (!res.ok) return;
        const d = await res.json();
        
        // Populate all statistical fields from /info (raw server data)
        document.getElementById('totalEntries').textContent = d.count ?? 0;
        document.getElementById('meanScore').textContent = fmt(d.mean);
        document.getElementById('scoreRange').textContent = `${fmt(d.min)} / ${fmt(d.max)}`;
        document.getElementById('statMedian').textContent = fmt(d.median);
        document.getElementById('statStdDev').textContent = fmt(d.stddev_pop);
        document.getElementById('statMinMax').textContent = `${fmt(d.min)} / ${fmt(d.max)}`;
        document.getElementById('statMode').textContent = fmt(d.mode);
        document.getElementById('statP25').textContent = fmt(d.p25);
        document.getElementById('statP75').textContent = fmt(d.p75);
        document.getElementById('statIQR').textContent = fmt(d.iqr);
        document.getElementById('statVariance').textContent = fmt(d.variance);
        document.getElementById('statRange').textContent = fmt(d.range);

        // Update date range slider bounds from server
        const earliest = parseCreatedAt(d.earliest_at);
        const latest = parseCreatedAt(d.latest_at);
        if (earliest && latest) {
            dateRangeMin = earliest;
            dateRangeMax = latest;
            updateRangeLabels();
        }
    } catch (e) { console.error('Stats error:', e); }
}

async function fetchPerformance() {
    try {
        const res = await fetch(`${API_BASE_URL}/performance`);
        if (!res.ok) return;
        const data = await res.json();

        const el = document.getElementById('perfTable');
        const header = '<div class="perf-header"><span>Endpoint</span><span>Avg Latency</span><span>Requests</span></div>';
        if (data.length === 0) {
            el.innerHTML = header + '<div style="text-align:center;padding:12px;color:#999;font-size:11px;">No endpoint data</div>';
            return;
        }
        el.innerHTML = header + data.map(e => `
            <div class="perf-row">
                <span class="endpoint">${esc(e.endpoint)}</span>
                <span class="latency">${e.avg_ms.toFixed(3)}ms</span>
                <span class="req-count">${e.count.toLocaleString()}</span>
            </div>
        `).join('');
    } catch (e) { console.error('Performance error:', e); }
}

function updateThroughput() {
    const now = Date.now();
    const elapsed = (now - lastThroughputTime) / 1000;
    if (elapsed >= 0.8) {
        const rps = Math.round(requestCount / elapsed);
        tpsHistory.push(rps);
        if (tpsHistory.length > 30) tpsHistory.shift();
        charts.timeline.data.datasets[0].data = tpsHistory;
        charts.timeline.update('none');
        requestCount = 0;
        lastThroughputTime = now;
    }
}

async function fetchHistory() {
    const user = document.getElementById('historyUser').value.trim();
    const params = new URLSearchParams({ count: HISTORY_COUNT, page: historyPage });
    if (user) params.set('title', user);

    // Apply date range filter from dual slider
    const startDate = getDateFromSlider('rangeMin');
    const endDate = getDateFromSlider('rangeMax');
    if (startDate) params.set('start', startDate.toISOString());
    if (endDate) params.set('end', endDate.toISOString());

    try {
        const res = await fetch(`${API_BASE_URL}/history?${params}`);
        if (!res.ok) return;
        const data = await res.json();
        const el = document.getElementById('historyResults');
        if (data.length === 0) {
            el.innerHTML = '<div style="text-align:center;padding:20px;color:#999;font-size:11px;">EMPTY_LOG: No matching events</div>';
        } else {
            el.innerHTML = data.map(e => `
                <div class="history-row">
                    <span>${esc(e.uploader)} <span style="color:#999">&rarr;</span> <strong>${fmt(e.value)}</strong></span>
                    <span class="time">${formatTime(e.created_at)}</span>
                </div>
            `).join('');
        }
        document.getElementById('historyPageLabel').textContent = `PAGE ${historyPage}`;
        document.getElementById('historyPrev').disabled = historyPage <= 1;
        document.getElementById('historyNext').disabled = data.length < HISTORY_COUNT;
    } catch (e) { console.error('History error:', e); }
}

// Date range slider helpers
function parseCreatedAt(arr) {
    if (!arr) return null;
    return new Date(Date.UTC(arr[0], 0, arr[1], arr[2], arr[3], arr[4]));
}

function getDateFromSlider(sliderId) {
    if (!dateRangeMin || !dateRangeMax) return null;
    const slider = document.getElementById(sliderId);
    const pct = parseInt(slider.value) / 1000;
    const range = dateRangeMax.getTime() - dateRangeMin.getTime();
    return new Date(dateRangeMin.getTime() + range * pct);
}

function enforceRangeConstraint() {
    const minSlider = document.getElementById('rangeMin');
    const maxSlider = document.getElementById('rangeMax');
    if (parseInt(minSlider.value) > parseInt(maxSlider.value)) {
        minSlider.value = maxSlider.value;
    }
}

function updateRangeTrack() {
    const min = parseInt(document.getElementById('rangeMin').value);
    const max = parseInt(document.getElementById('rangeMax').value);
    const track = document.getElementById('rangeTrack');
    const left = (min / 1000) * 100;
    const width = ((max - min) / 1000) * 100;
    track.style.setProperty('--range-left', left + '%');
    track.style.setProperty('--range-width', width + '%');
}

function updateRangeLabels() {
    const startDate = getDateFromSlider('rangeMin');
    const endDate = getDateFromSlider('rangeMax');
    document.getElementById('rangeStartLabel').textContent = startDate ? startDate.toLocaleString() : '--';
    document.getElementById('rangeEndLabel').textContent = endDate ? endDate.toLocaleString() : '--';
}

async function handleAdd(e) {
    e.preventDefault();
    const key = document.getElementById('username').value.trim();
    const value = parseFloat(document.getElementById('score').value);
    try {
        const res = await fetch(`${API_BASE_URL}/add`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ key, value })
        });
        if (res.ok) {
            requestCount++;
            showStatus('INGRESS_SUCCESS');
            document.getElementById('addEntryForm').reset();
        }
    } catch (e) { showStatus('INGRESS_FAILED', true); }
}

function startStressTest() {
    if (stressTestRunning) return;
    stressTestRunning = true;
    document.getElementById('startStress').disabled = true;
    document.getElementById('stopStress').disabled = false;
    const intensity = parseInt(document.getElementById('stressIntensity').value);
    stressTestInterval = setInterval(async () => {
        const batchSize = Math.max(1, Math.floor(intensity / 2));
        const batch = [];
        for(let i=0; i < batchSize; i++) {
            const baseName = STRESS_NAMES[Math.floor(Math.random() * STRESS_NAMES.length)];
            const uploader = `${baseName}_${Math.floor(Math.random() * 11)}`;
            const val = parseFloat((Math.random() * 1000).toFixed(2));
            batch.push(fetch(`${API_BASE_URL}/add`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ key: uploader, value: val })
            }).then(() => requestCount++).catch(() => {}));
        }
        await Promise.all(batch);
        document.getElementById('stressStatus').textContent = `LOAD_STATUS: ~${intensity * 10} REQ/SEC`;
    }, 100);
}

function stopStressTest() {
    stressTestRunning = false;
    clearInterval(stressTestInterval);
    document.getElementById('startStress').disabled = false;
    document.getElementById('stopStress').disabled = true;
    document.getElementById('stressStatus').textContent = 'ENGINE_IDLE';
}

function fmt(v) { return v != null ? v.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }) : '0.00'; }
function esc(t) { const d = document.createElement('div'); d.textContent = t; return d.innerHTML; }
function formatTime(arr) { 
    if (!arr) return "N/A";
    return new Date(Date.UTC(arr[0], 0, arr[1], arr[2], arr[3], arr[4])).toLocaleTimeString(); 
}

function showStatus(m, err=false) {
    const el = document.getElementById('statusMessage');
    el.textContent = m;
    el.className = 'show';
    el.style.background = err ? 'var(--danger)' : 'var(--text)';
    setTimeout(() => el.className = '', 1500);
}

async function loadBoardConfig() {
    try {
        const res = await fetch(`${API_BASE_URL}/boardconfig`);
        if (res.ok) {
            const config = await res.json();
            document.querySelector('header h1').textContent = config.title;
            document.getElementById('activeSortOrder').textContent = (config.sort_order || '--').toUpperCase();
        }
    } catch (e) {}
}

function toggleView() {
    const regularView = document.getElementById('regularView');
    const adminView = document.getElementById('adminView');
    const toggleBtn = document.getElementById('toggleViewBtn');
    if (currentView === 'regular') {
        regularView.style.display = 'none';
        adminView.style.display = 'block';
        toggleBtn.textContent = 'EXIT_ADMIN';
        currentView = 'admin';
    } else {
        regularView.style.display = 'block';
        adminView.style.display = 'none';
        toggleBtn.textContent = 'ADMIN_LOGIN';
        currentView = 'regular';
    }
}

async function handleAdminAuth(e) {
    e.preventDefault();
    const token = document.getElementById('adminToken').value.trim();
    try {
        const res = await fetch(`${API_BASE_URL}/admin/config`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
            body: JSON.stringify({})
        });
        if (res.ok) {
            adminToken = token;
            document.getElementById('adminAuthSection').style.display = 'none';
            document.getElementById('adminPanel').style.display = 'block';
            showStatus('AUTH_VERIFIED');
        } else { showStatus('AUTH_DENIED', true); }
    } catch (e) { showStatus('AUTH_ERROR', true); }
}

async function handleUpdateTitle(e) {
    e.preventDefault();
    const title = document.getElementById('newTitle').value.trim();
    try {
        const res = await fetch(`${API_BASE_URL}/admin/config`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${adminToken}` },
            body: JSON.stringify({ title })
        });
        if (res.ok) { showStatus('CONFIG_UPDATED'); loadBoardConfig(); }
    } catch (e) {}
}

async function handleUpdateSort(e) {
    e.preventDefault();
    const sort_order = document.getElementById('sortOrder').value;
    try {
        const res = await fetch(`${API_BASE_URL}/admin/config`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${adminToken}` },
            body: JSON.stringify({ sort_order })
        });
        if (res.ok) { showStatus('LOGIC_UPDATED'); loadBoardConfig(); }
    } catch (e) {}
}

function handleAdminLogout() {
    adminToken = null;
    document.getElementById('adminPanel').style.display = 'none';
    document.getElementById('adminAuthSection').style.display = 'block';
    showStatus('LOGOUT_SUCCESS');
}
