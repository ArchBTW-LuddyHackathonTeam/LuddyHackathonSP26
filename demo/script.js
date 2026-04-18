const API_BASE_URL = 'http://localhost:3000';

let leaderboardData = [];
let statsData = {
    total: 0,
    mean: 0,
    median: 0,
    stdDev: 0,
    min: 0,
    max: 0
};
let performanceData = {
    add: 0,
    remove: 0,
    leaderboard: 0,
    info: 0
};

document.addEventListener('DOMContentLoaded', () => {
    setupEvents();
    fetchLeaderboard();
    loadBoardTitle();
    // Refresh leaderboard every 2 seconds
    setInterval(fetchLeaderboard, 2000);
});

async function loadBoardTitle() {
    try {
        const response = await fetch(`${API_BASE_URL}/boardconfig`);
        if (response.ok) {
            const config = await response.json();
            document.querySelector('header h1').textContent = config.title;
        }
    } catch (error) {
        console.error('Failed to load board title:', error);
    }
}

let stressTestInterval = null;
let stressTestRunning = false;
let adminToken = null;
let currentView = 'regular'; // 'regular' or 'admin'

function setupEvents() {
    document.getElementById('addEntryForm').addEventListener('submit', handleAdd);
    document.getElementById('removeEntryForm').addEventListener('submit', handleRemove);
    document.getElementById('startStress').addEventListener('click', startStressTest);
    document.getElementById('stopStress').addEventListener('click', stopStressTest);
    document.getElementById('toggleViewBtn').addEventListener('click', toggleView);
    document.getElementById('adminAuthForm').addEventListener('submit', handleAdminAuth);
    document.getElementById('updateTitleForm').addEventListener('submit', handleUpdateTitle);
    document.getElementById('updateSortForm').addEventListener('submit', handleUpdateSort);
    document.getElementById('logoutAdmin').addEventListener('click', handleAdminLogout);
}

async function fetchLeaderboard() {
    try {
        const start = performance.now();
        const response = await fetch(`${API_BASE_URL}/leaderboard/json`);
        const duration = Math.round(performance.now() - start);

        if (response.ok) {
            const data = await response.json();
            // Convert API format (uploader, value) to display format (username, score)
            leaderboardData = data.map(entry => ({
                username: entry.uploader,
                score: entry.value
            }));
            performanceData.leaderboard = duration;
            render();
        }
    } catch (error) {
        console.error('Failed to fetch leaderboard:', error);
    }
}

function toggleView() {
    const regularView = document.getElementById('regularView');
    const adminView = document.getElementById('adminView');
    const toggleBtn = document.getElementById('toggleViewBtn');

    if (currentView === 'regular') {
        regularView.style.display = 'none';
        adminView.style.display = 'flex';
        toggleBtn.textContent = 'Regular View';
        currentView = 'admin';
    } else {
        regularView.style.display = 'flex';
        adminView.style.display = 'none';
        toggleBtn.textContent = 'Admin Mode';
        currentView = 'regular';
    }
}

async function handleAdd(e) {
    e.preventDefault();
    const username = document.getElementById('username').value.trim();
    const score = parseFloat(document.getElementById('score').value);

    if (!username || isNaN(score)) return;

    try {
        const start = performance.now();
        const response = await fetch(`${API_BASE_URL}/add`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ key: username, value: score })
        });
        const duration = Math.round(performance.now() - start);
        performanceData.add = duration;

        if (response.ok) {
            document.getElementById('addEntryForm').reset();
            showStatus('Added');
            await fetchLeaderboard();
        } else {
            showStatus('Failed to add', true);
        }
    } catch (error) {
        showStatus('Error: ' + error.message, true);
    }
}

async function handleRemove(e) {
    e.preventDefault();
    const username = document.getElementById('removeUsername').value.trim();

    if (!username) return;

    try {
        const start = performance.now();
        const response = await fetch(`${API_BASE_URL}/remove/${encodeURIComponent(username)}`, {
            method: 'DELETE'
        });
        const duration = Math.round(performance.now() - start);
        performanceData.remove = duration;

        if (response.ok) {
            document.getElementById('removeEntryForm').reset();
            showStatus('Removed');
            await fetchLeaderboard();
        } else if (response.status === 404) {
            showStatus('Not found', true);
        } else {
            showStatus('Failed to remove', true);
        }
    } catch (error) {
        showStatus('Error: ' + error.message, true);
    }
}

function render() {
    renderLeaderboard();
    renderStats();
    renderPerformance();
}

function renderLeaderboard() {
    const el = document.getElementById('leaderboard');
    const top10 = leaderboardData.slice(0, 10);

    if (top10.length === 0) {
        el.innerHTML = '<div style="text-align: center; padding: 20px; color: #888;">No entries yet</div>';
        return;
    }

    el.innerHTML = top10.map((entry, i) => `
        <div class="leaderboard-item">
            <span class="rank">${i + 1}</span>
            <span class="username">${esc(entry.username)}</span>
            <span class="score">${entry.score.toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2})}</span>
        </div>
    `).join('');
}

function renderStats() {
    const scores = leaderboardData.map(e => e.score);
    const n = scores.length;

    if (n === 0) {
        document.getElementById('totalEntries').textContent = '0';
        document.getElementById('meanScore').textContent = '0';
        document.getElementById('medianScore').textContent = '0';
        document.getElementById('stdDev').textContent = '0';
        document.getElementById('minScore').textContent = '0';
        document.getElementById('maxScore').textContent = '0';
        return;
    }

    const mean = scores.reduce((a, b) => a + b, 0) / n;
    const sorted = [...scores].sort((a, b) => a - b);
    const median = n % 2 === 0
        ? (sorted[n/2 - 1] + sorted[n/2]) / 2
        : sorted[Math.floor(n/2)];

    const variance = scores.reduce((sum, x) => sum + Math.pow(x - mean, 2), 0) / n;
    const stdDev = Math.sqrt(variance);

    document.getElementById('totalEntries').textContent = n;
    document.getElementById('meanScore').textContent = mean.toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2});
    document.getElementById('medianScore').textContent = median.toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2});
    document.getElementById('stdDev').textContent = stdDev.toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2});
    document.getElementById('minScore').textContent = Math.min(...scores).toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2});
    document.getElementById('maxScore').textContent = Math.max(...scores).toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2});
}

