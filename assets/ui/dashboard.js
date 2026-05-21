/**
 * Utah Browser shell — view routing + native layout mode (web vs app).
 */
(function () {
  function send(cmd, extra) {
    var payload = Object.assign({ cmd: cmd }, extra || {});
    if (window.utahSend) window.utahSend(payload);
  }

  function el(id) {
    return document.getElementById(id);
  }

  function currentView() {
    var views = ['dashboard', 'web', 'truth', 'merge', 'welcome'];
    for (var i = 0; i < views.length; i++) {
      var input = el('view-' + views[i]);
      if (input && input.checked) return views[i];
    }
    return 'web';
  }

  function setShellMode(view) {
    var isWeb = view === 'web';
    document.body.classList.toggle('utah-mode-web', isWeb);
    document.body.classList.toggle('utah-mode-app', !isWeb);
    send('set_shell_mode', { mode: isWeb ? 'web' : 'app' });
    var drawer = el('sidebar-drawer');
    if (drawer && isWeb) drawer.checked = false;
  }

  function gotoView(name) {
    var input = el('view-' + name);
    if (input) input.checked = true;
    setShellMode(name);
  }

  window.utahSetShellMode = function (mode) {
    var isWeb = mode === 'web';
    document.body.classList.toggle('utah-mode-web', isWeb);
    document.body.classList.toggle('utah-mode-app', !isWeb);
  };

  window.utahOnShellMode = window.utahSetShellMode;
  window.utahGotoView = gotoView;

  document.querySelectorAll('[data-goto]').forEach(function (node) {
    node.addEventListener('click', function () {
      gotoView(node.getAttribute('data-goto'));
    });
  });

  document.querySelectorAll('input[name="utah-view"]').forEach(function (radio) {
    radio.addEventListener('change', function () {
      setShellMode(currentView());
    });
  });

  var menuBtn = el('chrome-menu-btn');
  if (menuBtn) {
    menuBtn.addEventListener('click', function () {
      var v = currentView();
      if (v === 'web') {
        gotoView('dashboard');
      } else {
        var d = el('sidebar-drawer');
        if (d) d.checked = !d.checked;
      }
    });
  }

  var spyInput = el('spyglass-input');
  if (spyInput) {
    spyInput.addEventListener('keydown', function (e) {
      if (e.key !== 'Enter') return;
      e.preventDefault();
      var v = spyInput.value.trim();
      if (!v) return;
      var close = el('spyglass-open');
      if (close) close.checked = false;
      if (/^[\w-]+:\/\//i.test(v) || v.indexOf('.') >= 0) {
        gotoView('web');
        send('navigate', { url: v });
      } else {
        send('search_bookmarks', { query: v });
        gotoView('dashboard');
      }
      spyInput.value = '';
    });
  }

  document.addEventListener('keydown', function (e) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      var s = el('spyglass-open');
      if (s) {
        s.checked = true;
        if (spyInput) spyInput.focus();
      }
    }
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'l') {
      e.preventDefault();
      gotoView('web');
      var url = el('url');
      if (url) url.focus();
    }
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 't') {
      e.preventDefault();
      gotoView('truth');
    }
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'n') {
      e.preventDefault();
      send('new_tab', {});
      gotoView('web');
    }
  });

  function tickClock() {
    var c = el('clock');
    if (c) {
      c.textContent = new Date().toLocaleTimeString([], {
        hour: 'numeric',
        minute: '2-digit'
      });
    }
  }
  tickClock();
  setInterval(tickClock, 30000);

  function paintTruthGraph() {
    var svg = document.querySelector('.truth-graph-svg');
    if (!svg) return;
    var edges = svg.querySelector('.graph-edges');
    var nodes = svg.querySelector('.graph-nodes');
    if (!edges || !nodes) return;
    var labels = ['Sources', 'Corpus', 'Claim', 'Notes', 'Verdict'];
    var cx = 200;
    var cy = 140;
    var r = 95;
    edges.innerHTML = '';
    nodes.innerHTML = '';
    labels.forEach(function (label, i) {
      var a = (i / labels.length) * Math.PI * 2 - Math.PI / 2;
      var x = cx + Math.cos(a) * r;
      var y = cy + Math.sin(a) * r;
      var line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
      line.setAttribute('x1', String(cx));
      line.setAttribute('y1', String(cy));
      line.setAttribute('x2', String(x));
      line.setAttribute('y2', String(y));
      line.setAttribute('stroke', 'rgba(212,175,55,0.35)');
      edges.appendChild(line);
      var circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
      circle.setAttribute('cx', String(x));
      circle.setAttribute('cy', String(y));
      circle.setAttribute('r', '12');
      circle.setAttribute('fill', 'rgba(18,28,44,0.95)');
      circle.setAttribute('stroke', 'rgba(212,175,55,0.5)');
      nodes.appendChild(circle);
    });
  }
  paintTruthGraph();

  function setStatus(ollama, qdrant, knowledge, chunks) {
    function setText(id, t) {
      var n = el(id);
      if (n) n.textContent = t;
    }
    setText('st-ollama', ollama ? 'online' : 'offline');
    setText('st-qdrant', qdrant ? 'online' : 'offline');
    setText('st-knowledge', knowledge || '—');
    setText('st-chunks', String(chunks ?? 0));
    setText('w-stack', ollama && qdrant ? 'Ready' : 'Starting…');
    setText('w-knowledge', knowledge || '—');
    var sysO = el('sys-ollama');
    var sysQ = el('sys-qdrant');
    if (sysO) sysO.textContent = 'Ollama — ' + (ollama ? 'Ready' : 'Offline');
    if (sysQ) sysQ.textContent = 'Qdrant — ' + (qdrant ? 'Ready' : 'Offline');
  }

  window.addEventListener('utah-ipc', function (ev) {
    var d = ev.detail || {};
    if (d.event === 'status') {
      setStatus(d.ollama, d.qdrant, d.knowledge_path, d.chunks_indexed);
    }
    if (d.event === 'verify_result') {
      var result = el('truth-result');
      if (result) {
        result.className =
          'utah-truth-result ' + (d.flagged ? 'utah-flagged' : 'utah-ok');
        result.textContent = d.summary || '';
      }
    }
    if (d.event === 'navigation_changed') {
      var mergeWeb = el('merge-web-snippet');
      if (mergeWeb) mergeWeb.textContent = (d.title || d.url || '').slice(0, 80);
    }
  });

  window.addEventListener('load', function () {
    gotoView('web');
  });
})();
