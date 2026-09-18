/* 内置 UI 的短暂动画控制，不保存任务状态，也不是可导入的素材脚本。 */
(() => {
  const pet = document.querySelector('#pet');
  const eyelids = document.querySelector('.eyelids');
  const breath = document.querySelector('.breath');
  const tail = document.querySelector('.tail');
  const response = document.querySelector('.response');
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  let visible = !window.__TAURI_INTERNALS__ && !document.hidden;
  let visibilityRequest = 0;
  let ready = false;
  let pointer = null;
  let blinkTimer;
  let reopenTimer;
  let responseTimer;
  let idleTimer;
  let motionTimer;
  let motionStarted = Date.now();
  let modeStarted = Date.now();
  let responseStarted = 0;
  let mode = 'awake';
  const IDLE_MS = 180000;
  const active = () => ready && mode === 'awake' && visible && !reduced.matches;
  const canBlink = () => ready && ['awake', 'docked'].includes(mode) && visible && !reduced.matches;
  const native = command => window.__TAURI_INTERNALS__?.invoke(command) ?? Promise.resolve();
  const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
  async function refreshVisibility() {
    const request = ++visibilityRequest;
    let next = false;
    try {
      next = window.__TAURI_INTERNALS__
        ? await native('pet_is_visible') === true : !document.hidden;
    } catch (_) { console.warn('窗口可见性暂不可用，暂停动画'); }
    if (request !== visibilityRequest || next === visible) return;
    visible = next;
    sync();
  }
  function setMode(next) {
    mode = next;
    modeStarted = Date.now();
    pet.dataset.mode = next;
    pet.setAttribute('aria-label', next === 'docked' ? '点击趴在边缘的 Yonda 唤醒它' : '轻点 Yonda 小龙，按住移动可拖动');
    sync();
  }
  function interact() {
    clearTimeout(idleTimer);
    if (mode === 'awake') idleTimer = setTimeout(hide, IDLE_MS);
  }
  async function hide() {
    if (mode !== 'awake') return;
    if (pointer) { interact(); return; }
    clearTimeout(idleTimer);
    setMode('tucking');
    await wait(reduced.matches ? 0 : 320);
    try {
      // Rust 在窗口移动前检查任务门禁，前端不能设置任务数。
      const edge = await native('pet_dock');
      pet.dataset.edge = ['left', 'right', 'top', 'bottom'].includes(edge) ? edge : 'bottom';
      setMode('docked');
    } catch (_) {
      setMode('awake'); interact();
      console.warn('暂不隐藏小龙：任务门禁或窗口停靠失败');
    }
  }
  async function wake() {
    if (mode !== 'docked') return;
    // 原生操作期间仍保留眼睛入口；重复点击不派发第二次恢复。
    mode = 'restoring';
    try {
      await native('pet_wake');
      setMode('waking');
      await wait(reduced.matches ? 0 : 360);
      setMode('awake'); interact();
    } catch (_) { setMode('docked'); console.warn('唤醒失败，请再次点击眼睛'); }
  }

  function stop() {
    clearTimeout(blinkTimer);
    clearTimeout(reopenTimer);
    clearTimeout(responseTimer);
    clearInterval(motionTimer);
    breath.style.removeProperty('transform');
    tail.style.removeProperty('transform');
    response.style.removeProperty('transform');
    response.style.removeProperty('opacity');
    pet.classList.remove('active', 'blinking', 'responding');
  }
  function scheduleBlink() {
    clearTimeout(blinkTimer);
    if (!canBlink()) return;
    blinkTimer = setTimeout(() => {
      if (!canBlink()) return;
      if (!pointer && !pet.classList.contains('responding')) {
        pet.classList.add('blinking');
        reopenTimer = setTimeout(() => pet.classList.remove('blinking'), 130);
      }
      scheduleBlink();
    }, 3000 + Math.random() * 2000);
  }
  function drawMotion() {
    const now = Date.now();
    if (mode === 'tucking' || mode === 'waking') {
      const hiding = mode === 'tucking';
      const progress = Math.min(1, (now - modeStarted) / (hiding ? 320 : 360));
      const amount = hiding ? progress : 1 - progress;
      response.style.transform = `translateY(${6 * amount}px) scale(${1 - .25 * amount})`;
      response.style.opacity = String(1 - amount);
      return;
    }
    if (pet.classList.contains('responding')) {
      const nod = Math.sin(Math.PI * Math.min(1, (now - responseStarted) / 400));
      response.style.transform = `translateY(${2 * nod}px) rotate(${2 * nod}deg)`;
      return;
    }
    response.style.removeProperty('transform');
    if (pointer) return;
    const seconds = (now - motionStarted) / 1000;
    const pulse = (1 - Math.cos(seconds * Math.PI * 2 / 3.6)) / 2;
    const bobPhase = seconds % 16;
    const bob = bobPhase < 1.2 ? 3 * Math.sin(Math.PI * bobPhase / 1.2) : 0;
    const tilt = 1.6 * Math.sin(seconds * Math.PI * 2 / 9);
    const wagPhase = seconds % 8;
    const wag = wagPhase < 2.4 ? 7 * Math.sin(wagPhase * Math.PI * 2 / .8) * Math.sin(Math.PI * wagPhase / 2.4) : 0;
    tail.style.transform = `skewY(${wag}deg)`;
    breath.style.transform = `translateY(${-2 * pulse - bob}px) rotate(${tilt}deg) scale(${1 + .014 * pulse}, ${1 + .025 * pulse})`;
  }
  function sync() {
    stop();
    // WKWebView 未激活时会冻结 CSS 动画时间线；隐藏时停止，休眠仅保留眨眼。
    if (ready && visible && !reduced.matches && ['awake', 'tucking', 'waking'].includes(mode)) {
      drawMotion();
      motionTimer = setInterval(drawMotion, 50);
    }
    if (active()) pet.classList.add('active');
    if (canBlink()) scheduleBlink();
  }
  function respond() {
    if (!active() || pet.classList.contains('responding')) return;
    clearTimeout(reopenTimer);
    pet.classList.remove('blinking');
    pet.classList.add('responding');
    responseStarted = Date.now();
    drawMotion();
    responseTimer = setTimeout(() => pet.classList.remove('responding'), 400);
  }
  function release() {
    if (pointer && pet.hasPointerCapture(pointer.id)) pet.releasePointerCapture(pointer.id);
    pointer = null;
    pet.classList.remove('held');
  }
  pet.addEventListener('pointerdown', event => {
    if (event.button !== 0 || !event.isPrimary) return;
    if (mode === 'docked') { event.preventDefault(); wake(); return; }
    if (mode !== 'awake') return;
    interact();
    pointer = { id: event.pointerId, x: event.clientX, y: event.clientY, moved: false };
    pet.setPointerCapture(event.pointerId);
    pet.classList.add('held');
    clearTimeout(reopenTimer);
    clearTimeout(responseTimer);
    pet.classList.remove('blinking', 'responding');
  });
  pet.addEventListener('pointermove', event => {
    interact();
    if (!pointer || pointer.id !== event.pointerId || pointer.moved) return;
    if (Math.hypot(event.clientX - pointer.x, event.clientY - pointer.y) <= 4) return;
    pointer.moved = true;
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (invoke) {
      // 原生拖动接管后可能不再派发 pointerup，不保留悬挂的指针状态。
      release();
      invoke('plugin:window|start_dragging', { label: 'pet' }).catch(() => {
        console.warn('桌宠原生拖动启动失败');
      });
    }
  });
  pet.addEventListener('pointerup', event => {
    if (!pointer || pointer.id !== event.pointerId) return;
    const clicked = !pointer.moved;
    release();
    interact();
    if (clicked) respond();
  });
  pet.addEventListener('pointercancel', release);
  pet.addEventListener('lostpointercapture', () => { pointer = null; pet.classList.remove('held'); });
  function activate() {
    if (mode === 'docked') wake();
    else if (mode === 'awake') { interact(); respond(); }
  }
  pet.addEventListener('click', event => {
    // 系统辅助功能触发无指针的语义点击；普通鼠标已由pointer处理。
    if (event.detail === 0) activate();
  });
  pet.addEventListener('keydown', event => {
    if ((event.key === 'Enter' || event.key === ' ') && !event.repeat) {
      event.preventDefault();
      activate();
    }
  });
  window.addEventListener('blur', () => { release(); refreshVisibility(); });
  window.addEventListener('focus', refreshVisibility);
  window.addEventListener('yonda-show', () => {
    refreshVisibility();
    if (mode === 'docked') wake();
    else interact();
  });
  document.addEventListener('visibilitychange', () => {
    release(); refreshVisibility();
    // Space 切换或 WKWebView 隐藏不算与小龙互动，不重置闲置计时。
  });
  reduced.addEventListener('change', sync);
  refreshVisibility();
  Promise.all([eyelids.decode(), document.querySelector('.peek-eyelids').decode()]).then(() => { ready = true; setMode('awake'); interact(); }).catch(() => {
    console.warn('闭眼素材加载失败，保留静态小龙');
  });
})();
