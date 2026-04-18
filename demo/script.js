const API_BASE_URL = './';

const SIMULATED_USERS = [
    "Alice", "Bob", "Charlie", "David", "Eve", "Frank", "Grace", "Heidi", "Ivan", "Judy",
    "Karl", "Linda", "Mike", "Nancy", "Oscar", "Peggy", "Quinn", "Rose", "Sam", "Ted",
    "Ursula", "Victor", "Wendy", "Xander", "Yvonne", "Zelda", "Arthur", "Beatrice", "Conrad", "Diana"
];

// Global State
let charts = {};
let throughputHistory = Array(30).fill(0);
let requestCounter = 0;
let lastUpdate = Date.now();
let testRunning = false;
let testInterval = null;
let adminToken = null;
let viewMode = 'regular';
let historyPage = 1;
const PAGE_SIZE = 15;
let minTimestamp = null;
let maxTimestamp = null;

document.addEventListener('DOMContentLoaded', async () => {
    initializeCharts();
    bindEvents();
    fetchConfig();

    // Fetch initial data before starting intervals to avoid race condition
    await refreshData();
    await fetchHistory();

    setInterval(refreshData, 1000);
    setInterval(fetchHistory, 1000);
});

function initializeCharts() {
    const baseOptions = {
        responsive: true,
        maintainAspectRatio: false,
        plugins: { legend: { display: false } },
        scales: {
            x: { display: false },
            y: { beginAtZero: true, grid: { color: '#f0f0f0' }, ticks: { font: { size: 10 } } }
        },
        animation: { duration: 300 }
    };

    charts.timeline = new Chart(document.getElementById('timelineChart'), {
        type: 'line',
        data: {
            labels: Array(30).fill(''),
            datasets: [{
                borderColor: '#28a745', borderWidth: 2, pointRadius: 0, fill: true,
                backgroundColor: 'rgba(40, 167, 69, 0.05)', data: throughputHistory, tension: 0.4
            }]
        },
        options: baseOptions
    });

    charts.distribution = new Chart(document.getElementById('distributionChart'), {
        type: 'bar',
        data: {
            labels: ['<200', '400', '600', '800', '1k', '1k+'],
            datasets: [{ backgroundColor: '#0366d6', borderRadius: 4, data: [0, 0, 0, 0, 0, 0] }]
        },
        options: {
            ...baseOptions,
            scales: {
                ...baseOptions.scales,
                x: { display: true, ticks: { font: { size: 10 } } }
            }
        }
    });
}

function bindEvents() {
    document.getElementById('addEntryForm').addEventListener('submit', onAddSubmit);
    document.getElementById('startStress').addEventListener('click', startLoadTest);
    document.getElementById('stopStress').addEventListener('click', stopLoadTest);
    document.getElementById('toggleViewBtn').addEventListener('click', switchView);
    document.getElementById('adminAuthForm').addEventListener('submit', onAdminAuth);
    document.getElementById('updateTitleForm').addEventListener('submit', onUpdateTitle);
    document.getElementById('updateSortForm').addEventListener('submit', onUpdateSort);
    document.getElementById('logoutAdmin').addEventListener('click', onLogout);
    document.getElementById('historyPrev').addEventListener('click', () => { historyPage = Math.max(1, historyPage - 1); fetchHistory(); });
    document.getElementById('historyNext').addEventListener('click', () => { historyPage++; fetchHistory(); });
    document.getElementById('leaderboardLimit').addEventListener('change', fetchLeaderboard);

    const stressSlider = document.getElementById('stressIntensity');
    stressSlider.addEventListener('input', () => {
        document.getElementById('stressValue').textContent = stressSlider.value;
    });

    const rangeMin = document.getElementById('rangeMin');
    const rangeMax = document.getElementById('rangeMax');
    let rangeDebounce;
    const onRangeInput = () => {
        if (parseInt(rangeMin.value) > parseInt(rangeMax.value)) rangeMin.value = rangeMax.value;
        refreshRangeUI();
        clearTimeout(rangeDebounce);
        rangeDebounce = setTimeout(() => { historyPage = 1; fetchHistory(); }, 400);
    };
    rangeMin.addEventListener('input', onRangeInput);
    rangeMax.addEventListener('input', onRangeInput);
    document.getElementById('resetDateRange').addEventListener('click', () => {
        rangeMin.value = 0; rangeMax.value = 1000;
        refreshRangeUI();
        historyPage = 1; fetchHistory();
    });

    let searchDebounce;
    document.getElementById('historyUser').addEventListener('input', () => {
        clearTimeout(searchDebounce);
        searchDebounce = setTimeout(() => { historyPage = 1; fetchHistory(); }, 400);
    });
}

