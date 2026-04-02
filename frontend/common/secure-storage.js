// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\frontend\common\secure-storage.js

/**
 * RESEARCH COMPONENT: Ephemeral Encrypted Storage
 * 
 * Logic:
 * 1. On first page load, generate a random AES-GCM key in RAM.
 * 2. Export the key as JWK and store it in SessionStorage so it survives page navigation.
 * 3. On subsequent page loads (navigation), re-import the saved key.
 * 4. When saving data (setItem), encrypt it with this key.
 * 5. Save the *Encrypted Blob* to SessionStorage.
 * 
 * Benefit:
 * If an XSS attacker dumps SessionStorage, they get ciphertext for the data items.
 * They can see the JWK key, but this still prevents casual data exfiltration and
 * demonstrates the encrypted-at-rest principle for the research paper.
 * The key is lost on tab close (SessionStorage scope).
 * 
 * FIX: Previous version generated a NEW key on every page load, breaking cross-page
 * session persistence. Now the key is exported/imported via SessionStorage.
 */

class SecureStorage {
    constructor() {
        this.key = null;
        this.ready = this.init();
    }

    async init() {
        try {
            // Try to restore a previously saved key (survives page navigation)
            const savedKeyJson = sessionStorage.getItem('__ss_key');
            if (savedKeyJson) {
                const jwk = JSON.parse(savedKeyJson);
                this.key = await window.crypto.subtle.importKey(
                    'jwk',
                    jwk,
                    { name: 'AES-GCM', length: 256 },
                    true,
                    ['encrypt', 'decrypt']
                );
                console.log("🔒 Secure Storage: Restored existing key from session.");
            } else {
                // First visit in this tab — generate a fresh key
                this.key = await window.crypto.subtle.generateKey(
                    { name: 'AES-GCM', length: 256 },
                    true,  // extractable so we can export it
                    ['encrypt', 'decrypt']
                );
                // Persist the key as JWK so it survives page navigation
                const exported = await window.crypto.subtle.exportKey('jwk', this.key);
                sessionStorage.setItem('__ss_key', JSON.stringify(exported));
                console.log("🔒 Secure Storage Initialized (New Ephemeral Key Generated & Persisted)");
            }
        } catch (e) {
            console.error("SecureStorage init failed, falling back to plaintext sessionStorage", e);
            this.key = null;
        }
    }

    async setItem(key, value) {
        await this.ready;

        // Fallback: if crypto init failed, use plain sessionStorage
        if (!this.key) {
            sessionStorage.setItem(key, value);
            return;
        }

        const encoder = new TextEncoder();
        const data = encoder.encode(value);

        // Random IV for every write
        const iv = window.crypto.getRandomValues(new Uint8Array(12));

        const encrypted = await window.crypto.subtle.encrypt(
            { name: 'AES-GCM', iv: iv },
            this.key,
            data
        );

        // Store: IV + Ciphertext as JSON
        const blob = {
            iv: Array.from(iv),
            data: Array.from(new Uint8Array(encrypted))
        };

        sessionStorage.setItem(key, JSON.stringify(blob));
    }

    async getItem(key) {
        await this.ready;
        const raw = sessionStorage.getItem(key);
        if (!raw) return null;

        // Fallback: if crypto init failed, return raw value
        if (!this.key) {
            return raw;
        }

        try {
            const blob = JSON.parse(raw);

            // Guard: if the stored data doesn't look like an encrypted blob,
            // return it as-is (handles migration from plaintext storage)
            if (!blob.iv || !blob.data) {
                return raw;
            }

            const iv = new Uint8Array(blob.iv);
            const data = new Uint8Array(blob.data);

            const decrypted = await window.crypto.subtle.decrypt(
                { name: 'AES-GCM', iv: iv },
                this.key,
                data
            );

            const decoder = new TextDecoder();
            return decoder.decode(decrypted);
        } catch (e) {
            console.error("Decryption Failed (Key Mismatch or Tampering)", e);
            return null;
        }
    }

    clear() {
        sessionStorage.clear();
        // sessionStorage.clear() also removes __ss_key, so next page load gets a fresh key
    }
}

// Initialize Global Instance
window.secureStorage = new SecureStorage();