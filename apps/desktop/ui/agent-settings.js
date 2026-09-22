// @ts-check
(() => {
const byId = id => document.getElementById(id);
const status = byId('agent-status');
const list = byId('agent-list');
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

async function update(agent, value) {
  status.textContent = '正在更新 Agent状态…';
  try {
    await invoke('agent_registry_set_status', { agentId: agent.agent_id, status: value });
    status.textContent = `${agent.agent_id}${stateText(value)}`;
    await load();
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
window.addEventListener('yonda-agent-open', load);
load();
})();