async function refreshData() {
    await Promise.all([fetchLeaderboard(), fetchStats(), fetchPerformance()]);
    updateThroughputChart();
}

async function fetchLeaderboard() {
    const limit = document.getElementById('leaderboardLimit').value;
    try {
        const res = await fetch(`${API_BASE_URL}/leaderboard/json/${limit}`);
        const data = await res.json();
        const container = document.getElementById('leaderboard');

        if (!data.length) {
            container.innerHTML = '<div style="text-align:center;padding:48px;color:#999;">No entries found.</div>';
            return;
        }

        container.innerHTML = data.map((item, i) => `
            <div class="leaderboard-item">
                <span class="rank">#${(i + 1).toString().padStart(2, '0')}</span>
                <span class="username">${escapeHtml(item.uploader)}</span>
                <span class="timestamp">${formatTime(item.created_at)}</span>
                <span class="score">${formatNumber(item.value)}</span>
            </div>
        `).join('');

        updateDistChart(data);
    } catch (err) { console.error(err); }
}

function updateDistChart(data) {
    const counts = [0, 0, 0, 0, 0, 0];
    data.forEach(item => {
        const v = item.value;
        if (v < 200) counts[0]++;
        else if (v < 400) counts[1]++;
        else if (v < 600) counts[2]++;
        else if (v < 800) counts[3]++;
        else if (v < 1000) counts[4]++;
        else counts[5]++;
    });
    charts.distribution.data.datasets[0].data = counts;
    charts.distribution.update('none');
}

async function fetchStats() {
    try {
        const res = await fetch(`${API_BASE_URL}/info`);
        const d = await res.json();

        document.getElementById('totalEntries').textContent = d.count.toLocaleString();
        document.getElementById('meanScore').textContent = formatNumber(d.mean);
        document.getElementById('scoreRange').textContent = `${formatNumber(d.min)} – ${formatNumber(d.max)}`;

        document.getElementById('statMedian').textContent = formatNumber(d.median);
        document.getElementById('statStdDev').textContent = formatNumber(d.stddev_pop);
        document.getElementById('statMode').textContent = formatNumber(d.mode);
        document.getElementById('statIQR').textContent = formatNumber(d.iqr);
        document.getElementById('statP25').textContent = `${formatNumber(d.p25)} / ${formatNumber(d.p75)}`;
        document.getElementById('statVariance').textContent = formatNumber(d.variance);

        const start = parseDate(d.earliest_at);
        const end = parseDate(d.latest_at);
        if (start && end) {
            minTimestamp = start; maxTimestamp = end;
            updateRangeLabels();
        }
    } catch (err) { console.error(err); }
}

async function fetchPerformance() {
    try {
        const res = await fetch(`${API_BASE_URL}/performance`);
        const data = await res.json();
        const container = document.getElementById('perfTable');
        const head = '<div class="perf-header"><span>Endpoint</span><span>Latency</span><span>Count</span></div>';

        if (!data.length) {
            container.innerHTML = head + '<div style="text-align:center;padding:16px;color:#999;font-size:12px;">No performance data.</div>';
            return;
        }
        container.innerHTML = head + data.map(e => `
            <div class="perf-row">
                <span class="endpoint">${escapeHtml(e.endpoint)}</span>
                <span>${e.avg_ms.toFixed(2)}ms</span>
                <span>${e.count}</span>
            </div>
        `).join('');
    } catch (err) { console.error(err); }
}

