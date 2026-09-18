(() => {
  const invoke = (command) => window.__TAURI_INTERNALS__?.invoke(command) ?? Promise.reject(new Error('语音能力未提供'));
  const status = document.querySelector('#status'), transcript = document.querySelector('#transcript');
  const stop = document.querySelector('#stop'), retry = document.querySelector('#retry');
  function render(event) {
    const phase = event.phase;
    if (event.text) transcript.value = event.text;
    status.textContent = ({requesting:'正在请求系统权限…',listening:'正在聆听（最长 20 秒）',processing:'正在整理并发送给智能体…',success:'已交给智能体',delivery_unavailable:'当前没有可接收输入的 Agent',delivery_rejected:'Agent 拒绝了本次输入',delivery_unknown:'未确认 Agent 是否收到，请勿自动重试',failed:event.text || '语音输入失败',cancelled:'已取消'})[phase] || '正在准备…';
    stop.hidden = !['requesting','listening','processing'].includes(phase);
    retry.hidden = !['failed','delivery_unavailable'].includes(phase);
    transcript.readOnly = true;
  }
  window.addEventListener('yonda-voice', event => render(event.detail || {}));
  window.addEventListener('yonda-voice-open', () => { transcript.value = ''; render({phase:'requesting'}); });
  stop.addEventListener('click', () => invoke('voice_input_stop'));
  retry.addEventListener('click', async () => { transcript.value = ''; render({phase:'requesting'}); try { await invoke('voice_input_start'); } catch (error) { render({phase:'failed',text:error.message}); } });
  const close = async () => { try { await invoke('voice_input_close'); } catch { status.textContent = '关闭失败，请重试'; } };
  document.querySelector('#close').addEventListener('click', close);
  document.addEventListener('keydown', event => { if (event.key === 'Escape') { event.preventDefault(); close(); } });
})();
