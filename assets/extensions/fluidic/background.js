// background.js
// Evaluates on initial browser load and extension installation

chrome.runtime.onInstalled.addListener(() => {
    // Zero-Bug Policy: Establish strict default state.
    // Fluidic Spatial Topography is disabled by default.
    chrome.storage.local.get(['fluidicEnabled'], (result) => {
        if (result.fluidicEnabled === undefined) {
            chrome.storage.local.set({ fluidicEnabled: false }, () => {
                console.log("[UTAH BROWSER] Default Settings Initialized: Fluidic UI is OFF.");
            });
        }
    });
});
