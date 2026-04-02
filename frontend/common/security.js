// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\frontend\common\security.js

/**
 * Sentinel Active Defense Module
 * Implements "Zero-Trust Client" restrictions.
 */
class SecurityManager {
    constructor() {
        this.initListeners();
        this.initMutationObserver();
    }

    initListeners() {
        // 1. Disable Right Click (Context Menu)
        document.addEventListener('contextmenu', event => {
            event.preventDefault();
            this.logViolation('RIGHT_CLICK_ATTEMPT');
        });

        // 2. Disable Copy/Cut/Paste on sensitive fields
        // Note: Specific inputs use oncopy="return false", this is a global fallback
        document.addEventListener('copy', e => this.checkSensitive(e));
        document.addEventListener('cut', e => this.checkSensitive(e));
        document.addEventListener('paste', e => this.checkSensitive(e));

        // 3. Block DevTools Shortcuts & Screenshots
        document.addEventListener('keydown', e => {
            // F12
            if (e.key === 'F12') {
                e.preventDefault();
                this.logViolation('DEVTOOLS_ATTEMPT');
            }
            // Ctrl+Shift+I / Ctrl+Shift+J / Ctrl+U
            if (e.ctrlKey && (e.shiftKey && (e.key === 'I' || e.key === 'J') || e.key === 'u')) {
                e.preventDefault();
                this.logViolation('SOURCE_VIEW_ATTEMPT');
            }
            // PrintScreen (Detect attempt)
            if (e.key === 'PrintScreen') {
                this.logViolation('SCREENSHOT_ATTEMPT');
                // Creating a flash effect to ruin the screenshot visually
                document.body.style.filter = 'blur(10px) invert(1)';
                setTimeout(() => { document.body.style.filter = 'none'; }, 500);
            }
        });
    }

    checkSensitive(e) {
        // If target is an input or has secure class, block it
        if (e.target.tagName === 'INPUT' || e.target.classList.contains('secure-field')) {
            e.preventDefault();
            // Optional: Show warning toast
        }
    }

    // 4. Anti-Tamper: Detect if malicious extensions inject scripts
    initMutationObserver() {
        const observer = new MutationObserver(mutations => {
            mutations.forEach(mutation => {
                mutation.addedNodes.forEach(node => {
                    if (node.tagName === 'SCRIPT' && node.src !== '' && !node.src.includes('common') && !node.src.includes('/auth/') && !node.src.includes('/admin/') && !node.src.includes('/user/')) {
                        console.warn("🚨 FOREIGN SCRIPT DETECTED");
                        node.remove(); // Kill it
                        this.logViolation('SCRIPT_INJECTION');
                    }
                });
            });
        });

        observer.observe(document.documentElement, { childList: true, subtree: true });
    }

    logViolation(type) {
        console.warn(`SECURITY VIOLATION: ${type}`);
        // In full prod, send this to /api/security/log
        // fetch('/api/security/log', { method: 'POST', body: ... })
    }
}

// Auto-activate on load
const sentinelSecurity = new SecurityManager();
