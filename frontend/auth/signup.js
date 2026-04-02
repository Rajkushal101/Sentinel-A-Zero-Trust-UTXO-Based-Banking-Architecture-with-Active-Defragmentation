// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\frontend\auth\signup.js

document.addEventListener('DOMContentLoaded', () => {
    window.captchaHandler.refresh();
    
    document.getElementById('signupForm').addEventListener('submit', handleSignup);
    document.getElementById('password').addEventListener('input', checkStrength);
});

// Simple Password Strength Visualizer
function checkStrength(e) {
    const val = e.target.value;
    const s1 = document.getElementById('strength-1');
    const s2 = document.getElementById('strength-2');
    const s3 = document.getElementById('strength-3');

    // Reset
    s1.className = "flex-1 bg-slate-800 rounded-full transition";
    s2.className = "flex-1 bg-slate-800 rounded-full transition";
    s3.className = "flex-1 bg-slate-800 rounded-full transition";

    if(val.length > 4) s1.className = "flex-1 bg-red-500 rounded-full transition";
    if(val.length > 8 && /[A-Z]/.test(val)) s2.className = "flex-1 bg-yellow-500 rounded-full transition";
    if(val.length > 10 && /[0-9]/.test(val) && /[^A-Za-z0-9]/.test(val)) s3.className = "flex-1 bg-green-500 rounded-full transition";
}

async function handleSignup(e) {
    e.preventDefault();
    const btn = e.target.querySelector('button');
    const msg = document.getElementById('status-msg');
    msg.innerText = "";

    const username = document.getElementById('username').value.trim();
    const password = document.getElementById('password').value;
    const captchaAns = document.getElementById('captcha-answer').value.trim();
    const captchaTok = document.getElementById('captcha-id').value;

    // Client-side Validation
    if(password.length < 8) {
        msg.innerText = "Password must be at least 8 characters.";
        return;
    }

    btn.disabled = true;
    btn.innerText = "GENERATING IDENTITY...";

    try {
        const response = await fetch(`${API_BASE}/auth/signup`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                username,
                password,
                captcha_token: captchaTok,
                captcha_answer: captchaAns
            })
        });

        const data = await response.json();

        if (response.ok) {
            alert("Identity Created Successfully. Please login.");
            window.location.href = 'login.html';
        } else {
            throw new Error(data);
        }

    } catch (error) {
        msg.innerText = "Error: " + error.message;
        window.captchaHandler.refresh();
    } finally {
        btn.disabled = false;
        btn.innerText = "GENERATE KEYS";
    }
}