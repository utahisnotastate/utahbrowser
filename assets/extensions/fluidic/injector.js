// injector.js
(function() {
    // Asynchronous check against the local database
    chrome.storage.local.get(['fluidicEnabled'], function(result) {
        
        // Edge-Case Handling: If the database fails or is empty, assume false.
        if (!result.fluidicEnabled) {
            console.log("[UTAH BROWSER] Fluidic Override disabled via Settings.");
            return; // Abort execution
        }

        console.log("[UTAH BROWSER] Fluidic Override Enabled. Mutating DOM.");

        // Heuristic identification of standard web "containers"
        const targetSelectors = [
            'article', 'section', 'aside', 
            '[class*="card"]', '[class*="box"]', '[class*="panel"]', '[class*="container"]'
        ];

        function applyFluidic(el) {
            if (el.dataset.utahFluidified) return;
            
            const rect = el.getBoundingClientRect();
            // Skip elements that are too small or too large to be a functional card
            if (rect.width < 150 || rect.height < 100 || rect.width > 1200) return;

            el.dataset.utahFluidified = "true";
            el.classList.add('utah-fluid-override');

            // Wrap the existing content to push it forward on the Z-axis
            const wrapper = document.createElement('div');
            wrapper.classList.add('utah-content-lift');
            while (el.firstChild) {
                wrapper.appendChild(el.firstChild);
            }
            
            // Inject the tracking glow layer
            const glow = document.createElement('div');
            glow.classList.add('utah-card-glow');
            
            el.appendChild(glow);
            el.appendChild(wrapper);

            // Attach Kinematic Math
            el.addEventListener('mousemove', (e) => {
                const elRect = el.getBoundingClientRect();
                const x = e.clientX - elRect.left;
                const y = e.clientY - elRect.top;

                el.style.setProperty('--mouse-x', `${x}px`);
                el.style.setProperty('--mouse-y', `${y}px`);

                const centerX = elRect.width / 2;
                const centerY = elRect.height / 2;
                const rotateX = ((y - centerY) / 30) * -1;
                const rotateY = (x - centerX) / 30;

                el.style.transform = `perspective(1000px) rotateX(${rotateX}deg) rotateY(${rotateY}deg)`;
            });

            el.addEventListener('mouseleave', () => {
                el.style.transform = `perspective(1000px) rotateX(0deg) rotateY(0deg)`;
            });
        }

        function scan() {
            const elements = document.querySelectorAll(targetSelectors.join(', '));
            elements.forEach(applyFluidic);
        }

        // Initial scan
        scan();

        // Continuous monitoring for dynamic content (AJAX/SPA)
        const observer = new MutationObserver((mutations) => {
            scan();
        });
        observer.observe(document.body, { childList: true, subtree: true });
    });
})();