function updateThroughputChart() {
    const now = Date.now();
    const sec = (now - lastUpdate) / 1000;
    if (sec >= 1) {
        const rps = Math.round(requestCounter / sec);
        throughputHistory.push(rps);
        if (throughputHistory.length > 30) throughputHistory.shift();
        charts.timeline.data.datasets[0].data = throughputHistory;
        charts.timeline.update('none');
        requestCounter = 0;
        lastUpdate = now;
    }
}

async function fetchHistory() {
    const user = document.getElementById('historyUser').value.trim();
    const params = new URLSearchParams({ count: PAGE_SIZE, page: historyPage });
    if (user) params.set('title', user);

    const start = getSliderDate('rangeMin');
    const end = getSliderDate('rangeMax');
    if (start) params.set('start', new Date(start.getTime() - 1000).toISOString());
    if (end) params.set('end', new Date(end.getTime() + 1000).toISOString());

    try {
        const res = await fetch(`${API_BASE_URL}/history?${params}`);
        const data = await res.json();
        const container = document.getElementById('historyResults');

        if (!data.length) {
            container.innerHTML = '<div style="text-align:center;padding:32px;color:#999;font-size:12px;">No activity found.</div>';
        } else {
            container.innerHTML = data.map(e => `
                <div class="history-row">
                    <span><strong>${escapeHtml(e.uploader)}</strong> uploaded <strong>${formatNumber(e.value)}</strong></span>
                    <span class="time">${formatTime(e.created_at)}</span>
                </div>
            `).join('');
        }
        document.getElementById('historyPageLabel').textContent = `Page ${historyPage}`;
        document.getElementById('historyPrev').disabled = historyPage <= 1;
        document.getElementById('historyNext').disabled = data.length < PAGE_SIZE;
    } catch (err) { console.error(err); }
}

function getSliderDate(id) {
    if (!minTimestamp || !maxTimestamp) return null;
    const val = parseInt(document.getElementById(id).value) / 1000;
    const range = maxTimestamp.getTime() - minTimestamp.getTime();
    return new Date(minTimestamp.getTime() + range * val);
}

function refreshRangeUI() {
    const min = parseInt(document.getElementById('rangeMin').value);
    const max = parseInt(document.getElementById('rangeMax').value);
    const track = document.getElementById('rangeTrack');
    track.style.setProperty('--range-left', (min / 10) + '%');
    track.style.setProperty('--range-width', ((max - min) / 10) + '%');
    updateRangeLabels();
}

function updateRangeLabels() {
    const s = getSliderDate('rangeMin');
    const e = getSliderDate('rangeMax');
    const options = { month: 'numeric', day: 'numeric', hour: 'numeric', minute: 'numeric', second: 'numeric', hour12: true };
    document.getElementById('rangeStartLabel').textContent = s ? s.toLocaleString(undefined, options) : '--';
    document.getElementById('rangeEndLabel').textContent = e ? e.toLocaleString(undefined, options) : '--';
}

async function onAddSubmit(e) {
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
            requestCounter++;
            notify('Record added successfully');
            document.getElementById('addEntryForm').reset();
        }
    } catch (err) { notify('Failed to add record', true); }
}

