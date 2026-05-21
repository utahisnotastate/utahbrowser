/**
 * Calibration Console — thin IPC transport (view state is CSS :target).
 */
(function () {
  function send(cmd, extra) {
    var payload = Object.assign({ cmd: cmd }, extra || {});
    if (window.utahSend) window.utahSend(payload);
  }

  function el(id) {
    return document.getElementById(id);
  }

  function healthClass(h) {
    if (h === 'healthy') return 'zone-health-healthy';
    if (h === 'degraded') return 'zone-health-degraded';
    if (h === 'critical') return 'zone-health-critical';
    return 'zone-health-unknown';
  }

  function renderTelemetry(t) {
    if (!t) return;
    var ollama = el('tel-ollama');
    var qdrant = el('tel-qdrant');
    var embed = el('tel-embed');
    var points = el('tel-points');
    var chunks = el('tel-chunks');
    var note = el('tel-note');
    var bar = el('tel-bar');
    if (ollama) ollama.textContent = t.ollama_online ? 'ON' : 'OFF';
    if (qdrant) qdrant.textContent = t.qdrant_online ? 'ON' : 'OFF';
    if (embed) embed.textContent = t.embed_latency_ms != null ? t.embed_latency_ms + ' ms' : '—';
    if (points) points.textContent = t.vector_points != null ? String(t.vector_points) : '—';
    if (chunks) chunks.textContent = String(t.chunks_indexed ?? 0);
    if (note) note.textContent = t.gpu_note || '';
    if (bar) {
      var pct = Math.min(100, ((t.embed_latency_ms || 200) / 300) * 100);
      if (t.ollama_online) pct = 100 - pct;
      bar.style.setProperty('--telemetry-pct', pct + '%');
    }
  }

  function renderZones(zones) {
    var list = el('zone-list');
    if (!list) return;
    list.innerHTML = '';
    (zones || []).forEach(function (z) {
      var li = document.createElement('li');
      li.className = 'zone-item';
      li.dataset.zoneId = z.id;

      var info = document.createElement('div');
      info.innerHTML =
        '<span class="zone-health ' +
        healthClass(z.health) +
        '"></span><strong>' +
        (z.label || 'Zone') +
        '</strong><div class="zone-meta">' +
        z.path +
        ' · ' +
        z.readable_files +
        '/' +
        z.total_files +
        ' files · ' +
        z.indexed_chunks +
        ' chunks</div>';
      li.appendChild(info);

      var controls = document.createElement('div');
      controls.className = 'zone-controls';

      var slider = document.createElement('input');
      slider.type = 'range';
      slider.min = '0.1';
      slider.max = '5';
      slider.step = '0.1';
      slider.value = String(z.weight || 1);
      slider.className = 'zone-weight';
      slider.title = 'Oracle priority weight';
      slider.addEventListener('change', function () {
        send('set_zone_weight', { zone_id: z.id, weight: parseFloat(slider.value) });
      });
      controls.appendChild(slider);

      var ingest = document.createElement('button');
      ingest.type = 'button';
      ingest.className = 'utah-btn';
      ingest.textContent = 'Index';
      ingest.addEventListener('click', function () {
        send('ingest_zone', { zone_id: z.id });
      });
      controls.appendChild(ingest);

      var sanitize = document.createElement('button');
      sanitize.type = 'button';
      sanitize.className = 'utah-btn';
      sanitize.textContent = 'Sanitize';
      sanitize.addEventListener('click', function () {
        send('sanitize_zone', { zone_id: z.id });
      });
      controls.appendChild(sanitize);

      var remove = document.createElement('button');
      remove.type = 'button';
      remove.className = 'utah-btn';
      remove.textContent = '×';
      remove.title = 'Unbind zone';
      remove.addEventListener('click', function () {
        send('remove_zone', { zone_id: z.id });
      });
      controls.appendChild(remove);

      var dm = document.createElement('label');
      dm.innerHTML =
        '<input type="checkbox" ' +
        (z.direct_map ? 'checked' : '') +
        ' /> Direct-map';
      dm.querySelector('input').addEventListener('change', function (e) {
        send('set_zone_direct_map', { zone_id: z.id, direct_map: e.target.checked });
      });
      controls.appendChild(dm);

      li.appendChild(controls);
      list.appendChild(li);
    });
  }

  el('bind-zone-btn') &&
    el('bind-zone-btn').addEventListener('click', function () {
      send('bind_knowledge_zone');
    });

  el('refresh-console-btn') &&
    el('refresh-console-btn').addEventListener('click', function () {
      send('get_calibration_console');
    });

  el('global-direct-map') &&
    el('global-direct-map').addEventListener('change', function (e) {
      send('set_direct_mapping_global', { enabled: e.target.checked });
    });

  window.addEventListener('utah-ipc', function (ev) {
    var d = ev.detail || {};
    if (d.event === 'calibration_console') {
      renderTelemetry(d.telemetry);
      renderZones(d.zones);
      var gdm = el('global-direct-map');
      if (gdm) gdm.checked = !!d.direct_mapping_global;
    }
    if (d.event === 'zone_bound') {
      send('get_calibration_console');
    }
  });

  document.querySelectorAll('a[href="#settings-console"]').forEach(function (a) {
    a.addEventListener('click', function () {
      send('get_calibration_console');
    });
  });
})();
