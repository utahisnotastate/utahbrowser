/**
 * Unified compositor — one WebView, chrome + content iframe (no second engine).
 */
(function () {
  function frame() {
    return document.getElementById('utah-content-frame');
  }

  window.utahNavigateContent = function (url) {
    var f = frame();
    if (f && url) f.src = url;
  };

  window.utahContentBack = function () {
    var f = frame();
    try {
      if (f && f.contentWindow) f.contentWindow.history.back();
    } catch (e) {}
  };

  window.utahContentForward = function () {
    var f = frame();
    try {
      if (f && f.contentWindow) f.contentWindow.history.forward();
    } catch (e) {}
  };

  window.utahContentReload = function () {
    var f = frame();
    try {
      if (f && f.contentWindow) f.contentWindow.location.reload();
    } catch (e) {
      if (f && f.src) f.src = f.src;
    }
  };

  document.querySelectorAll('[data-goto-app]').forEach(function (btn) {
    btn.addEventListener('click', function () {
      if (window.utahSend) {
        window.utahSend({ cmd: 'set_shell_mode', mode: 'app' });
      }
    });
  });

  var preloadPool = document.getElementById('utah-prefetch-pool');
  if (!preloadPool) {
    preloadPool = document.createElement('div');
    preloadPool.id = 'utah-prefetch-pool';
    preloadPool.setAttribute('aria-hidden', 'true');
    preloadPool.style.cssText = 'position:absolute;width:0;height:0;overflow:hidden;pointer-events:none;';
    document.body.appendChild(preloadPool);
  }

  window.utahPreloadBuffer = function (bufferUri, originalUrl) {
    if (!bufferUri) return;
    var link = document.createElement('link');
    link.rel = 'prefetch';
    link.href = bufferUri;
    link.setAttribute('data-original-url', originalUrl || '');
    preloadPool.appendChild(link);
  };

  window.utahOnFrameReady = function () {
    if (window.utahSend) window.utahSend({ cmd: 'sync_browser' });
  };
})();
