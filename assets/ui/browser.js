/**
 * Utah Browser chrome — thin IPC transport for spatial graph UI.
 * View state is CSS-driven (:target / :has); this file does not own app state.
 */
(function () {
  var homeUrl = window.__utahHomeUrl || 'https://www.google.com';
  var tabs = [];
  var activeTabId = 0;

  function send(cmd, extra) {
    var payload = Object.assign({ cmd: cmd }, extra || {});
    if (window.utahSend) window.utahSend(payload);
  }

  function el(id) {
    return document.getElementById(id);
  }

  function renderTabs() {
    var strip = el('tab-strip');
    if (!strip) return;
    strip.innerHTML = '';
    tabs.forEach(function (tab) {
      var btn = document.createElement('button');
      btn.type = 'button';
      var cls = 'utah-tab' + (tab.id === activeTabId ? ' utah-tab-active' : '');
      if (tab.suspended) cls += ' utah-tab-suspended';
      btn.className = cls;
      btn.setAttribute('role', 'tab');
      btn.setAttribute('aria-selected', tab.id === activeTabId ? 'true' : 'false');
      btn.dataset.tabId = String(tab.id);

      var title = document.createElement('span');
      title.className = 'utah-tab-title';
      title.textContent = tab.title || tab.url || 'Memory Brick';
      btn.appendChild(title);

      if (tabs.length > 1) {
        var close = document.createElement('span');
        close.className = 'utah-tab-close';
        close.setAttribute('aria-label', 'Close tab');
        close.textContent = '×';
        close.addEventListener('click', function (e) {
          e.stopPropagation();
          send('close_tab', { tab_id: tab.id });
        });
        btn.appendChild(close);
      }

      btn.addEventListener('click', function () {
        if (tab.id !== activeTabId) send('switch_tab', { tab_id: tab.id });
      });
      strip.appendChild(btn);
    });
  }

  function renderBookmarks(bookmarks) {
    var bar = el('bookmark-bar');
    var merge = el('merge-bookmarks');
    if (!bar && !merge) return;
    if (bar) bar.innerHTML = '';
    if (merge) merge.innerHTML = '';
    (bookmarks || []).forEach(function (bm) {
      var a = document.createElement('button');
      a.type = 'button';
      a.className = 'utah-bookmark';
      a.draggable = true;
      a.setAttribute('data-prefetch-url', bm.url);
      a.title = (bm.intention || bm.url) + ' — ' + bm.url;
      a.textContent = bm.title || bm.url;
      a.addEventListener('click', function () {
        send('navigate', { url: bm.url });
      });
      a.addEventListener('mouseenter', function () {
        send('prefetch_hint', { url: bm.url });
      });
      a.addEventListener('contextmenu', function (e) {
        e.preventDefault();
        send('remove_bookmark', { bookmark_id: bm.id });
      });
      if (bar) bar.appendChild(a);
      if (merge) {
        var li = document.createElement('li');
        var btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'chip-btn';
        btn.textContent = bm.title || bm.url;
        btn.addEventListener('click', function () {
          send('navigate', { url: bm.url });
          if (window.utahGotoView) window.utahGotoView('web');
        });
        li.appendChild(btn);
        merge.appendChild(li);
      }
    });
  }

  function renderSpatialMap(hits) {
    var map = el('spatial-map');
    if (!map) return;
    map.innerHTML = '';
    var cx = 50;
    var cy = 50;
    (hits || []).forEach(function (h, i) {
      var angle = (i / Math.max(hits.length, 1)) * Math.PI * 2;
      var r = (h.proximity != null ? h.proximity : 0.3) * 42;
      var node = document.createElement('button');
      node.type = 'button';
      node.className = 'utah-spatial-node';
      node.style.left = cx + Math.cos(angle) * r + '%';
      node.style.top = cy + Math.sin(angle) * r + '%';
      node.textContent = h.title || h.url;
      node.title = h.intention || h.url;
      node.addEventListener('click', function () {
        send('navigate', { url: h.url });
      });
      map.appendChild(node);
    });
  }

  function renderExtensions(list) {
    var ul = el('extension-list');
    if (!ul) return;
    ul.innerHTML = '';
    (list || []).forEach(function (ext) {
      var li = document.createElement('li');
      li.textContent = ext.name + ' [' + ext.trigger + '] — ' + (ext.intent || '');
      var run = document.createElement('button');
      run.type = 'button';
      run.className = 'utah-btn';
      run.textContent = 'Run';
      run.addEventListener('click', function () {
        send('run_extension', { name: ext.name, action: 'click' });
      });
      li.appendChild(document.createTextNode(' '));
      li.appendChild(run);
      ul.appendChild(li);
    });
  }

  function syncUrlBar(url) {
    var input = el('url');
    if (input && document.activeElement !== input) input.value = url || '';
  }

  el('tab-new') && el('tab-new').addEventListener('click', function () {
    send('new_tab', {});
  });

  el('nav-back') && el('nav-back').addEventListener('click', function () {
    send('go_back');
  });
  el('nav-forward') && el('nav-forward').addEventListener('click', function () {
    send('go_forward');
  });
  el('nav-reload') && el('nav-reload').addEventListener('click', function () {
    send('reload');
  });
  el('nav-go-home') && el('nav-go-home').addEventListener('click', function () {
    send('go_home');
  });

  el('bookmark-add') && el('bookmark-add').addEventListener('click', function () {
    var q = el('bookmark-query');
    send('add_bookmark', { intention: q && q.value ? q.value : undefined });
  });

  var searchForm = el('bookmark-search-form');
  if (searchForm) {
    searchForm.addEventListener('submit', function (e) {
      e.preventDefault();
      var q = el('bookmark-query');
      if (q && q.value.trim()) send('search_bookmarks', { query: q.value.trim() });
    });
  }

  var urlForm = el('url-form');
  if (urlForm) {
    urlForm.addEventListener('submit', function (e) {
      e.preventDefault();
      send('navigate', { url: el('url').value });
    });
  }

  var vibeForm = el('vibe-extension-form');
  if (vibeForm) {
    vibeForm.addEventListener('submit', function (e) {
      e.preventDefault();
      send('vibe_extension', {
        name: el('ext-name').value,
        intent: el('ext-intent').value,
        trigger: 'DOM_LOADED'
      });
      vibeForm.reset();
    });
  }

  window.addEventListener('utah-ipc', function (ev) {
    var d = ev.detail || {};
    if (d.event === 'ghost_link_status') {
      var line = el('ghost-status-line');
      if (line) line.textContent = d.message || 'Ghost-Link';
      return;
    }
    if (d.event === 'tabs_updated') {
      tabs = d.tabs || [];
      activeTabId = d.active_id || 0;
      if (d.home_url) homeUrl = d.home_url;
      renderTabs();
    }
    if (d.event === 'navigation_changed') syncUrlBar(d.url);
    if (d.event === 'bookmarks_updated') renderBookmarks(d.bookmarks);
    if (d.event === 'spatial_bookmarks') {
      renderSpatialMap(d.hits);
      renderBookmarks(d.hits);
    }
    if (d.event === 'extensions_updated') renderExtensions(d.extensions);
    if (d.event === 'prefetch_buffered' && window.utahPreloadBuffer) {
      window.utahPreloadBuffer(d.buffer_uri, d.url);
    }
  });

  el('ghost-status-btn') && el('ghost-status-btn').addEventListener('click', function () {
    send('get_ghost_link_status');
  });

  el('urm-status-btn') &&
    el('urm-status-btn').addEventListener('click', function () {
      send('get_urm_status');
    });

  window.addEventListener('utah-ipc', function (ev) {
    var d = ev.detail || {};
    if (d.event === 'urm_status') {
      var line = document.getElementById('urm-status-line');
      if (!line) {
        line = document.createElement('p');
        line.id = 'urm-status-line';
        line.className = 'utah-muted urm-status-line';
        var tools = document.getElementById('tools-panel');
        if (tools && tools.querySelector('.utah-card')) {
          tools.querySelector('.utah-card').appendChild(line);
        }
      }
      if (line) line.textContent = d.message || 'URM';
    }
  });

  send('sync_browser');
  send('list_extensions');
  send('get_ghost_link_status');
})();
