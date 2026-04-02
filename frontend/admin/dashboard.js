// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\frontend\admin\dashboard.js

let killSwitchEnabled = true; // tracks current state

// Initialize Dashboard
document.addEventListener('DOMContentLoaded', async () => {
    // AUTH BYPASS: Reveal the page immediately
    // Admin Session Check
(async function () {
    let token = null, role = null;
    if (window.secureStorage) {
        token = await window.secureStorage.getItem('sentinel_token');
        role = await window.secureStorage.getItem('sentinel_role');
    }
    if (!token) token = sessionStorage.getItem('sentinel_token');
    if (!role) role = sessionStorage.getItem('sentinel_role');
    
    if (!token || role !== 'admin') {
        window.location.href = '../auth/login.html';
    } else {
        document.body.style.display = 'flex';
    }
})();

    // 2. Load Stats
    await loadSystemStats();

    // 3. Initialize Charts
    initCharts();
});

async function getAuthHeaders() {
    let token = null, fp = null;
    if (window.secureStorage) {
        token = await window.secureStorage.getItem('sentinel_token');
        fp = await window.secureStorage.getItem('sentinel_device_fp');
    }
    if (!token) token = sessionStorage.getItem('sentinel_token');
    if (!fp) fp = sessionStorage.getItem('sentinel_device_fp');
    
    let headers = {
        'Authorization': 'Bearer ' + token,
        'Content-Type': 'application/json'
    };
    if (fp) headers['X-Device-Fingerprint'] = fp;
    return headers;
}

async function loadSystemStats() {
    try {
        const response = await fetch(`${API_BASE}/admin/stats`, { headers: await getAuthHeaders() });
        if (response.status === 401 || response.status === 403) {
            window.location.href = '../auth/login.html';
            return;
        }
        if (!response.ok) throw new Error('Stats fetch failed');
        const data = await response.json();

        document.getElementById('stat-supply').innerText = `₹${(data.total_supply || 0).toLocaleString()}`;
        document.getElementById('stat-users').innerText = data.user_count || "0";
        document.getElementById('utxo-count').innerText = `Active UTXOs: ${data.active_utxos || 0}`;

        // Update kill switch UI
        killSwitchEnabled = data.transactions_enabled !== false;
        updateKillSwitchUI();

    } catch (e) {
        console.error("Stats Load Error:", e);
        document.getElementById('stat-supply').innerText = 'Error';
        document.getElementById('stat-users').innerText = 'Error';
    }
}

function updateKillSwitchUI() {
    const statusEl = document.getElementById('kill-switch-status');
    const btnEl = document.getElementById('kill-switch-btn');
    if (killSwitchEnabled) {
        statusEl.innerHTML = 'TRANSACTIONS: ENABLED';
        statusEl.className = 'px-3 py-1 bg-green-900/30 text-green-400 border border-green-900/50 rounded-full text-xs font-mono';
        btnEl.innerText = 'DISABLE TRANSACTIONS';
        btnEl.className = 'px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg text-sm font-medium transition';
    } else {
        statusEl.innerHTML = 'TRANSACTIONS: DISABLED';
        statusEl.className = 'px-3 py-1 bg-red-900/30 text-red-400 border border-red-900/50 rounded-full text-xs font-mono animate-pulse';
        btnEl.innerText = 'ENABLE TRANSACTIONS';
        btnEl.className = 'px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg text-sm font-medium transition';
    }
}

async function toggleKillSwitch() {
    const newState = !killSwitchEnabled;
    const action = newState ? 'enable' : 'DISABLE';
    if (!confirm(`Are you sure you want to ${action} all transactions?`)) return;

    try {
        const resp = await fetch(`${API_BASE}/admin/kill-switch`, {
            method: 'POST',
            headers: await getAuthHeaders(),
            body: JSON.stringify({ enabled: newState })
        });
        if (!resp.ok) throw new Error('Kill switch toggle failed');
        const data = await resp.json();
        killSwitchEnabled = data.transactions_enabled;
        updateKillSwitchUI();
    } catch (e) {
        alert('Kill switch error: ' + e.message);
    }
}

async function triggerDefrag() {
    if (!confirm('Defragment all wallets? This consolidates UTXOs for all users.')) return;

    const resultEl = document.getElementById('defrag-result');
    resultEl.classList.remove('hidden');
    resultEl.innerText = 'Running defragmentation...';

    try {
        const resp = await fetch(`${API_BASE}/admin/defrag`, {
            method: 'POST',
            headers: await getAuthHeaders(),
            body: JSON.stringify({})
        });
        if (!resp.ok) throw new Error('Defrag failed');
        const data = await resp.json();
        resultEl.innerText = `✅ ${data.message} — ${data.users_affected} wallets processed`;
        resultEl.className = 'mt-3 text-xs text-green-400';
        await loadSystemStats(); // Refresh stats
    } catch (e) {
        resultEl.innerText = '❌ Defrag error: ' + e.message;
        resultEl.className = 'mt-3 text-xs text-red-400';
    }
}

function initCharts() {
    const ctx = document.getElementById('velocityChart').getContext('2d');
    new Chart(ctx, {
        type: 'line',
        data: {
            labels: ['00:00', '04:00', '08:00', '12:00', '16:00', '20:00'],
            datasets: [{
                label: 'Transactions Per Second',
                data: [12, 19, 45, 120, 85, 30],
                borderColor: '#3b82f6',
                backgroundColor: 'rgba(59, 130, 246, 0.1)',
                tension: 0.4,
                fill: true
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: { legend: { display: false } },
            scales: {
                y: { grid: { color: '#1e293b' }, ticks: { color: '#94a3b8' } },
                x: { grid: { display: false }, ticks: { color: '#94a3b8' } }
            }
        }
    });
}

function logout() {
    if (window.secureStorage) window.secureStorage.clear();
    sessionStorage.clear();
    window.location.href = '../auth/login.html';
}