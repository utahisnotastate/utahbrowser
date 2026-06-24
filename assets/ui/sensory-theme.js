/**
 * sensory-theme.js
 * Kinematic actuator for Fluidic Spatial Topography.
 * Calculates cursor trajectory and maps it to CSS variables for 3D tilt and glow effects.
 */
document.addEventListener("DOMContentLoaded", () => {
    function initFluidCards() {
        const cards = document.querySelectorAll('.fluid-card');

        cards.forEach(card => {
            // Avoid duplicate listeners if script re-runs
            if (card.dataset.fluidInitialized) return;
            card.dataset.fluidInitialized = "true";

            card.addEventListener('mousemove', (e) => {
                const rect = card.getBoundingClientRect();
                
                // Calculate absolute cursor position within the card boundaries
                const x = e.clientX - rect.left;
                const y = e.clientY - rect.top;

                // Map the coordinates to CSS variables for the radial glow
                card.style.setProperty('--mouse-x', `${x}px`);
                card.style.setProperty('--mouse-y', `${y}px`);

                // Calculate rotational tilt vectors
                const centerX = rect.width / 2;
                const centerY = rect.height / 2;
                
                // Adjust the divisor (20) to increase/decrease tilt severity
                const rotateX = ((y - centerY) / 20) * -1;
                const rotateY = (x - centerX) / 20;

                card.style.transform = `perspective(1000px) rotateX(${rotateX}deg) rotateY(${rotateY}deg)`;
            });

            // Reset the geometry when the user's cursor leaves the element
            card.addEventListener('mouseleave', () => {
                card.style.transform = `perspective(1000px) rotateX(0deg) rotateY(0deg)`;
            });
        });
    }

    // Run initial scan
    initFluidCards();

    // Ghost-Link haptic theme — applies palette from theme.json via IPC.
    function applyTheme(d) {
        if (!d || d.event !== 'sensory_theme') return;
        var root = document.documentElement;
        root.setAttribute('data-sensory-mode', d.mode || 'calm');
        if (d.accent) root.style.setProperty('--sensory-accent', d.accent);
        document.body.classList.toggle('sensory-focus', d.mode === 'focus');
        document.body.classList.toggle('sensory-calm', d.mode === 'calm');
    }

    window.addEventListener('utah-ipc', function (ev) {
        applyTheme(ev.detail || {});
    });

    // Re-scan when panels change (since Utah Browser uses a SPA-like layout)
    const observer = new MutationObserver((mutations) => {
        initFluidCards();
    });

    const mainContent = document.querySelector('.utah-main') || document.body;
    observer.observe(mainContent, { childList: true, subtree: true });
});
