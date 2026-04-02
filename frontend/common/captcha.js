// C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\frontend\common\captcha.js

class CaptchaHandler {
    constructor(containerId, inputId, tokenInputId) {
        // We only store IDs here, not the elements themselves
        // This fixes the "White Blank" bug by finding elements only when needed
        this.containerId = containerId || 'captcha-image';
        this.inputId = inputId || 'captcha-answer';
        this.tokenInputId = tokenInputId || 'captcha-id';
    }

    async refresh() {
        // 1. Find Elements (Lazy Load)
        const container = document.getElementById(this.containerId);
        const input = document.getElementById(this.inputId);
        const tokenInput = document.getElementById(this.tokenInputId);

        if(!container) {
            console.warn("CAPTCHA: Container element not found yet.");
            return;
        }

        // 2. Show Loading State
        container.innerHTML = '<div class="text-slate-500 text-xs animate-pulse font-mono tracking-widest">CONNECTING TO CORE...</div>';
        if(input) input.value = '';
        
        // 3. Detect Correct API URL (dynamic to support mobile connections)
        const baseUrl = typeof API_BASE !== 'undefined'
            ? API_BASE
            : `http://${window.location.hostname}:3001/api`;

        try {
            // FIX: Updated route from '/captcha' to '/auth/captcha' to match Rust Router
            const response = await fetch(`${baseUrl}/auth/captcha`, { method: 'POST' });
            
            if(!response.ok) throw new Error("Captcha Gen Failed");

            const data = await response.json();

            // 4. Inject SVG
            container.innerHTML = data.svg;
            
            if(tokenInput) tokenInput.value = data.token;

            // 5. Update Instructions
            const colorMap = { 
                'RED': 'text-red-500', 
                'GREEN': 'text-green-500', 
                'BLUE': 'text-blue-500', 
                'PURPLE': 'text-purple-500', 
                'ORANGE': 'text-orange-500' 
            };
            
            const colorClass = colorMap[data.target_color] || 'text-white';
            
            // Check for instruction label, create if missing
            let instr = document.getElementById('captcha-instruction');
            if (!instr) {
                instr = document.createElement('div');
                instr.id = 'captcha-instruction';
                instr.className = "text-[10px] text-slate-500 mb-1 font-mono text-center tracking-wider";
                container.parentNode.insertBefore(instr, container);
            }
            
            instr.innerHTML = `TYPE CHARACTERS FLASHING <span class="font-bold ${colorClass}">${data.target_color}</span>`;

        } catch (error) {
            console.error("Captcha Load Error:", error);
            container.innerHTML = '<div class="text-red-500 text-xs border border-red-500 p-1">CONNECTION LOST</div>';
        }
    }
}

// Global Singleton
window.captchaHandler = new CaptchaHandler();