function renderPerformance() {
    document.getElementById('addPerf').textContent = performanceData.add + 'ms';
    document.getElementById('removePerf').textContent = performanceData.remove + 'ms';
    document.getElementById('leaderboardPerf').textContent = performanceData.leaderboard + 'ms';
    document.getElementById('infoPerf').textContent = performanceData.info + 'ms';
}

function showStatus(msg, isError = false) {
    const el = document.getElementById('statusMessage');
    el.textContent = msg;
    el.className = isError ? 'show error' : 'show';
    setTimeout(() => el.classList.remove('show'), 2000);
}

function esc(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function startStressTest() {
    if (stressTestRunning) return;

    stressTestRunning = true;
    document.getElementById('startStress').disabled = true;
    document.getElementById('stopStress').disabled = false;
    document.getElementById('stressStatus').textContent = 'Running...';

    let count = 0;
    stressTestInterval = setInterval(async () => {
        const username = `User${Math.floor(Math.random() * 10000)}`;
        const score = parseFloat((Math.random() * 10000).toFixed(2));

        try {
            await fetch(`${API_BASE_URL}/add`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ key: username, value: score })
            });
            count++;
            document.getElementById('stressStatus').textContent = `Running... (${count} requests)`;
        } catch (error) {
            console.error('Stress test error:', error);
        }
    }, 100);
}

function stopStressTest() {
    if (!stressTestRunning) return;

    stressTestRunning = false;
    clearInterval(stressTestInterval);
    document.getElementById('startStress').disabled = false;
    document.getElementById('stopStress').disabled = true;
    document.getElementById('stressStatus').textContent = 'Idle';

    fetchLeaderboard();
}

async function handleAdminAuth(e) {
    e.preventDefault();
    const token = document.getElementById('adminToken').value.trim();

    if (!token) return;

    try {
        const response = await fetch(`${API_BASE_URL}/admin/config`, {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({})
        });

        if (response.ok) {
            adminToken = token;
            document.getElementById('adminAuthSection').style.display = 'none';
            document.getElementById('adminPanel').style.display = 'flex';
            showStatus('Admin authenticated');
            await loadBoardConfig();
        } else {
            showStatus('Invalid admin token', true);
            document.getElementById('adminStatus').textContent = 'Authentication failed';
            document.getElementById('adminStatus').className = 'admin-status error';
        }
    } catch (error) {
        showStatus('Authentication error', true);
        document.getElementById('adminStatus').textContent = 'Error';
        document.getElementById('adminStatus').className = 'admin-status error';
    }
}

async function loadBoardConfig() {
    try {
        const response = await fetch(`${API_BASE_URL}/boardconfig`);
        if (response.ok) {
            const config = await response.json();
            document.getElementById('newTitle').placeholder = `Current: ${config.title}`;
            document.getElementById('sortOrder').value = config.sort_order;
        }
    } catch (error) {
        console.error('Failed to load board config:', error);
    }
}

async function handleUpdateTitle(e) {
    e.preventDefault();
    const newTitle = document.getElementById('newTitle').value.trim();

    if (!newTitle || !adminToken) return;

    try {
        const response = await fetch(`${API_BASE_URL}/admin/config`, {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${adminToken}`
            },
            body: JSON.stringify({ title: newTitle })
        });

        if (response.ok) {
            showStatus('Title updated');
            document.getElementById('updateTitleForm').reset();
            await loadBoardTitle();
            await loadBoardConfig();
        } else {
            showStatus('Failed to update title', true);
        }
    } catch (error) {
        showStatus('Update error', true);
    }
}

async function handleUpdateSort(e) {
    e.preventDefault();
    const sortOrder = document.getElementById('sortOrder').value;

    if (!sortOrder || !adminToken) return;

    try {
        const response = await fetch(`${API_BASE_URL}/admin/config`, {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${adminToken}`
            },
            body: JSON.stringify({ sort_order: sortOrder })
        });

        if (response.ok) {
            showStatus('Sort order updated');
            await loadBoardConfig();
            await fetchLeaderboard();
        } else {
            showStatus('Failed to update sort order', true);
        }
    } catch (error) {
        showStatus('Update error', true);
    }
}

function handleAdminLogout() {
    adminToken = null;
    document.getElementById('adminPanel').style.display = 'none';
    document.getElementById('adminAuthSection').style.display = 'block';
    document.getElementById('adminAuthForm').reset();
    document.getElementById('adminStatus').textContent = '';
    document.getElementById('adminStatus').className = 'admin-status';
    showStatus('Logged out');
}

// API helper functions for console debugging
window.api = {
    async add(username, score) {
        const res = await fetch(`${API_BASE_URL}/add`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ key: username, value: score })
        });
        return res.json();
    },

    async remove(username) {
        const res = await fetch(`${API_BASE_URL}/remove/${encodeURIComponent(username)}`, {
            method: 'DELETE'
        });
        return res.status;
    },

    async getLeaderboard() {
        const res = await fetch(`${API_BASE_URL}/leaderboard/json`);
        return res.json();
    },

    async getBoardConfig() {
        const res = await fetch(`${API_BASE_URL}/boardconfig`);
        return res.json();
    }
};
