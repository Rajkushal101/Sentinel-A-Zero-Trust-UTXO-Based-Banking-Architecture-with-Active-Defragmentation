// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\frontend\common\api.js

// Auto-detect Dev Mode (Python local server on 8000) vs Prod (Docker/Nginx proxy)
const API_BASE_URL = window.location.port === '8000'
    ? `http://${window.location.hostname}:3001/api`   // Dev: dynamically use current hostname for local network support
    : `http://${window.location.hostname}:3001/api`;                        // Prod: relative path via Nginx reverse proxy

// Global base URL accessible by all pages (admin pages use this too)
const API_BASE = API_BASE_URL;

const api = {
    async request(endpoint, method = 'GET', body = null) {
        let token = null;
        if (window.secureStorage) {
            token = await window.secureStorage.getItem('sentinel_token');
        }
        if (!token) token = sessionStorage.getItem('sentinel_token');
        
        const headers = {
            'Content-Type': 'application/json',
            'Accept': 'application/json'
        };

        if (token) {
            headers['Authorization'] = `Bearer ${token}`;
        }

        // Send device fingerprint with every request for server-side validation
        let fp = null;
        if (window.secureStorage) {
            fp = await window.secureStorage.getItem('sentinel_device_fp');
        }
        if (!fp) fp = sessionStorage.getItem('sentinel_device_fp');
        if (fp) {
            headers['X-Device-Fingerprint'] = fp;
        }

        const config = { method, headers };

        if (body) {
            config.body = JSON.stringify(body);
        }

        try {
            const response = await fetch(`${API_BASE}${endpoint}`, config);

            if (response.status === 401) {
                console.warn("Session Expired or Unauthorized");
                if (window.secureStorage) window.secureStorage.clear();
                sessionStorage.clear();
                window.location.href = '/auth/login.html';
                return null;
            }

            if (response.status === 403) {
                throw new Error("Access Denied: Account Frozen or Insufficient Permissions");
            }

            const data = await response.json();

            if (!response.ok) {
                throw new Error(data || `Server Error: ${response.status}`);
            }

            return data;

        } catch (error) {
            console.error(`API Error [${endpoint}]:`, error);
            throw error;
        }
    },

    get(endpoint) { return this.request(endpoint, 'GET'); },
    post(endpoint, body) { return this.request(endpoint, 'POST', body); }
};
