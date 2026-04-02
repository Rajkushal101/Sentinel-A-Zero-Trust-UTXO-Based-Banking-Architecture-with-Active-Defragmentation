// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\frontend\auth\login.js

document.addEventListener('DOMContentLoaded', () => {
    // 1. Initialize the Living CAPTCHA
    window.captchaHandler.refresh();

    // 2. Bind Form Submit
    document.getElementById('loginForm').addEventListener('submit', handleLogin);
});

async function handleLogin(e) {
    e.preventDefault();
    const btn = e.target.querySelector('button');
    const msg = document.getElementById('status-msg');
    
    // Clear previous status
    msg.innerText = "";
    
    // 1. Collect Data
    const username = document.getElementById('username').value.trim();
    const password = document.getElementById('password').value;
    const captchaAns = document.getElementById('captcha-answer').value.trim();
    const captchaTok = document.getElementById('captcha-id').value;

    if(!username || !password || !captchaAns) {
        msg.innerText = "MISSING CREDENTIALS";
        return;
    }

    // 2. Generate Device Fingerprint (Client Side Binding)
    // In a full research paper, this uses Canvas Fingerprinting + AudioContext.
    // For the prototype, we hash the Navigator Data.
    const rawFingerprint = navigator.userAgent + navigator.hardwareConcurrency + screen.colorDepth + screen.width + new Date().getTimezoneOffset();
    const deviceFp = btoa(rawFingerprint); // Simple base64 for demo (Use SHA-256 in prod)

    // 3. Disable UI
    btn.disabled = true;
    btn.innerHTML = `<span class="animate-spin">⟳</span> VERIFYING...`;

    try {
        // 4. Send to "Edge Gateway"
        const response = await fetch(`${API_BASE}/auth/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                username: username,
                password: password,
                captcha_token: captchaTok,
                captcha_answer: captchaAns,
                device_fp: deviceFp
            })
        });

        const data = await response.json();

        if (response.ok) {
            // 5. Securely Store Token using encrypted storage (Zero-Trust)
            if (window.secureStorage) {
                await window.secureStorage.setItem('sentinel_token', data.token);
                await window.secureStorage.setItem('sentinel_user', data.user_id);
                await window.secureStorage.setItem('sentinel_role', data.role || 'user');
                await window.secureStorage.setItem('sentinel_device_fp', deviceFp);
            } else {
                sessionStorage.setItem('sentinel_token', data.token);
                sessionStorage.setItem('sentinel_user', data.user_id);
                sessionStorage.setItem('sentinel_role', data.role || 'user');
                sessionStorage.setItem('sentinel_device_fp', deviceFp);
            }
            
            // 6. Redirect based on server-provided role (not client-side guess)
            const role = data.role || 'user';
            if(role === 'admin') {
                window.location.href = '../admin/dashboard.html';
            } else {
                window.location.href = '../user/dashboard.html';
            }
        } else {
            throw new Error(data || "Authentication Failed");
        }

    } catch (error) {
        console.error("Login Error:", error);
        msg.innerText = "ACCESS DENIED: " + (error.message || "Invalid Credentials");
        
        // Security Protocol: Refresh CAPTCHA on failure to prevent replay/brute-force
        window.captchaHandler.refresh();
        document.getElementById('password').value = "";
        document.getElementById('captcha-answer').value = "";
    } finally {
        btn.disabled = false;
        btn.innerHTML = `SECURE LOGIN`;
    }
}   