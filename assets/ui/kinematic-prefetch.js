/**
 * Phase 8: Stochastic Kinematic Prefetching
 * Predicts user intent by analyzing mouse cursor trajectory toward anchor tags.
 */
(function() {
    let lastX = 0;
    let lastY = 0;
    let lastTime = Date.now();
    let velocityX = 0;
    let velocityY = 0;

    function getPrediction(x, y, vx, vy) {
        // Simple linear prediction for the next 400ms
        return {
            x: x + vx * 400,
            y: y + vy * 400
        };
    }

    function checkCollision(px, py) {
        const target = document.elementFromPoint(px, py);
        if (target && target.tagName === 'A' && target.href) {
            return target.href;
        }
        // Also check if nearest parent is an anchor
        const anchor = target?.closest('a');
        if (anchor && anchor.href) return anchor.href;
        return null;
    }

    document.addEventListener('mousemove', (e) => {
        const now = Date.now();
        const dt = now - lastTime;
        if (dt > 0) {
            velocityX = (e.clientX - lastX) / dt;
            velocityY = (e.clientY - lastY) / dt;
        }
        
        lastX = e.clientX;
        lastY = e.clientY;
        lastTime = now;

        // Only predict if moving with significant velocity
        if (Math.abs(velocityX) > 0.1 || Math.abs(velocityY) > 0.1) {
            const pred = getPrediction(e.clientX, e.clientY, velocityX, velocityY);
            const predictedUrl = checkCollision(pred.x, pred.y);
            
            if (predictedUrl && !predictedUrl.startsWith('javascript:') && !predictedUrl.startsWith('#')) {
                if (window.utahSend) {
                    // Send prefetch hint to Rust backend
                    window.utahSend({
                        cmd: 'prefetch_hint',
                        url: predictedUrl,
                        metadata: { source: 'kinematic_prediction', confidence: 0.85 }
                    });
                }
            }
        }
    }, { passive: true });
})();
