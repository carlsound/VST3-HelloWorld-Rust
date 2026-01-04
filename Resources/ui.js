
(function() {
  const log = (msg) => { const pre = document.getElementById('log'); if (!pre) return; pre.textContent = String(msg) + "\n" + pre.textContent; };
  function sendToHost(obj) { const s = JSON.stringify(obj); if (window.chrome && window.chrome.webview) { window.chrome.webview.postMessage(s); } else if (window.ipc && typeof window.ipc.postMessage === 'function') { window.ipc.postMessage(s); } else { window.postMessage(obj, '*'); } }
  const ORIENTATION = 'vertical';
  const PARAM_ID = 1; const MIN_DB = -60.0; const MAX_DB = 12.0; const A_MIN = Math.pow(10, MIN_DB/20); const A_MAX = Math.pow(10, MAX_DB/20);
  function uiToDb(s) { const a = A_MIN * Math.pow(A_MAX / A_MIN, Number(s)); return 20 * Math.log10(a); }
  function dbToUi(db) { const a = Math.pow(10, Number(db) / 20); return Math.log10(a / A_MIN) / Math.log10(A_MAX / A_MIN); }
  function normToDb(norm) { return MIN_DB + Number(norm) * (MAX_DB - MIN_DB); }
  function dbToNorm(db)   { return (Number(db) - MIN_DB) / (MAX_DB - MIN_DB); }
  const gain = document.getElementById('gain'); if (!gain) { console.warn("Gain slider element #gain not found."); return; }
  gain.addEventListener('pointerdown', () => { sendToHost({ kind: 'begin', param_id: PARAM_ID }); });
  gain.addEventListener('input', () => { const s = Number(gain.value); const db = uiToDb(s); const norm = dbToNorm(db); sendToHost({ kind: 'perform', param_id: PARAM_ID, value: norm }); log(`UI perform -> db=${db.toFixed(2)} norm=${norm.toFixed(3)} s=${s.toFixed(3)}`); });
  const endGesture = () => sendToHost({ kind: 'end', param_id: PARAM_ID }); gain.addEventListener('pointerup', endGesture); gain.addEventListener('pointercancel', endGesture); gain.addEventListener('mouseleave', endGesture);
  window.setGainSlider = function(norm) { const db = normToDb(norm); const s = dbToUi(db); gain.value = String(s); log(`Host setGainSlider(norm=${norm.toFixed(3)} → db=${db.toFixed(2)} → s=${s.toFixed(3)})`); };

  const canvas = document.getElementById('meterCanvas'); const ctx = canvas.getContext('2d');
  function resizeCanvasForDPR() { const dpr = window.devicePixelRatio || 1; const cssW = canvas.clientWidth; const cssH = canvas.clientHeight; canvas.width = Math.floor(cssW * dpr); canvas.height = Math.floor(cssH * dpr); ctx.setTransform(dpr, 0, 0, dpr, 0, 0); }
  resizeCanvasForDPR(); window.addEventListener('resize', resizeCanvasForDPR);

  const MARGIN = { left: 16, right: 16, top: 16, bottom: 16 }; const AXIS_W = 32; const COL_W = 120; const COL_G = 24;
  const GRID_MAJOR_DB = [0, -12, -24, -36, -48, -60]; const GRID_MINOR_DB = [-6, -18, -30, -42, -54];
  const meters = { peakL: -Infinity, peakR: -Infinity, rmsL: -Infinity, rmsR: -Infinity, holdL: -Infinity, holdR: -Infinity, holdTimeL: 0, holdTimeR: 0, clipL: false, clipR: false, clipTimeL: 0, clipTimeR: 0 };
  let tagHits = [];
  function dbToFrac(db) { if (!isFinite(db)) return 0; const f = (db - MIN_DB) / (0 - MIN_DB); return Math.max(0, Math.min(1, f)); }
  function fmtDb(db) { if (!isFinite(db)) return '-∞ dB'; return `${db.toFixed(1)} dB`; }
  function amplitude(db) { return Math.pow(10, db / 20); }
  function color(varName) { return getComputedStyle(document.documentElement).getPropertyValue(varName).trim(); }
  function gradientStopsForDb(db) { const f = dbToFrac(db); const hueStart = Math.max(0, 160 - 160 * f); const hueEnd = Math.max(0, 120 - 120 * f); const start = `hsl(${hueStart}deg, 65%, 22%)`; const end = `hsl(${hueEnd}deg, 85%, 52%)`; return { start, end }; }
  function drawTag(px, py, text, dbValue, channel, type) { ctx.font = '11px system-ui'; const padding = 6, r = 6; const tw = Math.ceil(ctx.measureText(text).width), th = 16, w = tw + padding * 2, h = th; const { start, end } = gradientStopsForDb(dbValue); const grad = ctx.createLinearGradient(px, py, px, py + h); grad.addColorStop(0.0, start); grad.addColorStop(1.0, end); ctx.fillStyle = grad; ctx.beginPath(); ctx.moveTo(px + r, py); ctx.lineTo(px + w - r, py); ctx.quadraticCurveTo(px + w, py, px + w, py + r); ctx.lineTo(px + w, py + h - r); ctx.quadraticCurveTo(px + w, py + h, px + w - r, py + h); ctx.lineTo(px + r, py + h); ctx.quadraticCurveTo(px, py + h, px, py + h - r); ctx.lineTo(px, py + r); ctx.quadraticCurveTo(px, py, px + r, py); ctx.closePath(); ctx.fill(); ctx.strokeStyle = 'rgba(255,255,255,0.12)'; ctx.lineWidth = 1; ctx.stroke(); ctx.fillStyle = '#ffffff'; ctx.textAlign = 'left'; ctx.textBaseline = 'middle'; ctx.fillText(text, px + padding, py + h / 2); tagHits.push({ x: px, y: py, w, h, labelText: text, channel, type, db: dbValue, amp: amplitude(dbValue) }); }

  function drawLeftAxis(axX, y, h) { ctx.strokeStyle = color('--grid'); ctx.fillStyle = color('--tick'); ctx.lineWidth = 1; ctx.font = '12px system-ui'; ctx.textAlign = 'left'; ctx.textBaseline = 'middle'; GRID_MAJOR_DB.forEach(db => { const fy = y + (1 - dbToFrac(db)) * h; ctx.beginPath(); ctx.moveTo(axX, fy); ctx.lineTo(axX + AXIS_W + COL_W * 2 + COL_G, fy); ctx.stroke(); ctx.fillText(`${db}`, axX + 2, fy); }); GRID_MINOR_DB.forEach(db => { const fy = y + (1 - dbToFrac(db)) * h; ctx.beginPath(); ctx.moveTo(axX + 8, fy); ctx.lineTo(axX + AXIS_W - 4, fy); ctx.stroke(); }); ctx.fillStyle = color('--tick'); ctx.font = '12px system-ui'; ctx.textAlign = 'center'; ctx.textBaseline = 'top'; ctx.fillText('dBFS', axX + AXIS_W / 2, y + h + 4); }

  function drawChannelVertical(x, y, w, h, peakDb, rmsDb, holdDb, clipFlag, label) { ctx.fillStyle = color('--panel'); ctx.fillRect(x, y, w, h); const rmsH = Math.round(dbToFrac(rmsDb) * h); ctx.fillStyle = color('--rms'); ctx.fillRect(x + 8, y + (h - rmsH), w - 16, rmsH); const peakH = Math.round(dbToFrac(peakDb) * h); ctx.fillStyle = color('--peak'); ctx.fillRect(x + 12, y + (h - peakH), w - 24, peakH); const holdY = y + (1 - dbToFrac(holdDb)) * h; ctx.strokeStyle = color('--hold'); ctx.lineWidth = 2; ctx.beginPath(); ctx.moveTo(x + 6, holdY); ctx.lineTo(x + w - 6, holdY); ctx.stroke(); if (clipFlag) { ctx.fillStyle = color('--clip'); ctx.fillRect(x + w - 12, y + 4, 8, 8); } ctx.fillStyle = color('--tick'); ctx.font = '12px system-ui'; ctx.textAlign = 'center'; ctx.textBaseline = 'top'; ctx.fillText(label, x + w / 2, y + h + 4); const tagX = x + w + 8; const peakY = y + (h - peakH) - 10; const rmsY  = y + (h - rmsH) + 10; drawTag(tagX, Math.max(y + 2, peakY), fmtDb(peakDb), peakDb, label, 'Peak'); drawTag(tagX, Math.min(y + h - 22, rmsY), fmtDb(rmsDb),  rmsDb,  label, 'RMS'); }

  let lastTS = performance.now();
  function render(ts) {
    const cssW = canvas.clientWidth, cssH = canvas.clientHeight; ctx.clearRect(0, 0, cssW, cssH); tagHits = [];
    const plotX = MARGIN.left, plotY = MARGIN.top; const plotW = cssW - MARGIN.left - MARGIN.right; const plotH = cssH - MARGIN.top - MARGIN.bottom;
    const totalColsW = AXIS_W + COL_W * 2 + COL_G; const startX = plotX + Math.max(0, (plotW - totalColsW) / 2); const axisX = startX; const col1X = axisX + AXIS_W; const col2X = col1X + COL_W + COL_G; const colY = plotY; const colH = plotH;
    drawLeftAxis(axisX, colY, colH);
    const dt = Math.max(0, ts - lastTS) / 1000; lastTS = ts;
    function updateHold(currentDb, holdDb, holdTime) { if (currentDb > holdDb) { holdDb = currentDb; holdTime = 1.0; } else if (holdTime > 0) { holdTime -= dt; } else { holdDb = Math.max(MIN_DB, holdDb - 10.0 * dt); } return { holdDb, holdTime }; }
    ({ holdDb: meters.holdL, holdTime: meters.holdTimeL } = updateHold(meters.peakL, meters.holdL, meters.holdTimeL));
    ({ holdDb: meters.holdR, holdTime: meters.holdTimeR } = updateHold(meters.peakR, meters.holdR, meters.holdTimeR));
    function updateClip(currentDb, clip, clipTime) { if (currentDb > 0.0) { clip = true; clipTime = 1.0; } else if (clipTime > 0) { clipTime -= dt; } else { clip = false; } return { clip, clipTime }; }
    ({ clip: meters.clipL, clipTime: meters.clipTimeL } = updateClip(meters.peakL, meters.clipL, meters.clipTimeL));
    ({ clip: meters.clipR, clipTime: meters.clipTimeR } = updateClip(meters.peakR, meters.clipR, meters.clipTimeR));
    drawChannelVertical(col1X, colY, COL_W, colH, meters.peakL, meters.rmsL, meters.holdL, meters.clipL, 'L');
    drawChannelVertical(col2X, colY, COL_W, colH, meters.peakR, meters.rmsR, meters.holdR, meters.clipR, 'R');
    requestAnimationFrame(render);
  }
  requestAnimationFrame(render);

  const tooltip = document.getElementById('meterTooltip'); let hoverActive = false;
  function showTooltip(x, y, tag) { const ampTxt = tag.amp.toFixed(3); tooltip.innerHTML = `
      <div class="tt-title">${tag.channel} — ${tag.type}</div>
      <div class="tt-line"><span class="tt-label">dB:</span><span class="tt-value">${fmtDb(tag.db)}</span></div>
      <div class="tt-line"><span class="tt-label">Linear:</span><span class="tt-value">${ampTxt}</span></div>
    `; tooltip.style.left = `${x}px`; tooltip.style.top = `${y}px`; tooltip.setAttribute('aria-hidden', 'false'); hoverActive = true; }
  function hideTooltip() { tooltip.setAttribute('aria-hidden', 'true'); hoverActive = false; }
  function hitTest(px, py) { for (let i = 0; i < tagHits.length; i++) { const t = tagHits[i]; if (px >= t.x && px <= t.x + t.w && py >= t.y && py <= t.y + t.h) return t; } return null; }
  canvas.addEventListener('mousemove', (ev) => { const rect = canvas.getBoundingClientRect(); const px = ev.clientX - rect.left; const py = ev.clientY - rect.top; const tag = hitTest(px, py); if (tag) { canvas.style.cursor = 'pointer'; showTooltip(ev.clientX, ev.clientY, tag); } else { canvas.style.cursor = 'default'; if (hoverActive) hideTooltip(); } });
  canvas.addEventListener('mouseleave', () => { canvas.style.cursor = 'default'; if (hoverActive) hideTooltip(); });

  setInterval(() => sendToHost({ kind: 'poll_meters' }), 50);
  window.updateMeters = function(m) {
    meters.peakL = Number(m.peakL); meters.peakR = Number(m.peakR); meters.rmsL = Number(m.rmsL); meters.rmsR = Number(m.rmsR);
    const fmt = (x) => (isFinite(x) ? `${Number(x).toFixed(1)} dB` : '-∞ dB');
    const peakL = document.getElementById('peakL'); const peakR = document.getElementById('peakR'); const rmsL  = document.getElementById('rmsL'); const rmsR  = document.getElementById('rmsR');
    if (peakL) peakL.textContent = fmt(meters.peakL); if (peakR) peakR.textContent = fmt(meters.peakR); if (rmsL) rmsL.textContent = fmt(meters.rmsL); if (rmsR) rmsR.textContent = fmt(meters.rmsR);
  };
})();
