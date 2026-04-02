import os
import glob
import re

NEW_AUTH_HEADERS = """async function getAuthHeaders() {
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
}"""

DASHBOARD_IIFE = """// Admin Session Check
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
})();"""

files_to_check = glob.glob('../frontend/admin/*.html') + ['../frontend/admin/dashboard.js']
print(f"Found {len(files_to_check)} files")

for filepath in files_to_check:
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    pattern = re.compile(r'async function getAuthHeaders\(\)\s*\{.*?return\s*\{.*?\};\s*\}', re.DOTALL)
    content = pattern.sub(NEW_AUTH_HEADERS, content)
    
    if 'dashboard.js' in filepath:
        content = content.replace("document.body.style.display = 'flex';", DASHBOARD_IIFE)
        
        auth_err_fix = """if (response.status === 401 || response.status === 403) {
            window.location.href = '../auth/login.html';
            return;
        }
        if (!response.ok) throw new Error('Stats fetch failed');"""
        content = content.replace("if (!response.ok) throw new Error('Stats fetch failed');", auth_err_fix)

    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

print("Done fixing admin files.")