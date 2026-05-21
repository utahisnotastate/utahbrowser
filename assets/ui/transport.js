/**
 * Utah Browser transport layer — IPC bridge only (not UI logic).
 */
(function () {
  function send(cmd, extra) {
    var payload = Object.assign({ cmd: cmd }, extra || {});
    if (window.utahSend) window.utahSend(payload);
  }

  function ensureThen(cmd, extra) {
    send('ensure_services');
    var handler = function (ev) {
      var d = ev.detail || {};
      if (d.event === 'status' || d.event === 'error') {
        window.removeEventListener('utah-ipc', handler);
        if (d.event === 'error') return;
        send(cmd, extra);
      }
    };
    window.addEventListener('utah-ipc', handler);
  }

  document.querySelectorAll('[data-utah-cmd]').forEach(function (btn) {
    var cmd = btn.getAttribute('data-utah-cmd');
    btn.addEventListener('click', function () {
      if (cmd === 'get_status') {
        send('get_status');
      } else {
        ensureThen(cmd);
      }
    });
  });

  var form = document.getElementById('verify-form');
  if (form) {
    form.addEventListener('submit', function (e) {
      e.preventDefault();
      var text = document.getElementById('verify-text').value;
      ensureThen('verify_text', { text: text });
    });
  }

  var nav = document.querySelector('.utah-nav-form');
  if (nav) {
    nav.addEventListener('submit', function (e) {
      e.preventDefault();
      var url = document.getElementById('url').value;
      var frame = document.getElementById('utah-page');
      if (frame) frame.src = url;
      send('navigate', { url: url });
    });
  }

  window.addEventListener('utah-ipc', function (ev) {
    var d = ev.detail || {};
    if (d.event === 'status') {
      set('st-ollama', d.ollama ? 'online' : 'offline');
      set('st-qdrant', d.qdrant ? 'online' : 'offline');
      set('st-knowledge', d.knowledge_path || '—');
      set('st-chunks', String(d.chunks_indexed ?? 0));
    }
    if (d.event === 'verify_result') {
      var el = document.getElementById('truth-result');
      var hud = document.getElementById('truth-hud-toggle');
      if (el) {
        el.className = 'utah-truth-result ' + (d.flagged ? 'utah-flagged' : 'utah-ok');
        el.textContent = d.summary || '';
      }
      if (hud && d.flagged) hud.checked = true;
    }
    if (d.event === 'ingest_progress') {
      set('st-chunks', d.message || '');
    }
    if (d.event === 'error' && d.message) {
      var r = document.getElementById('truth-result');
      if (r) {
        r.className = 'utah-truth-result utah-flagged';
        r.textContent = d.message;
      }
      set('st-qdrant', 'offline');
    }
  });

  function set(id, text) {
    var n = document.getElementById(id);
    if (n) n.textContent = text;
  }

  ensureThen('get_status');
})();
