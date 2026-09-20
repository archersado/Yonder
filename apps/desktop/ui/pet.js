/* 内置 UI 的短暂动画控制，不保存任务状态，也不是可导入的素材脚本。 */
(() => {
  const pet = document.querySelector('#pet');
  const voiceTrigger = document.querySelector('#voice-trigger');
  const regionTrigger = document.querySelector('#region-trigger');
  const regionStatus = document.querySelector('#region-status');
  const eyelids = document.querySelector('.eyelids');
  const breath = document.querySelector('.breath');
  const tail = document.querySelector('.tail');
  const response = document.querySelector('.response');
  const stateFrame = document.querySelector('.state-frame');
  const gestureImages = {};
  const propImages = {};
  const stateProp = document.querySelector('.state-prop');
  const propImage = stateProp.querySelector('img');
  const cameraGrip = document.querySelector('.camera-grip');
  const cameraFlash = document.querySelector('.camera-flash');
  const stateCue = document.querySelector('.state-cue');
  const waitDots = document.querySelectorAll('.wait-dots i');
  const stateGesture = document.querySelector('.state-gesture');
  const nextFrame = document.querySelector('.state-frame-next');
  const stateTail = document.querySelector('.state-tail');
  const stateTailImage = stateTail.querySelector('img');
  const statePaws = document.querySelectorAll('.state-paw');
  const stateEyelids = document.querySelector('.state-eyelids');
  const animations = JSON.parse(document.querySelector('#state-animations').textContent);
  const frameImages = {};
  const blinkImages = {};
  const bodyImages = {};
  const voiceFrame = new Image(); voiceFrame.src = 'runtime/lifecycle-v11/voice-listening.png';
  const voiceBlink = new Image(); voiceBlink.src = 'runtime/lifecycle-v11/voice-listening-blink.png';
  let framesReady = false;
  let stateStarted = Date.now();
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  let visible = !window.__TAURI_INTERNALS__ && !document.hidden;
  let visibilityRequest = 0;
  let ready = false;
  let pointer = null;
  let blinkTimer;
  let blinkStarted = 0;
  let reopenTimer;
  let responseTimer;
  let idleTimer;
  let hoverTimer;
  let nativeHovered = false;
  let hasTasks = null;
  let taskState = 'unknown';
  let agentConnected = false;
  let voiceActive = false;
  const displayedState = () => voiceActive ? 'voice_listening' : taskState;
  let menuOpen = false;
  let menuAutomatic = true;
  let menuEntered = false;
  let menuExitTimer;
  let motionTimer;
  let motionStarted = Date.now();
  let modeStarted = Date.now();
  let responseStarted = 0;
  let terminalTimer;
  let lastTerminalId = '';
  let mode = 'awake';
  const IDLE_MS = 180000;
  const active = () => ready && mode === 'awake' && visible && !reduced.matches;
  const canBlink = () => ready && ['awake', 'docked'].includes(mode) && visible && !reduced.matches;
  const native = (command, args) => window.__TAURI_INTERNALS__?.invoke(command, args) ?? Promise.resolve();
  const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
  function updateAccessibility() {
    const connection = agentConnected ? 'Agent已连接' : 'Agent未连接';
    document.title = voiceActive ? `Yonda · 正在聆听 · ${connection}` : `Yonda · ${connection}`;
    const action = voiceActive ? 'Yonda正在聆听' : mode === 'docked' ? '点击趴在边缘的Yonda唤醒它' : '轻点Yonda小龙，按住移动可拖动';
    pet.setAttribute('aria-label', `${action}，${connection}`);
  }
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
    updateAccessibility();
    sync();
  }
  function interact() {
    clearTimeout(idleTimer);
    if (mode === 'awake') idleTimer = setTimeout(hide, IDLE_MS);
  }
  regionTrigger.addEventListener('pointerdown', event => event.stopPropagation());
  regionTrigger.addEventListener('click', event => { event.stopPropagation(); interact(); native('region_preview_open').catch(error => { regionStatus.textContent=error.message==='desktop-control-active'?'Agent 正在控制桌面，暂不能圈选。':'圈选暂不可用，请稍后重试。'; }); });
  async function hide() {
    if (mode !== 'awake') return;
    if (pointer || voiceActive || taskState === 'executing' || taskState === 'unknown') { interact(); return; }
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
    stateTail.style.removeProperty('transform');
    response.style.removeProperty('transform');
    response.style.removeProperty('opacity');
    pet.style.removeProperty('--paw-right'); pet.style.removeProperty('--paw-left');
    pet.style.removeProperty('--state-paw-right'); pet.style.removeProperty('--state-paw-left');
    pet.classList.remove('active', 'blinking', 'responding');
    stateEyelids.style.opacity = '0';
    stateProp.style.removeProperty('transform'); cameraFlash.style.opacity = '0';
    stateCue.style.removeProperty('transform'); stateCue.style.removeProperty('opacity');
    for (const dot of waitDots) dot.style.removeProperty('opacity');
    pet.style.removeProperty('--wait-dot');
    stateGesture.style.removeProperty('transform');
  }
  function scheduleBlink() {
    clearTimeout(blinkTimer);
    if (!canBlink()) return;
    blinkTimer = setTimeout(() => {
      if (!canBlink()) return;
      if (!pointer && !pet.classList.contains('responding')) {
        pet.classList.add('blinking');
        blinkStarted = Date.now();
        reopenTimer = setTimeout(() => pet.classList.remove('blinking'), 180);
      }
      scheduleBlink();
    }, 3000 + Math.random() * 2000);
  }
  function drawStateFrame() {
    if (!framesReady || !visible || !['awake', 'tucking', 'waking'].includes(mode) || pointer) return;
    const shown = displayedState();
    const voiceListening = shown === 'voice_listening';
    const assetState = shown === 'voice_listening' ? 'listening' : shown;
    const state = animations[assetState] ? assetState : 'idle';
    const animation = animations[state];
    let index = animation.static_frame;
    let nextIndex = index;
    let mix = 0;
    if (!reduced.matches && shown !== 'unknown') {
      const elapsed = Date.now() - stateStarted;
      const order = animation.body_frames ?? [0, 1, 2, 3];
      const position = elapsed / animation.duration_ms * order.length;
      const step = Math.floor(position);
      index = animation.playback === 'hold' ? Math.min(3, step) : order[step % order.length];
      nextIndex = animation.playback === 'hold' ? Math.min(3, step + 1) : order[(step + 1) % order.length];
      const fraction = animation.pose_loop ? Math.min(1,(position-step)*1.6) : position-step;
      mix = fraction * fraction * (3 - 2 * fraction);
      if (state === 'idle') {
        index = 0;
        nextIndex = 0; mix = 0;
      }
      if (state === 'executing') { index = 0; nextIndex = 0; mix = 0; }
      if (!animation.pose_loop && (state === 'waiting_for_user' || state === 'paused')) { index = 3; nextIndex = 3; mix = 0; }
    }
    if (index === nextIndex) mix = 0;
    pet.dataset.pose = String(index);
    stateProp.style.display = propImages[state] ? 'block' : 'none';
    if (propImages[state] && propImage.src !== propImages[state].src) propImage.src = propImages[state].src;
    stateGesture.style.display = gestureImages[state] ? 'block' : 'none';
    if (gestureImages[state] && stateGesture.src !== gestureImages[state].src) stateGesture.src = gestureImages[state].src;
    const image = voiceListening ? voiceFrame : frameImages[state][index];
    if (propImages[state] && cameraGrip.src !== image.src) cameraGrip.src = image.src;
    const body = bodyImages[state] ?? image;
    if (stateFrame.src !== body.src) stateFrame.src = body.src;
    stateFrame.style.opacity = String(1 - mix);
    const next = voiceListening ? voiceFrame : frameImages[state][nextIndex];
    if (nextFrame.src !== next.src) nextFrame.src = next.src;
    if (stateTailImage.src !== image.src) stateTailImage.src = image.src;
    if (state === 'executing') for (const paw of statePaws) {
      if (paw.src !== image.src) paw.src = image.src;
    }
    nextFrame.style.opacity = String(mix);
    const closed = voiceListening ? voiceBlink : blinkImages[state][index];
    if (stateEyelids.src !== closed.src) stateEyelids.src = closed.src;
    const elapsedBlink = Date.now() - blinkStarted;
    const closure = elapsedBlink < 60 ? elapsedBlink / 60 : elapsedBlink < 100 ? 1 : 1 - (elapsedBlink - 100) / 80;
    stateEyelids.style.opacity = String(pet.classList.contains('blinking') && !reduced.matches ? Math.max(0, Math.min(1, closure)) : 0);
  }
  function drawMotion() {
    drawStateFrame();
    const state = displayedState();
    const listening = state === 'listening' || state === 'voice_listening';
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
    if (framesReady) {
      const period = state === 'paused' ? 5 : 3.6;
      const pulse = (1 - Math.cos(seconds * Math.PI * 2 / period)) / 2;
      breath.style.transform = `translateY(${-pulse}px) scale(${1 + .006 * pulse}, ${1 + .012 * pulse})`;
      if (state === 'idle') {
        const pulse = (1 - Math.cos(seconds * Math.PI * 2 / 3.6)) / 2;
        const bobPhase = seconds % 16;
        const bob = bobPhase < 1.2 ? 3 * Math.sin(Math.PI * bobPhase / 1.2) : 0;
        const tilt = 1.6 * Math.sin(seconds * Math.PI * 2 / 9);
        breath.style.transform = `translateY(${-2 * pulse - bob}px) rotate(${tilt}deg) scale(${1 + .014 * pulse}, ${1 + .025 * pulse})`;
        const phase = seconds % 8;
        const wag = phase < 2.4 ? 7 * Math.sin(phase * Math.PI * 2 / .8) * Math.sin(Math.PI * phase / 2.4) : 0;
        stateTail.style.transform = `skewY(${wag}deg)`;
      }
      if (state === 'executing') {
        const phase = seconds % 1.4 / 1.4;
        const lift = position => position < .25 ? (1 - Math.cos(Math.PI * position / .25)) / 2
          : position < .4 ? 1 : position < .9 ? (1 + Math.cos(Math.PI * (position - .4) / .5)) / 2 : 0;
        pet.style.setProperty('--state-paw-right', `${phase < .5 ? 14 * lift(phase * 2) : 0}deg`);
        pet.style.setProperty('--state-paw-left', `${phase >= .5 ? -14 * lift((phase - .5) * 2) : 0}deg`);
      }
      // 专属动作与呼吸/闭眼分层；循环只改变外观，不触发任务用例。
      const wave = period => Math.sin(seconds * Math.PI * 2 / period);
      const arc = period => (1 - Math.cos(seconds * Math.PI * 2 / period)) / 2;
      let action = '';
      if (listening) action = `translateY(${-3 * arc(2.8)}px) rotate(${-2 * arc(2.8)}deg)`;
      if (state === 'thinking') action = `rotate(${3.5 * wave(4.8)}deg)`;
      stateGesture.style.transform = state === 'paused' ? `perspective(500px) rotateY(${25 + 55 * arc(3.6)}deg)` : state === 'success' ? `rotate(${-12 * wave(2.8)}deg)` : '';
      if (listening) { const cue = arc(2.8); stateCue.style.transform = `scale(${.82 + .18 * cue})`; stateCue.style.opacity = String(.55 + .45 * cue); }
      if (state === 'thinking') for (let index = 0; index < waitDots.length; index++) waitDots[index].style.opacity = String(.25 + .75 * (1 + Math.cos(seconds * Math.PI * 2 / 1.8 - index * Math.PI * 2 / 3)) / 2);
      if (state === 'recording') {
        const phase = seconds % 4.8;
        const press = phase >= 1.75 && phase < 2.2 ? Math.sin(Math.PI * (phase - 1.75) / .45) ** 2 : 0;
        stateProp.style.transform = `translateY(${press}px) scale(${1 + .006 * press})`;
        const flash = phase >= 1.82 && phase < 2.27 ? .95 * Math.sin(Math.PI * (phase - 1.82) / .45) ** 2 : 0;
        cameraFlash.style.opacity = String(flash);
      }
      if (state === 'waiting_for_user') action = `translateY(${-2 * arc(4)}px) rotate(${-1 * arc(4)}deg)`;
      if (state === 'paused') action = `translateY(${1.6 * arc(5.2)}px) rotate(${-1 * arc(5.2)}deg)`;
      if (state === 'recording') action = `rotate(${.5 * wave(6)}deg)`;
      if (state === 'success') {
        const phase = seconds % 2.4;
        action = `translateY(${phase < 1.2 ? -2 * (1 - Math.cos(Math.PI * 2 * phase / 1.2)) : 0}px) rotate(${.8 * wave(2.4)}deg)`;
      }
      if (state === 'failed') action = `translateY(${3 * arc(3.4)}px) scale(1,${1 - .025 * arc(3.4)})`;
      if (state === 'thinking' || state === 'recording') stateTail.style.transform = `skewY(${(state === 'thinking' ? 2 : 8) * wave(state === 'thinking' ? 2.4 : 3.2)}deg)`;
      if (action) response.style.transform = action;
      return;
    }
    if (state === 'executing') {
      // 局部爪层交替微转，原始透明形象和爪根锚点保持不变。
      const phase = seconds % 1.4 / 1.4;
      pet.style.setProperty('--paw-right', `${phase < .5 ? -5 * Math.sin(phase * Math.PI * 2) : 0}deg`);
      pet.style.setProperty('--paw-left', `${phase >= .5 ? 4 * Math.sin((phase - .5) * Math.PI * 2) : 0}deg`);
      breath.style.transform = `translateY(${-Math.sin(seconds * Math.PI * 2 / 1.4)}px) rotate(1deg)`;
      return;
    }
    if (state === 'paused' || state === 'waiting_for_user') {
      breath.style.transform = `scaleY(${1 + .008 * Math.sin(seconds * Math.PI * 2 / 5)})`;
      return;
    }
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
    drawStateFrame();
    // WKWebView 未激活时会冻结 CSS 动画时间线；隐藏时停止，休眠仅保留眨眼。
    if (ready && visible && !reduced.matches && ['awake', 'tucking', 'waking'].includes(mode)) {
      drawMotion();
      motionTimer = setInterval(drawMotion, framesReady ? 16 : 50);
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
  function hover() {
    clearTimeout(hoverTimer);
    if (mode === 'awake' && hasTasks === true && !voiceTrigger.matches(':hover')) hoverTimer = setTimeout(() => {
      if (mode === 'awake' && hasTasks === true && !pointer && !voiceTrigger.matches(':hover')) showTaskMenu();
    }, 200);
  }
  if (!window.__TAURI_INTERNALS__) {
    pet.addEventListener('pointerenter', hover);
    pet.addEventListener('pointerleave', () => clearTimeout(hoverTimer));
  }
  pet.addEventListener('pointerdown', event => {
    clearTimeout(hoverTimer);
    nativeHovered = true;
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
      native('task_menu_hide').catch(() => console.warn('菜单收起失败'));
      invoke('plugin:window|start_dragging', { label: 'pet' }).catch(() => {
        console.warn('桌宠原生拖动启动失败');
      });
    }
  });
  pet.addEventListener('pointerup', event => {
    if (!pointer || pointer.id !== event.pointerId) return;
    const clicked = !pointer.moved;
    release();
    if (clicked) activate();
    else interact();
  });
  pet.addEventListener('pointercancel', release);
  pet.addEventListener('lostpointercapture', () => { pointer = null; pet.classList.remove('held'); });
  function activate(openMenu = false) {
    if (mode === 'docked') wake();
    else if (mode === 'awake') {
      interact(); respond();
      if (openMenu && hasTasks === true) showTaskMenu(true);
    }
  }
  pet.addEventListener('click', event => {
    // 系统辅助功能触发无指针的语义点击；普通鼠标已由pointer处理。
    if (event.detail === 0) activate();
  });
  pet.addEventListener('keydown', event => {
    if ((event.key === 'Enter' || event.key === ' ') && !event.repeat) {
      event.preventDefault();
      activate(true);
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
  async function showTaskMenu(focus = false) {
    try { menuAutomatic = !focus; menuOpen = await native('task_menu_show', { focus }) === true; menuEntered = false; }
    catch { console.warn('任务菜单暂不可用'); }
  }
  function hideTaskMenu() {
    clearTimeout(menuExitTimer); menuExitTimer = null;
    menuOpen = false; menuEntered = false;
    native('task_menu_hide').catch(() => console.warn('菜单收起失败'));
  }
  window.addEventListener('yonda-menu-open', event => { menuOpen = true; menuAutomatic = event.detail !== 'manual'; menuEntered = false; });
  function applyPresentation(nextHasTasks, nextState, keepTerminal = false) {
    if (!keepTerminal) { clearTimeout(terminalTimer); terminalTimer = null; }
    if (nextHasTasks && hasTasks !== true) nativeHovered = false;
    hasTasks = nextHasTasks;
    if (taskState !== nextState) {
      taskState = nextState; stateStarted = Date.now(); if (!voiceActive) pet.dataset.state = nextState; sync();
      if (mode === 'docked') wake();
    }
    if (!hasTasks && menuOpen && menuAutomatic) hideTaskMenu();
  }
  async function loadInitialPresentation() {
    try {
      const [nextHasTasks, nextState] = await native('pet_task_state');
      applyPresentation(nextHasTasks, nextState);
    } catch {
      hasTasks = null;
      if (taskState !== 'unknown') { taskState = 'unknown'; stateStarted = Date.now(); pet.dataset.state = 'unknown'; sync(); if (mode === 'docked') wake(); }
      clearTimeout(hoverTimer);
    }
  }
  async function loadAgentConnection() {
    try { applyAgentConnection(await native('pet_agent_connected') === true); }
    catch { applyAgentConnection(false); }
  }
  function applyAgentConnection(connected) {
    if (agentConnected === connected && pet.dataset.agentConnected) return;
    agentConnected = connected;
    pet.dataset.agentConnected = String(connected);
    updateAccessibility();
    if (mode === 'docked') wake();
  }
  window.addEventListener('yonda-presentation', event => {
    const detail = event.detail;
    if (!detail || typeof detail.hasTasks !== 'boolean' || typeof detail.state !== 'string') return;
    if (typeof detail.eventId === 'string') {
      if (!detail.eventId || detail.eventId === lastTerminalId || typeof detail.resumeHasTasks !== 'boolean' || typeof detail.resumeState !== 'string') return;
      lastTerminalId = detail.eventId;
      clearTimeout(terminalTimer);
      applyPresentation(detail.hasTasks, detail.state, true);
      if (mode === 'docked') wake();
      terminalTimer = setTimeout(() => { terminalTimer = null; applyPresentation(detail.resumeHasTasks, detail.resumeState, true); }, 1800);
      return;
    }
    applyPresentation(detail.hasTasks, detail.state);
  });
  window.addEventListener('yonda-agent-connection', event => {
    if (typeof event.detail?.connected === 'boolean') applyAgentConnection(event.detail.connected);
  });
  voiceTrigger.addEventListener('pointerdown', event => event.stopPropagation());
  voiceTrigger.addEventListener('pointerup', event => event.stopPropagation());
  voiceTrigger.addEventListener('click', async event => {
    event.stopPropagation(); clearTimeout(hoverTimer); interact();
    try { await native('voice_input_open'); }
    catch { console.warn('语音输入暂不可用'); }
  });
  window.addEventListener('yonda-voice', event => {
    const phase = event.detail?.phase;
    const nextActive = ['requesting', 'listening', 'processing'].includes(phase);
    if (nextActive !== voiceActive) {
      voiceActive = nextActive; stateStarted = Date.now(); pet.dataset.state = displayedState(); sync();
      updateAccessibility();
      if (voiceActive && mode === 'docked') wake();
    }
  });
  async function checkHover() {
    try {
      const [inside, inMenu] = await native('pet_hover_region');
      if (inside || inMenu) { clearTimeout(menuExitTimer); menuExitTimer = null; }
      if (inMenu) menuEntered = true;
      if (visible && mode === 'awake' && inside && !nativeHovered && !pointer) hover();
      if (!inside) clearTimeout(hoverTimer);
      if (menuOpen && !inside && !inMenu && (menuAutomatic || menuEntered)) {
        if (menuEntered) hideTaskMenu();
        else if (!menuExitTimer) menuExitTimer = setTimeout(() => { menuExitTimer = null; hideTaskMenu(); }, 350);
      }
      nativeHovered = inside;
    } catch { clearTimeout(hoverTimer); }
    setTimeout(checkHover, 250);
  }
  if (window.__TAURI_INTERNALS__) checkHover();
  loadAgentConnection();
  refreshVisibility();
  Promise.all([voiceFrame.decode(), voiceBlink.decode(), ...Object.entries(animations).map(async ([state, animation]) => {
    frameImages[state] = await Promise.all(animation.files.map(async file => {
      const image = new Image(); image.src = file; await image.decode(); return image;
    }));
    blinkImages[state] = await Promise.all(animation.blink_files.map(async file => {
      const image = new Image(); image.src = file; await image.decode(); return image;
    }));
    if (animation.prop_file) { const image = new Image(); image.src = animation.prop_file; await image.decode(); propImages[state] = image; }
    if (animation.gesture_file) { const image = new Image(); image.src = animation.gesture_file; await image.decode(); gestureImages[state] = image; }
    if (animation.body_file) { const image = new Image(); image.src = animation.body_file; await image.decode(); bodyImages[state] = image; }
  })]).then(() => {
    // 透明素材全部解码后启用；连续局部动作不依赖整图加法混合。
    framesReady = true;
    const state = displayedState() === 'voice_listening' ? 'listening' : displayedState(); stateFrame.src = frameImages[animations[state] ? state : 'idle'][0].src;
    if (framesReady) pet.classList.add('frames-ready'); sync();
    if (window.__TAURI_INTERNALS__) loadInitialPresentation();
  }).catch(error => { pet.dataset.assetError = String(error?.message ?? error); console.warn('状态素材加载失败，保留原透明小龙'); });
  Promise.all([eyelids.decode(), document.querySelector('.peek-eyelids').decode()]).then(() => { ready = true; setMode('awake'); interact(); }).catch(() => {
    console.warn('闭眼素材加载失败，保留静态小龙');
  });
})();