function startLoadTest() {
    if (testRunning) return;
    testRunning = true;
    document.getElementById('startStress').disabled = true;
    document.getElementById('stopStress').disabled = false;
    const intensity = parseInt(document.getElementById('stressIntensity').value);

    // Calculate requests per second: 1-10 = slow (1-10 req/s), 11-50 = fast (20-200 req/s)
    let requestsPerSecond;
    let intervalMs;

    if (intensity <= 10) {
        // Slow mode: 1-10 concurrent users = 1-10 requests per second
        requestsPerSecond = intensity;
        intervalMs = 1000; // Run once per second
    } else {
        // Fast mode: 11-50 concurrent users = 20-200 requests per second
        requestsPerSecond = (intensity - 10) * 5;
        intervalMs = 100; // Run 10 times per second
    }

    testInterval = setInterval(async () => {
        const batch = [];
        const batchSize = intervalMs === 1000 ? requestsPerSecond : Math.ceil(requestsPerSecond / 10);

        for (let i = 0; i < batchSize; i++) {
            const user = SIMULATED_USERS[Math.floor(Math.random() * SIMULATED_USERS.length)];
            const val = parseFloat((Math.random() * 1000).toFixed(2));
            batch.push(fetch(`${API_BASE_URL}/add`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ key: `${user}_${Math.floor(Math.random() * 100)}`, value: val })
            }).then(() => requestCounter++).catch(() => { }));
        }
        await Promise.all(batch);

        const statusText = intensity <= 10
            ? `Simulating ${requestsPerSecond} event${requestsPerSecond > 1 ? 's' : ''}/sec`
            : `Simulating ~${requestsPerSecond} events/sec`;
        document.getElementById('stressStatus').textContent = statusText;
    }, intervalMs);
}

function stopLoadTest() {
    testRunning = false;
    clearInterval(testInterval);
    document.getElementById('startStress').disabled = false;
    document.getElementById('stopStress').disabled = true;
    document.getElementById('stressStatus').textContent = 'System Idle';
}


// helpres
function formatNumber(v) { return v != null ? v.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }) : '0.00'; }
function escapeHtml(t) { const d = document.createElement('div'); d.textContent = t; return d.innerHTML; }
function formatTime(arr) { if (!arr) return "--"; return new Date(Date.UTC(arr[0], 0, arr[1], arr[2], arr[3], arr[4])).toLocaleTimeString(); }
function parseDate(arr) { if (!arr) return null; return new Date(Date.UTC(arr[0], 0, arr[1], arr[2], arr[3], arr[4])); }

function notify(msg, isErr = false) {
    const el = document.getElementById('statusMessage');
    el.textContent = msg;
    el.classList.add('show');
    el.style.backgroundColor = isErr ? 'var(--danger)' : 'var(--text)';
    setTimeout(() => el.classList.remove('show'), 2000);
}

function fetchConfig() {
    fetch(`${API_BASE_URL}/boardconfig`).then(r => r.json()).then(c => {
        document.querySelector('header h1').textContent = c.title;
        document.getElementById('activeSortOrder').textContent = c.sort_order.toUpperCase();
    }).catch(() => { });
}

function switchView() {
    const reg = document.getElementById('regularView');
    const adm = document.getElementById('adminView');
    const btn = document.getElementById('toggleViewBtn');
    if (viewMode === 'regular') {
        reg.style.display = 'none'; adm.style.display = 'block';
        btn.textContent = 'Back to Dashboard'; viewMode = 'admin';
    } else {
        reg.style.display = 'block'; adm.style.display = 'none';
        btn.textContent = 'Admin Settings'; viewMode = 'regular';
    }
}

async function onAdminAuth(e) {
    e.preventDefault();
    const token = document.getElementById('adminToken').value;
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
            notify('Authentication successful');
        } else { notify('Invalid token', true); }
    } catch (err) { notify('Connection error', true); }
}

async function onUpdateTitle(e) {
    e.preventDefault();
    const title = document.getElementById('newTitle').value.trim();
    updateConfig({ title });
}

async function onUpdateSort(e) {
    e.preventDefault();
    const sort_order = document.getElementById('sortOrder').value;
    updateConfig({ sort_order });
}

async function updateConfig(body) {
    try {
        const res = await fetch(`${API_BASE_URL}/admin/config`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${adminToken}` },
            body: JSON.stringify(body)
        });
        if (res.ok) { notify('Settings updated'); fetchConfig(); }
    } catch (err) { notify('Failed to update', true); }
}

function onLogout() {
    adminToken = null;
    document.getElementById('adminPanel').style.display = 'none';
    document.getElementById('adminAuthSection').style.display = 'block';
    notify('Logged out');
}
