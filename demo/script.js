let mockLeaderboard = [
    { username: "Alice", score: 95.75 },
    { username: "Bob", score: 87.50 },
    { username: "Charlie", score: 82.25 },
    { username: "Diana", score: 78.00 },
    { username: "Eve", score: 75.33 },
    { username: "Frank", score: 69.67 },
    { username: "Grace", score: 64.12 },
    { username: "Henry", score: 58.99 },
    { username: "Iris", score: 52.45 },
    { username: "Jack", score: 47.88 }
];

let mockPerformance = {
    add: 12,
    remove: 8,
    leaderboard: 15,
    info: 10
};

const API_BASE_URL = 'http://localhost:8000';

document.addEventListener('DOMContentLoaded', () => {
    render();
    setupEvents();
});

let stressTestInterval = null;
let stressTestRunning = false;

function setupEvents() {
    document.getElementById('addEntryForm').addEventListener('submit', handleAdd);
    document.getElementById('removeEntryForm').addEventListener('submit', handleRemove);
    document.getElementById('startStress').addEventListener('click', startStressTest);
    document.getElementById('stopStress').addEventListener('click', stopStressTest);
}

async function handleAdd(e) {
    e.preventDefault();
    const username = document.getElementById('username').value.trim();
    const score = parseFloat(document.getElementById('score').value);

    if (!username || isNaN(score)) return;

    const index = mockLeaderboard.findIndex(e => e.username === username);
    if (index !== -1) {
        mockLeaderboard[index].score = score;
    } else {
        mockLeaderboard.push({ username, score });
    }

    mockLeaderboard.sort((a, b) => b.score - a.score);

    document.getElementById('addEntryForm').reset();
    render();
    showStatus('Added');
}

async function handleRemove(e) {
    e.preventDefault();
    const username = document.getElementById('removeUsername').value.trim();

    const index = mockLeaderboard.findIndex(e => e.username === username);
    if (index !== -1) {
        mockLeaderboard.splice(index, 1);
        showStatus('Removed');
    } else {
        showStatus('Not found', true);
    }

    document.getElementById('removeEntryForm').reset();
    render();
}

function render() {
    renderLeaderboard();
    renderStats();
    renderPerformance();
}

function renderLeaderboard() {
    const el = document.getElementById('leaderboard');
    const top10 = mockLeaderboard.slice(0, 10);

    el.innerHTML = top10.map((entry, i) => `
        <div class="leaderboard-item">
            <span class="rank">${i + 1}</span>
            <span class="username">${esc(entry.username)}</span>
            <span class="score">${entry.score.toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2})}</span>
        </div>
    `).join('');
}

function renderStats() {
    const scores = mockLeaderboard.map(e => e.score);
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
    document.getElementById('addPerf').textContent = mockPerformance.add + 'ms';
    document.getElementById('removePerf').textContent = mockPerformance.remove + 'ms';
    document.getElementById('leaderboardPerf').textContent = mockPerformance.leaderboard + 'ms';
    document.getElementById('infoPerf').textContent = mockPerformance.info + 'ms';
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
    stressTestInterval = setInterval(() => {
        const username = `User${Math.floor(Math.random() * 10000)}`;
        const score = parseFloat((Math.random() * 10000).toFixed(2));

        const index = mockLeaderboard.findIndex(e => e.username === username);
        if (index !== -1) {
            mockLeaderboard[index].score = score;
        } else {
            mockLeaderboard.push({ username, score });
        }

        mockLeaderboard.sort((a, b) => b.score - a.score);
        count++;

        if (count % 10 === 0) {
            render();
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

    render();
}

window.api = {
    async add(username, score) {
        const res = await fetch(`${API_BASE_URL}/add`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, score })
        });
        return res.json();
    },

    async remove(username) {
        const res = await fetch(`${API_BASE_URL}/remove`, {
            method: 'DELETE',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username })
        });
        return res.json();
    },

    async getLeaderboard() {
        const res = await fetch(`${API_BASE_URL}/leaderboard`);
        return res.json();
    },

    async getInfo() {
        const res = await fetch(`${API_BASE_URL}/info`);
        return res.json();
    },

    async getPerformance() {
        const res = await fetch(`${API_BASE_URL}/performance`);
        return res.json();
    }
};
