/**
 * Phase 7: Geometric Ad-Ablation
 * Prunes DOM nodes based on geometric heuristics (IAB ad unit sizes, sticky overlays, etc.)
 */
(function() {
    const IAB_SIZES = [
        {w: 728, h: 90},  // Leaderboard
        {w: 300, h: 250}, // Medium Rectangle
        {w: 160, h: 600}, // Wide Skyscraper
        {w: 300, h: 600}, // Half Page
        {w: 970, h: 250}, // Billboard
        {w: 320, h: 50}   // Mobile Leaderboard
    ];

    function isAdGeometry(el) {
        const rect = el.getBoundingClientRect();
        if (rect.width === 0 || rect.height === 0) return false;

        // Check for common IAB sizes
        const matchesIAB = IAB_SIZES.some(size => 
            Math.abs(rect.width - size.w) < 5 && Math.abs(rect.height - size.h) < 5
        );
        if (matchesIAB) return true;

        // Check for sticky/floating parasitic noise
        const style = window.getComputedStyle(el);
        if ((style.position === 'fixed' || style.position === 'sticky') && 
            rect.width > window.innerWidth * 0.8 && 
            rect.height < 150) {
            return true; // Likely a bottom/top sticky banner
        }

        // Check for cross-origin iframes with high z-index
        if (el.tagName === 'IFRAME') {
            try {
                // Accessing contentDocument will throw if cross-origin
                const doc = el.contentDocument; 
            } catch (e) {
                if (parseInt(style.zIndex) > 100) return true;
            }
        }

        return false;
    }

    function ablate() {
        const candidates = document.querySelectorAll('div, iframe, ins, aside');
        candidates.forEach(el => {
            if (isAdGeometry(el)) {
                if (el.style.display !== 'none') {
                    console.log('[UTAH_ABLATION] Pruning geometric noise:', el);
                    el.style.display = 'none';
                    el.setAttribute('data-utah-ablated', 'true');
                    
                    // Report to Sovereign Shield
                    if (window.ipc && window.ipc.postMessage) {
                        window.ipc.postMessage(JSON.stringify({
                            cmd: 'report_shield_block',
                            url: el.src || el.id || 'Geometric Element',
                            category: 'Layout Annoyance'
                        }));
                    }
                }
            }
        });
    }

    // Initial pass
    ablate();

    // Observer for dynamic content
    const observer = new MutationObserver((mutations) => {
        mutations.forEach(m => {
            if (m.addedNodes.length > 0) {
                ablate();
            }
        });
    });

    observer.observe(document.body, { childList: true, subtree: true });

    // Algorithmic Identity Protection (Fingerprint Shuffling)
    // Randomizes canvas fingerprints to prevent tracking.
    const originalGetContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function(type, attributes) {
        const context = originalGetContext.call(this, type, attributes);
        if (type === '2d' && context) {
            const originalGetImageData = context.getImageData;
            context.getImageData = function(x, y, w, h) {
                const imageData = originalGetImageData.call(this, x, y, w, h);
                // Inject random noise into the least significant bits
                for (let i = 0; i < imageData.data.length; i += 4) {
                    imageData.data[i] = imageData.data[i] ^ (Math.random() > 0.5 ? 1 : 0);
                }
                return imageData;
            };
        }
        return context;
    };
})();
