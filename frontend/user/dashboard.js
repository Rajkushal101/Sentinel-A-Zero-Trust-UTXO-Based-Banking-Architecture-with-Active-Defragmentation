// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\frontend\user\dashboard.js

document.addEventListener('DOMContentLoaded', async () => {
    // 1. Check Session (with fallback if secureStorage unavailable)
    let token = null;
    if (window.secureStorage) {
        token = await window.secureStorage.getItem('sentinel_token');
    }
    if (!token) token = sessionStorage.getItem('sentinel_token');
    if (!token) {
        window.location.href = '../auth/login.html';
        return;
    }

    // 2. Fetch Wallet Data
    loadWalletData();
});

async function loadWalletData() {
    try {
        const data = await api.get('/wallet/balance');
        
        // Update UI
        document.getElementById('balance-display').innerText = data.balance.toLocaleString('en-IN');
        document.getElementById('utxo-count').innerText = data.utxo_count;

        // Render UTXO Visualization (Research Demo)
        renderUtxoVisuals(data.balance, data.utxo_count);

    } catch (e) {
        console.error("Wallet Load Error:", e);
        if(e.message.includes("401")) logout();
    }
}

function renderUtxoVisuals(balance, count) {
    const grid = document.getElementById('utxo-grid');
    grid.innerHTML = '';

    if (count === 0) {
        grid.innerHTML = '<div class="col-span-4 text-center text-slate-500 py-8">Wallet is empty. Ask Admin to Mint.</div>';
        return;
    }

    // Since API currently returns aggregate, we simulate the breakdown for the UI prototype
    // In full version, API would return list of Token IDs
    const avgVal = Math.floor(balance / count);
    
    // Limit visuals to 8 to prevent DOM flooding
    const displayCount = Math.min(count, 8);

    for (let i = 0; i < displayCount; i++) {
        const note = document.createElement('div');
        note.className = "bg-slate-800 p-4 rounded-xl border border-slate-700 relative overflow-hidden group hover:-translate-y-1 transition duration-300";
        note.innerHTML = `
            <div class="text-[10px] text-slate-500 font-mono mb-1">DIGITAL NOTE</div>
            <div class="text-xl font-bold text-emerald-400">₹${avgVal}</div>
            <div class="absolute -bottom-4 -right-4 w-12 h-12 bg-emerald-500/10 rounded-full blur-xl group-hover:bg-emerald-500/20 transition"></div>
        `;
        grid.appendChild(note);
    }

    if (count > 8) {
        const more = document.createElement('div');
        more.className = "flex items-center justify-center text-slate-500 text-sm font-mono";
        more.innerText = `+ ${count - 8} more notes...`;
        grid.appendChild(more);
    }
}

function logout() {
    if (window.secureStorage) window.secureStorage.clear();
    sessionStorage.clear();
    window.location.href = '../auth/login.html';
}