// @ts-check
// 协议模型唯一来源：crates/protocol/generated/protocol.ts；此页不定义第二套状态。
(() => {
const byId = id => document.getElementById(id);
const notice = byId('notice'), tasks = byId('tasks'), detail = byId('detail');
const next = byId('next'), refresh = byId('refresh');
const labels = { created: '已创建', running: '执行中', 'waiting-for-user': '等待用户', paused: '已暂停', interrupted: '已中断', completed: '已完成', failed: '失败', cancelled: '已取消' };
const sourceLabels = { 'local-agent': '本地 Agent', 'cloud-agent': '云端 Agent' };
const observationLabels = { matched: '已匹配', 'not-matched': '未匹配', unknown: '未知' };
const unknownLabels = { 'invalid-input': '输入无效', 'dependency-unavailable': '依赖不可用', 'worker-failed': '执行器失败', 'timed-out': '执行超时', 'invalid-response': '响应无效', 'identity-mismatch': '执行身份不匹配', 'observe-failed': '观察失败', 'user-input': '用户已接管输入' };
let includeFinished = false, cursor = null, nextCursor = null, pageNumber = 1;
let round = 0, selection = 0;
const pendingControls = new Map();

async function query(method, params) {
  const invoke = window.__TAURI_INTERNALS__?.invoke;
  if (!invoke) throw new Error('任务查询能力未提供');
  const request = JSON.stringify({ jsonrpc: '2.0', id: crypto.randomUUID(), method,
    params: { agent_id: 'desktop', capability: method === 'task.cancel' ? 'task.cancel' : method === 'task.control' ? 'task.control' : 'task.read', deadline: Date.now() + 10000, ...params } });
  const response = JSON.parse(await invoke('task_query', { request }));
  if (response.error) throw new Error(response.error.message);
  return response.result;
}
function message(text, error = false) { notice.textContent = text; notice.dataset.error = String(error); }
function placeholder(text) {
  detail.replaceChildren();
  const p = document.createElement('p'); p.className = 'placeholder'; p.textContent = text; detail.append(p);
}
function action(task, text, reason, handler) {
  const button = document.createElement('button'); button.textContent = text;
  button.disabled = Boolean(reason); button.title = reason || text;
  if (reason) button.setAttribute('aria-label', `${text}：${reason}`);
  if (handler) button.addEventListener('click', () => handler(task, button));
  return button;
}
async function cancelTask(task, button) {
  button.disabled = true;
  try {
    const result = await query('task.cancel', { task_id: task.task_id, expected_sequence: task.sequence });
    if (result.kind !== 'snapshot' || result.task.status !== 'cancelled') throw new Error('取消结果未确认');
    await load();
    message('任务已取消，数据和历史已保留');
  } catch (error) {
    message(`取消失败：${error.message ?? '请刷新后重试'}`, true);
    button.disabled = false;
  }
}
async function controlTask(task, kind, button) {
  button.disabled = true; button.textContent = '正在停止…';
  try {
    const result = kind === 'takeover'
      ? JSON.parse(await window.__TAURI_INTERNALS__.invoke('user_takeover', { taskId: task.task_id, expectedSequence: task.sequence }))
      : await query('task.control', { task_id: task.task_id, expected_sequence: task.sequence, kind });
    if (result?.error) throw new Error(result.error.message);
    const value = result?.result ?? result;
    if (value.kind !== 'control') throw new Error('停止请求未登记');
    if (value.control.phase === 'pending') {
      pendingControls.set(task.task_id, kind);
      message(kind === 'takeover' ? '正在停止任务，确认边界后可接管' : '正在停止任务，数据和历史将保留');
    } else if (kind === 'takeover' && value.control.focus_phase === 'focused') {
      pendingControls.delete(task.task_id); message('任务工作已定位，可以接管'); await load();
    } else if (kind === 'takeover' && value.control.focus_phase === 'failed') {
      pendingControls.delete(task.task_id); message('未能定位任务工作，请手动打开', true); await load();
    } else if (kind === 'cancel' && value.control.phase === 'stopped') {
      pendingControls.delete(task.task_id); message('任务已取消，数据和历史已保留'); await load();
    } else throw new Error('停止结果待核实');
  } catch (error) {
    message(`操作失败：${error.message ?? '请刷新后重试'}`, true);
    button.disabled = false; button.textContent = kind === 'takeover' ? '接管' : '取消任务';
  }
}
async function openBrowserTaskSpace(task, button) {
  button.disabled = true; button.textContent = '正在交接…';
  try {
    await window.__TAURI_INTERNALS__.invoke('browser_task_space_open', { taskId: task.task_id, expectedSequence: task.sequence });
    message('已在 ego-lite 中打开对应任务');
  } catch (error) {
    message(`打开失败：${error.message ?? error ?? '请刷新后重试'}`, true);
    button.disabled = false; button.textContent = '打开 ego-lite';
  }
}
function timelineText(event) {
  if (event.step_declaration) return `Agent 声明步骤：${event.step_declaration.label}`;
  if (event.attempt_result?.phase === 'unknown') return `执行结果未知：${unknownLabels[event.attempt_result.unknown_reason] ?? '原因未提供'}`;
  if (event.attempt_result?.phase === 'observed') return event.attempt_result.action_succeeded ? '动作已观察：成功' : '动作已观察：未达成';
  if (event.wait_reason) return `等待用户：${event.wait_reason}`;
  return `${labels[event.previous] ?? event.previous} → ${labels[event.status] ?? event.status}`;
}
function appendTimeline(list, events, after) {
  let cursor = BigInt(after);
  for (const event of events) {
    const sequenceValue = BigInt(event.sequence);
    if (sequenceValue <= cursor) throw new Error('时间线序号无效');
    const item = document.createElement('li'), sequence = document.createElement('small'), text = document.createElement('span');
    sequence.textContent = `#${event.sequence}`; text.textContent = timelineText(event);
    item.append(sequence, text); list.append(item); cursor = sequenceValue;
  }
  return cursor.toString();
}
function offerTimelineMore(container, list, task, after, current, currentRound) {
  if (BigInt(after) >= BigInt(task.sequence)) return;
  const area = document.createElement('div'); area.className = 'timeline-more';
  const hint = document.createElement('p'); hint.textContent = '仍有记录未加载';
  const button = document.createElement('button'); button.textContent = '加载更多时间线';
  area.append(hint, button); container.append(area);
  button.addEventListener('click', async () => {
    button.disabled = true; button.textContent = '正在加载…';
    area.querySelector('.timeline-error')?.remove();
    try {
      const page = await query('task.events', { task_id: task.task_id, after_sequence: after, limit: 20 });
      if (current !== selection || currentRound !== round) return;
      if (page.kind !== 'events' || !Array.isArray(page.events) || !page.events.length) throw new Error('后续记录为空，请刷新任务');
      const nextAfter = appendTimeline(list, page.events, after);
      area.remove(); offerTimelineMore(container, list, task, nextAfter, current, currentRound);
    } catch (error) {
      if (current !== selection || currentRound !== round) return;
      button.disabled = false; button.textContent = '重试加载时间线';
      const failure = document.createElement('p'); failure.className = 'timeline-error'; failure.textContent = `时间线读取失败：${error.message ?? '请重试'}`; area.append(failure);
    }
  });
}
async function select(task, button) {
  const current = ++selection, currentRound = round;
  for (const item of tasks.querySelectorAll('button.task')) item.setAttribute('aria-pressed', String(item === button));
  placeholder('正在读取详情…');
  const recentAfter = (BigInt(task.sequence) > 20n ? BigInt(task.sequence) - 20n : 0n).toString();
  const [detailResult, timelineResult, recentResult, browserResult] = await Promise.allSettled([
    query('task.step.get', { task_id: task.task_id }),
    query('task.events', { task_id: task.task_id, after_sequence: '0', limit: 20 }),
    query('task.events', { task_id: task.task_id, after_sequence: recentAfter, limit: 20 }),
    query('task.browser.get', { task_id: task.task_id })
  ]);
  if (current !== selection || currentRound !== round) return;
  try {
    if (detailResult.status === 'rejected') throw detailResult.reason;
    const result = detailResult.value;
    if (result.kind !== 'step') throw new Error('任务详情响应不可用');
    detail.replaceChildren();
    const h2 = document.createElement('h2'); h2.textContent = result.task.name || "未命名历史任务"; detail.append(h2);
    const dl = document.createElement('dl');
    const events = recentResult.status === 'fulfilled' && recentResult.value.kind === 'events' ? recentResult.value.events : [];
    const waitReason = [...events].reverse().find(event => typeof event.wait_reason === 'string' && event.wait_reason)?.wait_reason ?? '未提供';
    const statusReason = [...events].reverse().map(event => event.wait_reason
      ? `等待用户：${event.wait_reason}`
      : event.attempt_result?.phase === 'unknown' ? `执行结果未知：${unknownLabels[event.attempt_result.unknown_reason] ?? '原因未提供'}` : null
    ).find(Boolean) ?? '未提供';
    const step = result.task.current_step ?? result.step;
    const observation = result.task.observation
      ? `${observationLabels[result.task.observation.result] ?? result.task.observation.result} · ${result.task.observation.summary}`
      : '未观察';
    let browser = '未关联', browserReference = null;
    if (browserResult.status === 'rejected') browser = '关联状态读取失败';
    else if (browserResult.value.kind !== 'browser-state') browser = '关联状态读取失败';
    else if (browserResult.value.reference) {
      const reference = browserReference = browserResult.value.reference, ownership = {agent:'Agent控制',agentDelegatedToUser:'用户控制',user:'用户控制'}[reference.ownership] ?? reference.ownership;
      browser = `${reference.external_task_ref} · ${ownership} · ${reference.managed_pages}个托管页面 · ${reference.finished ? '已结束' : '活动'} · 更新序号 ${reference.updated_sequence}`;
    }
    for (const [name, value] of [['任务 ID', result.task.task_id], ['Agent', result.task.owner_agent_id], ['状态', labels[result.task.status] ?? result.task.status], ['状态说明', statusReason], ['序号', result.task.sequence], ['来源', sourceLabels[result.task.source] ?? '来源未知'], ['当前步骤', step?.label ?? '未声明步骤'], ['步骤标识', step ? `${step.step_id} · 接受序号 ${step.accepted_sequence}` : '未提供'], ['观察摘要', observation], ['下一步意图', result.task.next_intent ?? '未声明意图'], ['等待原因', waitReason], ['浏览器 Task Space', browser]]) {
      const dt = document.createElement('dt'), dd = document.createElement('dd'); dt.textContent = name; dd.textContent = value; dl.append(dt, dd);
      if (name === '浏览器 Task Space' && browserReference?.ownership === 'agent' && !browserReference.finished) {
        const open = document.createElement('button'); open.className = 'browser-open'; open.textContent = '打开 ego-lite';
        open.addEventListener('click', () => openBrowserTaskSpace(result.task, open)); dd.append(document.createElement('br'), open);
      }
    }
    detail.append(dl);
    const heading = document.createElement('h3'); heading.textContent = '时间线'; detail.append(heading);
    if (timelineResult.status === 'rejected') {
      const error = document.createElement('p'); error.className = 'timeline-error'; error.textContent = `时间线读取失败：${timelineResult.reason?.message ?? '请重试'}`; detail.append(error);
    } else {
      const timeline = timelineResult.value;
      if (timeline.kind !== 'events' || !Array.isArray(timeline.events)) throw new Error('时间线响应不可用');
      if (!timeline.events.length) {
        const empty = document.createElement('p'); empty.className = 'timeline-empty'; empty.textContent = '暂无已提交记录'; detail.append(empty);
      } else {
        const list = document.createElement('ol'); list.className = 'timeline';
        const after = appendTimeline(list, timeline.events, '0');
        detail.append(list);
        offerTimelineMore(detail, list, result.task, after, current, currentRound);
      }
    }
  } catch (error) {
    if (current === selection && currentRound === round) placeholder(`详情读取失败：${error.message ?? '请重试'}`);
  }
}
async function load(reset = true) {
  const current = ++round; ++selection;
  if (reset) { cursor = null; pageNumber = 1; }
  const stale = tasks.childElementCount > 0;
  placeholder('选择任务查看详情'); message('正在读取任务…');
  next.hidden = true; refresh.disabled = true;
  try {
    const result = await query('task.list', { after_task_id: cursor, include_finished: includeFinished, running_only: !includeFinished, limit: 20 });
    if (current !== round) return;
    if (result.kind !== 'tasks' || !Array.isArray(result.tasks)) throw new Error('任务列表响应不可用');
    const visibleTasks = result.tasks;
    tasks.replaceChildren();
    for (const task of visibleTasks) {
      if (task.status !== 'running') pendingControls.delete(task.task_id);
      const pending = pendingControls.has(task.task_id);
      const li = document.createElement('li'), button = document.createElement('button');
      button.className = 'task'; button.setAttribute('aria-pressed', 'false');
      const title = document.createElement('strong'), agent = document.createElement('small'), status = document.createElement('span');
      title.textContent = task.name || "未命名历史任务";
      title.title = title.textContent; agent.textContent = `Agent · ${task.owner_agent_id}`;
      status.className = 'status'; status.textContent = labels[task.status] ?? task.status;
      button.append(title, agent, status); button.addEventListener('click', () => select(task, button)); li.append(button);
      const actions = document.createElement('div'); actions.className = 'task-actions';
      actions.append(
        action(task, pending ? '正在停止…' : '接管', pending ? '停止请求已登记' : task.status === 'running' ? '' : '仅执行中的任务支持接管', (task,button)=>controlTask(task,'takeover',button)),
        action(task, pending ? '正在停止…' : '取消任务', pending ? '停止请求已登记' : ['created','running'].includes(task.status) ? '' : '仅未开始或执行中的任务支持取消', task.status === 'created' ? cancelTask : (task,button)=>controlTask(task,'cancel',button))
      );
      li.append(actions); tasks.append(li);
    }
    nextCursor = result.next_after_task_id; next.hidden = !nextCursor;
    byId('page-label').textContent = `第 ${pageNumber} 页 · ${visibleTasks.length} 项`;
    message(visibleTasks.length ? '显示真实任务状态' : includeFinished ? '暂无任务' : '暂无正在执行的任务');
  } catch (error) {
    if (current !== round) return;
    message(`${error.message ?? '任务读取失败'}${stale ? '；已有列表可能过期，请刷新' : ''}`, true);
  } finally { if (current === round) refresh.disabled = false; }
}
for (const [id, finished] of [['ongoing', false], ['all', true]]) {
  byId(id).addEventListener('click', () => {
    includeFinished = finished;
    byId('ongoing').setAttribute('aria-pressed', String(!finished)); byId('all').setAttribute('aria-pressed', String(finished)); load();
  });
}
refresh.addEventListener('click', () => load());
next.addEventListener('click', () => { cursor = nextCursor; ++pageNumber; load(false); });
async function close() {
  try { await window.__TAURI_INTERNALS__?.invoke('task_menu_close'); }
  catch { message('关闭失败，请使用窗口关闭按钮重试', true); }
}
byId('close').addEventListener('click', close);
document.addEventListener('keydown', event => { if (event.key === 'Escape') { event.preventDefault(); close(); } });
window.addEventListener('blur', close);
window.addEventListener('yonda-tasks-open', () => { byId('refresh').focus(); load(); });
load();
})();
