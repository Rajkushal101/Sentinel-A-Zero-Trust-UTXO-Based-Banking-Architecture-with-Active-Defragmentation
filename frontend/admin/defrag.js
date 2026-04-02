document.addEventListener('DOMContentLoaded', async () => {
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
            // Auto load on init
            refreshData();
        }
    })();
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

async function logout() {
    if (window.secureStorage) await window.secureStorage.clear();
    sessionStorage.clear();
    window.location.href = '../auth/login.html';
}

function switchTab(tabId) {
    document.getElementById('defrag-tab').classList.add('hidden');
    document.getElementById('tx-tab').classList.add('hidden');

    document.getElementById('btn-defrag-tab').className = "py-2 border-b-2 border-transparent text-slate-400 hover:text-slate-200";
    document.getElementById('btn-tx-tab').className = "py-2 border-b-2 border-transparent text-slate-400 hover:text-slate-200";

    document.getElementById(tabId).classList.remove('hidden');
    document.getElementById('btn-' + tabId).className = "py-2 border-b-2 border-purple-500 text-purple-400 font-medium";
}

async function refreshData() {
    await fetchDefragHistory();
    await fetchTransactions();
}

async function triggerGlobalDefrag() {
    const btn = document.getElementById('global-defrag-btn');
    const status = document.getElementById('defrag-status');
    btn.disabled = true;
    btn.innerText = 'PROCESSING...';
    status.className = "text-sm font-mono text-blue-400";
    status.innerText = "Running global defragmentation...";
    status.classList.remove('hidden');

    try {
        const response = await fetch(`${API_BASE}/admin/defrag`, {
            method: 'POST',
            headers: await getAuthHeaders(),
            body: JSON.stringify({}) // empty payload for global defrag
        });

        let data = [];
        try {
            data = await response.json();
        } catch(e) {}

        if (response.ok) {
            status.className = "text-sm font-mono text-green-400";
            status.innerText = "SUCCESS: Defragmentation Complete";
        } else {
            status.className = "text-sm font-mono text-red-400";
            status.innerText = "FAILED: " + (data.error || "Server Error");     
            console.error("Defrag Failed", data);
        }
    } catch (error) {
        status.className = "text-sm font-mono text-red-400";
        status.innerText = "NETWORK ERROR";
    }

    btn.disabled = false;
    btn.innerText = 'EXECUTE GLOBAL DEFRAG';

    // Automatically retrieve the newest logs
    refreshData();
}

async function fetchDefragHistory() {
    const tbody = document.getElementById('defrag-logs-table');

    try {
        const response = await fetch(`${API_BASE}/admin/history`, {
            method: 'GET',
            headers: await getAuthHeaders()
        });

        if (!response.ok) throw new Error('Failed to fetch defrag history');    

        const data = await response.json();

        if (!data.events || data.events.length === 0) {
            tbody.innerHTML = '<tr><td colspan="6" class="p-4 text-center text-slate-500">No defragmentation events found.</td></tr>';
            return;
        }

        tbody.innerHTML = data.events.map(ev => {
            const date = new Date(ev.timestamp);
            const statusColor = ev.status === 'COMPLETED' ? 'text-green-400 bg-green-900/30' : 'text-red-400 bg-red-900/30';
            const valueStr = '₹' + ev.total_value.toLocaleString();
            const accountDisp = ev.account_name ? `<span class="capitalize text-blue-400 font-bold">${ev.account_name}</span> <span class="text-slate-500 text-[10px]">(${ev.user_id.split('-')[0]})</span>` : ev.user_id;

            return `
                <tr class="hover:bg-slate-800/50 transition">
                    <td class="p-4 text-slate-300 font-mono text-xs">${date.toLocaleString()}</td>
                    <td class="p-4">${accountDisp}</td>
                    <td class="p-4">
                        <span class="px-2 py-1 rounded text-xs font-bold ${statusColor}">${ev.status}</span>
                    </td>
                    <td class="p-4 text-red-300 font-mono">${ev.old_utxo_count} <span class="text-xs text-slate-500">tokens</span></td>
                    <td class="p-4 text-green-300 font-mono">${ev.new_utxo_count} <span class="text-xs text-slate-500">standard notes</span></td>
                    <td class="p-4 text-slate-300 font-mono font-bold">${valueStr}</td>
                </tr>
            `;
        }).join('');
    } catch (error) {
        console.error(error);
        tbody.innerHTML = `<tr><td colspan="6" class="p-4 text-center text-red-500">Error loading defrag history: ${error.message}</td></tr>`;
    }
}

async function fetchTransactions() {
    const tbody = document.getElementById('tx-logs-table');

    try {
        const response = await fetch(`${API_BASE}/admin/transactions`, {        
            method: 'GET',
            headers: await getAuthHeaders()
        });

        if (!response.ok) throw new Error('Failed to fetch transactions');      

        const data = await response.json();

        if (!data.transactions || data.transactions.length === 0) {
            tbody.innerHTML = '<tr><td colspan="5" class="p-4 text-center text-slate-500">No transactions found in the global ledger.</td></tr>';
            return;
        }

        tbody.innerHTML = data.transactions.map(tx => {
            const date = new Date(tx.timestamp);

            let typeColor = 'text-slate-300';
            if (tx.tx_type === 'MINT') typeColor = 'text-blue-400';
            if (tx.tx_type === 'TRANSFER') typeColor = 'text-green-400';        
            if (tx.tx_type === 'DEFRAG') typeColor = 'text-purple-400';

            const fromStr = tx.tx_type === 'MINT' || !tx.from_user ? '<span class="text-slate-500">CENTRAL BANK</span>' : tx.from_user;
            const toStr = tx.to_user || 'SYSTEM';

            return `
                <tr class="hover:bg-slate-800/50 transition">
                    <td class="p-4 text-slate-300 font-mono text-xs">${date.toLocaleString()}</td>
                    <td class="p-4 text-white font-mono text-xs">${tx.tx_id.split('-')[0]}...</td>
                    <td class="p-4 font-bold ${typeColor}">${tx.tx_type}</td>   
                    <td class="p-4 text-white font-mono">₹${tx.amount.toLocaleString()}</td>     
                    <td class="p-4 font-mono text-xs">
                        <div class="text-red-400 text-[10px]">From: ${fromStr}</div>
                        <div class="text-green-400 text-[10px]">To: ${toStr}</div>
                    </td>
                </tr>
            `;
        }).join('');
    } catch (error) {
        console.error(error);
        tbody.innerHTML = `<tr><td colspan="5" class="p-4 text-center text-red-500">Error loading transactions: ${error.message}</td></tr>`;
    }
}

// Initial load
document.addEventListener('DOMContentLoaded', refreshData);


