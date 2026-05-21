/**
 * Intent-resolution buffer — cursor trajectory toward links triggers prefetch hints.
 * Works on chrome DOM (bookmarks, tabs, url bar). Cross-origin iframe content is excluded.
 */
(function () {
  var SAMPLE_MAX = 10;
  var MIN_SAMPLES = 4;
  var COOLDOWN_MS = 400;
  var lastHint = 0;
  var samples = [];

  function now() {
    return Date.now();
  }

  function pushSample(x, y) {
    samples.push({ x: x, y: y, t: now() });
    if (samples.length > SAMPLE_MAX) samples.shift();
  }

  function velocity() {
    if (samples.length < 2) return { vx: 0, vy: 0 };
    var a = samples[samples.length - 2];
    var b = samples[samples.length - 1];
    var dt = Math.max(1, b.t - a.t);
    return { vx: (b.x - a.x) / dt, vy: (b.y - a.y) / dt };
  }

  function predictPoint() {
    if (!samples.length) return null;
    var last = samples[samples.length - 1];
    var v = velocity();
    return { x: last.x + v.vx * 120, y: last.y + v.vy * 120 };
  }

  function nearestLink(x, y) {
    var el = document.elementFromPoint(x, y);
    if (!el) return null;
    var node = el.closest('a[href], .utah-bookmark, [data-prefetch-url]');
    if (!node) return null;
    var url = node.getAttribute('data-prefetch-url') || node.getAttribute('href');
    if (!url || url.indexOf('javascript:') === 0) return null;
    return url;
  }

  function maybeHint(url) {
    if (!url || !window.utahSend) return;
    var t = now();
    if (t - lastHint < COOLDOWN_MS) return;
    lastHint = t;
    window.utahSend({ cmd: 'prefetch_hint', url: url });
  }

  document.addEventListener(
    'mousemove',
    function (e) {
      pushSample(e.clientX, e.clientY);
      if (samples.length < MIN_SAMPLES) return;
      var p = predictPoint();
      if (!p) return;
      var url = nearestLink(p.x, p.y);
      if (url) maybeHint(url);
    },
    { passive: true }
  );

})();
