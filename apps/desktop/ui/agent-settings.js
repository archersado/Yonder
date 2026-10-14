// @ts-check
(() => {
const byId = id => document.getElementById(id);
const status = byId('agent-status');
const list = byId('agent-list');
const sessionList = byId('agent-session-list');
const sessionHint = byId('agent-session-hint');
const voiceReplyEnabled = byId('voice-reply-enabled');
const voiceReplyStatus = byId('voice-reply-status');
const stateText = value => ({ enabled: '已启用', disabled: '已禁用', revoked: '已撤权' })[value] ?? value;
const timeText = value => value ? new Date(value).toLocaleString() : '未连接';

async function invoke(command, args) {
  if (!window.__TAURI_INTERNALS__) throw new Error('本机Agent管理能力未提供');
  return window.__TAURI_INTERNALS__.invoke(command, args);
}

function row(agent) {
  const item = document.createElement('li');
  const meta = document.createElement('div');
  meta.className = 'meta';
  const id = document.createElement('span');
  id.className = 'id';
  id.textContent = agent.agent_id;
  const state = document.createElement('span');
  state.className = 'state';
  state.textContent = stateText(agent.status);
  meta.append(id, state);
  const seen = document.createElement('span');
  seen.className = 'state';
  seen.textContent = `最近连接：${timeText(agent.last_seen_at)}`;
  const actions = document.createElement('div');
  actions.className = 'actions';
  const disable = document.createElement('button');
  disable.type = 'button';
  disable.textContent = agent.status === 'disabled' ? '启用' : '禁用';
  disable.disabled = agent.status === 'revoked';
  disable.addEventListener('click', () => update(agent, agent.status === 'disabled' ? 'enabled' : 'disabled'));
  const revoke = document.createElement('button');
  revoke.type = 'button';
  revoke.textContent = '撤权';
  revoke.disabled = agent.status === 'revoked';
  let confirmingRevoke = false;
  revoke.addEventListener('click', () => {
    if (!confirmingRevoke) {
      confirmingRevoke = true;
      revoke.textContent = '确认撤权';
      status.textContent = `再次点击以撤权 ${agent.agent_id}`;
      return;
    }
    update(agent, 'revoked');
  });
  actions.append(disable, revoke);
  item.append(meta, seen, actions);
  return item;
}

function sessionRow(session) {
  const item = document.createElement('li');
  const meta = document.createElement('div');
  meta.className = 'meta';
  const id = document.createElement('span');
  id.className = 'id';
  id.textContent = `${session.agent_id} · 会话 ${session.session_ref}`;
  const state = document.createElement('span');
  state.className = session.active ? 'state session-current' : 'state';
  state.textContent = session.active ? (session.explicit ? '当前接收目标' : '已自动选择') : '在线';
  meta.append(id, state);
  const capabilities = document.createElement('span');
  capabilities.className = 'state';
  const values = [];
  if (session.supports_attachment) values.push('支持截图');
  if (session.supports_reply) values.push('支持回复');
  capabilities.textContent = values.length ? values.join(' · ') : '支持文字输入';
  item.append(meta, capabilities);
  if (!session.active) {
    const actions = document.createElement('div');
    actions.className = 'actions';
    const choose = document.createElement('button');
    choose.type = 'button';
    choose.textContent = '选择';
    choose.setAttribute('aria-label', `选择 ${session.agent_id} 会话 ${session.session_ref} 作为接收目标`);
    choose.addEventListener('click', () => selectSession(session, choose));
    actions.append(choose);
    item.append(actions);
  }
  return item;
}

function renderSessions(sessions) {
  sessionList.replaceChildren(...sessions.map(sessionRow));
  const active = sessions.find(session => session.active);
  sessionHint.textContent = sessions.length === 0
    ? '当前没有可接收输入的在线会话。'
    : active
      ? `${active.explicit ? '已选择' : '已自动选择'} ${active.agent_id} · 会话 ${active.session_ref}`
      : '请选择圈选与语音输入的接收会话。';
}

async function loadSessions() {
  try {
    renderSessions(await invoke('agent_session_list'));
  } catch (error) {
    sessionHint.textContent = `在线会话读取失败：${error?.message ?? error ?? '请重试'}`;
  }
}

async function selectSession(session, button) {
  button.disabled = true;
  sessionHint.textContent = '正在选择接收会话…';
  try {
    renderSessions(await invoke('agent_session_select', { sessionRef: session.session_ref }));
    sessionHint.textContent = `已选择 ${session.agent_id} · 会话 ${session.session_ref}`;
  } catch (error) {
    sessionHint.textContent = error?.message?.includes('断开') || String(error).includes('断开')
      ? '该会话已断开，请重新选择。'
      : `会话选择失败：${error?.message ?? error ?? '请重试'}`;
    await loadSessions();
  }
}

async function load() {
  status.textContent = '正在读取 Agent…';
  try {
    const agents = await invoke('agent_registry_list');
    list.replaceChildren(...agents.map(row));
    status.textContent = agents.length ? 'Agent列表已更新' : '尚无登记Agent';
  } catch (error) {
    status.textContent = `Agent列表读取失败：${error?.message ?? error ?? '请重试'}`;
  }
}

async function loadVoiceReply() {
  voiceReplyStatus.textContent = '正在读取本地语音能力…';
  try {
    const config = await invoke('voice_reply_config_get');
    voiceReplyEnabled.checked = config.enabled;
    voiceReplyEnabled.disabled = !config.available;
    voiceReplyStatus.textContent = config.available
      ? (config.enabled ? '已开启；只使用本机已安装的中文声音。' : '默认关闭；开启后只使用本机已安装的中文声音。')
      : '本地中文声音不可用；文字回复不受影响。';
  } catch (error) {
    voiceReplyEnabled.disabled = true;
    voiceReplyStatus.textContent = `语音回复配置读取失败：${error?.message ?? error ?? '请重试'}`;
  }
}

voiceReplyEnabled.addEventListener('change', async () => {
  voiceReplyEnabled.disabled = true;
  try {
    const config = await invoke('voice_reply_config_save', { enabled: voiceReplyEnabled.checked });
    voiceReplyEnabled.checked = config.enabled;
    voiceReplyStatus.textContent = !config.available ? '本地中文声音不可用；文字回复不受影响。' : config.enabled ? '已开启本地语音回复。' : '已关闭并停止语音回复。';
    voiceReplyEnabled.disabled = !config.available;
  } catch (error) {
    voiceReplyStatus.textContent = `语音回复配置保存失败：${error?.message ?? error ?? '请重试'}`;
    await loadVoiceReply();
  }
});

async function update(agent, value) {
  status.textContent = '正在更新 Agent状态…';
  try {
    await invoke('agent_registry_set_status', { agentId: agent.agent_id, status: value });
    status.textContent = `${agent.agent_id}${stateText(value)}`;
    await Promise.all([load(), loadSessions()]);
  } catch (error) {
    status.textContent = `Agent状态更新失败：${error?.message ?? error ?? '请重试'}`;
  }
}

byId('register-form').addEventListener('submit', async event => {
  event.preventDefault();
  const input = byId('agent-id');
  const agentId = input.value.trim();
  const button = byId('register');
  button.disabled = true;
  status.textContent = '正在登记 Agent…';
  try {
    await invoke('agent_registry_register', { agentId });
    input.value = '';
    status.textContent = `已登记 ${agentId}`;
    await load();
  } catch (error) {
    status.textContent = `Agent登记失败：${error?.message ?? error ?? '请检查ID'}`;
  } finally {
    button.disabled = false;
  }
});

byId('close').addEventListener('click', async () => {
  try { await invoke('agent_settings_close'); }
  catch { status.textContent = '关闭失败，请重试'; }
});
document.addEventListener('keydown', event => {
  if (event.key === 'Escape') { event.preventDefault(); byId('close').click(); }
});
window.addEventListener('yonda-agent-open', () => { load(); loadSessions(); loadVoiceReply(); });
window.addEventListener('yonda-agent-sessions-changed', loadSessions);
load();
loadSessions();
loadVoiceReply();
})();
