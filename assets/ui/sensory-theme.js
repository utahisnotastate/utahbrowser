/**
 * Ghost-Link haptic theme — applies palette from theme.json via IPC.
 */
(function () {
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
})();
