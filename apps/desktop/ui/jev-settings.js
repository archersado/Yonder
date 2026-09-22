// @ts-check
(() => {
const byId = id => document.getElementById(id);
const status = byId('jev-status');

async function invoke(command, args) {
  if (!window.__TAURI_INTERNALS__) throw new Error('本机设置能力未提供');
  return window.__TAURI_INTERNALS__.invoke(command, args);
}
async function load() {
  status.textContent = '正在读取 Jev 配置…';
  try {
    const config = await invoke('jev_config_get');
    byId('jev-enabled').checked = Boolean(config.enabled);
    byId('jev-service-mode').value = config.service_mode;
    byId('jev-endpoint').value = config.endpoint;
    byId('jev-step-limit').value = config.step_limit;
    byId('jev-time-limit').value = config.time_limit_ms;
    byId('jev-token-limit').value = config.token_limit;
    for (const input of document.querySelectorAll('.jev-capability')) input.checked = config.capabilities.includes(input.value);
    status.textContent = config.enabled ? '已启用；保存成功不代表模型可用' : '未启用';
  } catch (error) {
    status.textContent = `Jev 配置读取失败：${error?.message ?? error ?? '请重试'}`;
  }
}
async function save(event) {
  event.preventDefault();
  const button = byId('jev-save');
  button.disabled = true;
  status.textContent = '正在保存 Jev 配置…';
  const config = {
    enabled: byId('jev-enabled').checked,
    service_mode: byId('jev-service-mode').value,
    endpoint: byId('jev-endpoint').value.trim(),
    step_limit: Number(byId('jev-step-limit').value),
    time_limit_ms: Number(byId('jev-time-limit').value),
    token_limit: Number(byId('jev-token-limit').value),
    capabilities: [...document.querySelectorAll('.jev-capability')].filter(input => input.checked).map(input => input.value)
  };
  try {
    const saved = await invoke('jev_config_save', { config });
    status.textContent = saved.enabled ? '已启用；保存成功不代表模型可用' : '已保存，当前未启用';
  } catch (error) {
    status.textContent = `Jev 配置保存失败：${error?.message ?? error ?? '请检查字段'}`;
  } finally {
    button.disabled = false;
  }
}
async function close() {
  try { await invoke('jev_settings_close'); }
  catch { status.textContent = '关闭失败，请重试'; }
}
byId('jev-form').addEventListener('submit', save);
byId('close').addEventListener('click', close);
document.addEventListener('keydown', event => { if (event.key === 'Escape') { event.preventDefault(); close(); } });
window.addEventListener('yonda-jev-open', load);
load();
})();
