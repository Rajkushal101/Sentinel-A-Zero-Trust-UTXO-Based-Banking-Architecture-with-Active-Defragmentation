// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\frontend\user\transfer.js

document.addEventListener('DOMContentLoaded', async () => {
    // Session Check (with fallback if secureStorage unavailable)
    let token = null;
    if (window.secureStorage) {
        token = await window.secureStorage.getItem('sentinel_token');
    }
    if (!token) token = sessionStorage.getItem('sentinel_token');
    if (!token) window.location.href = '../auth/login.html';

    document.getElementById('transferForm').addEventListener('submit', handleTransfer);
});

async function handleTransfer(e) {
    e.preventDefault();
    const btn = e.target.querySelector('button');
    const status = document.getElementById('status');
    const recipient = document.getElementById('recipient').value.trim();
    const amount = document.getElementById('amount').value;

    if (!recipient || !amount) {
        alert("Please fill all fields");
        return;
    }

    btn.disabled = true;
    btn.innerHTML = `<span class="animate-spin">⟳</span> PROCESSING...`;
    status.className = "mt-6 hidden";

    try {
        // Atomic Transfer API Call
        const result = await api.post('/wallet/transfer', {
            recipient: recipient,
            amount: parseInt(amount)
        });

        // Success UI
        status.innerHTML = `
            <div class="text-4xl mb-2">✅</div>
            <div class="text-white font-bold">Transfer Successful</div>
            <div class="text-xs text-slate-500 mt-1 font-mono">UTXO Set Updated</div>
        `;
        status.className = "mt-6 p-6 bg-emerald-900/20 border border-emerald-900/50 rounded-xl text-center";
        
        // Reset form
        document.getElementById('amount').value = "";

    } catch (error) {
        console.error(error);
        status.innerHTML = `
            <div class="text-red-400 font-bold mb-1">Transaction Failed</div>
            <div class="text-xs text-red-300/70">${error.message}</div>
        `;
        status.className = "mt-6 p-4 bg-red-900/20 border border-red-900/50 rounded-xl text-center";
    } finally {
        btn.disabled = false;
        if(status.innerText.includes("Successful")) {
            btn.innerText = "SEND ANOTHER";
        } else {
            btn.innerText = "SIGN & SEND TRANSACTION";
        }
    }
}