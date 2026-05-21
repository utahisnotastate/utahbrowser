/**
 * Ghost-Chrome bookmark tether — drag creates a floating preview chip.
 */
(function () {
  var floatEl = null;

  function ensureFloat() {
    if (floatEl) return floatEl;
    floatEl = document.createElement('div');
    floatEl.className = 'utah-bookmark-float';
    floatEl.setAttribute('aria-hidden', 'true');
    document.body.appendChild(floatEl);
    return floatEl;
  }

  document.addEventListener('dragstart', function (e) {
    var t = e.target;
    if (!t || !t.classList || !t.classList.contains('utah-bookmark')) return;
    var label = t.textContent || 'Bookmark';
    var url = t.title && t.title.split(' — ').pop();
    if (url) {
      t.setAttribute('data-prefetch-url', url);
      e.dataTransfer.setData('text/uri-list', url);
    }
    var f = ensureFloat();
    f.textContent = label;
    f.classList.add('utah-bookmark-float-active');
  });

  document.addEventListener('drag', function (e) {
    if (!floatEl || !floatEl.classList.contains('utah-bookmark-float-active')) return;
    floatEl.style.left = e.clientX + 12 + 'px';
    floatEl.style.top = e.clientY + 12 + 'px';
  });

  document.addEventListener('dragend', function () {
    if (floatEl) floatEl.classList.remove('utah-bookmark-float-active');
  });
})();
