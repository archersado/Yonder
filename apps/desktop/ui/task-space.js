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
const focusFailureLabels = { 'permission-unavailable':'缺少辅助功能权限', 'process-changed':'目标进程已变化', 'window-missing':'目标窗口不存在', 'mapping-not-unique':'无法唯一识别目标窗口', 'activation-failed':'窗口前置失败', 'verification-failed':'前置结果未通过核验', 'geometry-changed':'窗口身份已变化', 'reference-unavailable':'工作引用不可用' };
const artifactAvailabilityLabels = { available:'可用', missing:'缺失', changed:'已变化', unverified:'未验证' };
const fileGrantPurposeLabels = { read:'读取', 'create-new':'新建', replace:'替换', trash:'移至回收站' };
let includeFinished = false, cursor = null, nextCursor = null, pageNumber = 1;
let round = 0, selection = 0;
const pendingControls = new Map();
const confirmationIds = new Map();

async function query(method, params) {
  const invoke = window.__TAURI_INTERNALS__?.invoke;
  if (!invoke) throw new Error('任务查询能力未提供');
  const request = JSON.stringify({ jsonrpc: '2.0', id: crypto.randomUUID(), method,
    params: { agent_id: 'desktop', capability: method === 'task.cancel' ? 'task.cancel' : method === 'task.control' ? 'task.control' : method === 'task.result.confirm' ? 'task.result.confirm' : 'task.read', deadline: Date.now() + 10000, ...params } });
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
async function confirmResult(task, button, textarea) {
  button.disabled = true; button.textContent = '正在确认…';
  const confirmationId = confirmationIds.get(task.task_id) ?? crypto.randomUUID();
  confirmationIds.set(task.task_id, confirmationId);
  const comment = textarea.value.trim() || null;
  try {
    await window.__TAURI_INTERNALS__.invoke('task_confirm', {
      taskId: task.task_id,
      expectedSequence: task.sequence,
      confirmationId,
      comment
    });
    await load();
    message('结果已确认，任务终态未改变');
  } catch (error) {
    message(`结果确认失败：${error.message ?? '请刷新后重试'}`, true);
    button.disabled = false; button.textContent = '确认结果';
  }
}
async function chooseFileGrant(task, purpose, button) {
  button.disabled = true; button.textContent = '正在选择…';
  try {
    const result = await window.__TAURI_INTERNALS__.invoke('file_grant_choose', { taskId: task.task_id, purpose });
    if (result == null) {
      message('未签发文件授权');
    } else if (!fileGrantPurposeLabels[result.purpose] || !Number.isSafeInteger(result.expiresAtMs)) {
      throw new Error('文件授权响应不可用');
    } else {
      message('文件授权已签发，归属 Agent 可读取授权引用');
    }
    await select(task, document.querySelector(`button.task[data-task-id="${CSS.escape(task.task_id)}"]`));
  } catch (error) {
    message(`文件授权失败：${error.message ?? '请重试'}`, true);
    button.disabled = false; button.textContent = `授权${fileGrantPurposeLabels[purpose] ?? '文件'}`;
  }
}
async function revokeFileGrant(task, grant, button) {
  button.disabled = true; button.textContent = '正在撤销…';
  try {
    await window.__TAURI_INTERNALS__.invoke('file_grant_revoke', { taskId: task.task_id, grantId: grant.grantId });
    message('文件授权已撤销');
    await select(task, document.querySelector(`button.task[data-task-id="${CSS.escape(task.task_id)}"]`));
  } catch (error) {
    message(`撤销失败：${error.message ?? '请重试'}`, true);
    button.disabled = false; button.textContent = '撤销';
  }
}
function renderFileGrants(container, task, result) {
  const heading = document.createElement('h3'); heading.textContent = '文件授权'; container.append(heading);
  const description = document.createElement('p'); description.className = 'file-grant-note';
  description.textContent = '授权有效期最长 15 分钟。选择的文件位置不会显示在此处。'; container.append(description);
  const terminal = ['completed', 'failed', 'cancelled'].includes(task.status);
  if (terminal) {
    const unavailable = document.createElement('p'); unavailable.className = 'file-grant-empty'; unavailable.textContent = '终态任务不能新增或查看文件授权'; container.append(unavailable); return;
  }
  const actions = document.createElement('div'); actions.className = 'file-grant-actions';
  for (const [purpose, text] of [['read','授权读取'], ['create-new','授权新建'], ['replace','授权替换'], ['trash','授权移至回收站']]) {
    const button = document.createElement('button'); button.textContent = text;
    button.addEventListener('click', () => chooseFileGrant(task, purpose, button)); actions.append(button);
  }
  container.append(actions);
  if (result.status === 'rejected') {
    const failure = document.createElement('p'); failure.className = 'file-grant-error'; failure.textContent = `文件授权读取失败：${result.reason?.message ?? '请刷新后重试'}`; container.append(failure); return;
  }
  const grants = result.value;
  if (!Array.isArray(grants) || grants.some(grant => !grant || typeof grant.grantId !== 'string' || !fileGrantPurposeLabels[grant.purpose] || !Number.isSafeInteger(grant.expiresAtMs))) {
    const failure = document.createElement('p'); failure.className = 'file-grant-error'; failure.textContent = '文件授权响应不可用'; container.append(failure); return;
  }
  if (!grants.length) {
    const empty = document.createElement('p'); empty.className = 'file-grant-empty'; empty.textContent = '暂无有效文件授权'; container.append(empty); return;
  }
  const list = document.createElement('ul'); list.className = 'file-grants';
  for (const grant of grants) {
    const item = document.createElement('li'), text = document.createElement('span'), revoke = document.createElement('button');
    text.textContent = `${fileGrantPurposeLabels[grant.purpose]} · 至 ${new Date(grant.expiresAtMs).toLocaleTimeString()}`;
    revoke.textContent = '撤销'; revoke.addEventListener('click', () => revokeFileGrant(task, grant, revoke)); item.append(text, revoke); list.append(item);
  }
  container.append(list);
}
async function loadCommandApprovals(task) {
  const summaries = await window.__TAURI_INTERNALS__.invoke('command_approval_list', { taskId: task.task_id });
  if (!Array.isArray(summaries) || summaries.some(item => !item || typeof item.commandId !== 'string' || !['awaiting-user', 'approved'].includes(item.state) || !Number.isSafeInteger(item.expiresAtMs))) {
    throw new Error('命令批准列表响应不可用');
  }
  return Promise.all(summaries.map(async summary => {
    const preview = await window.__TAURI_INTERNALS__.invoke('command_approval_preview', { taskId: task.task_id, commandId: summary.commandId });
    if (!preview || preview.commandId !== summary.commandId || typeof preview.program !== 'string' || !Array.isArray(preview.args) || preview.args.some(value => typeof value !== 'string') || typeof preview.cwd !== 'string' || !preview.env || Array.isArray(preview.env) || typeof preview.env !== 'object' || !Number.isSafeInteger(preview.timeoutMs) || preview.expiresAtMs !== summary.expiresAtMs) {
      throw new Error('命令批准预览响应不可用');
    }
    return { ...summary, preview };
  }));
}
async function decideCommandApproval(task, approval, decision, button) {
  button.disabled = true;
  const approving = decision === 'approve';
  button.textContent = approving ? '正在批准…' : '正在拒绝…';
  try {
    await window.__TAURI_INTERNALS__.invoke(`command_approval_${decision}`, { taskId: task.task_id, commandId: approval.commandId });
    message(approving ? '命令已批准，仅归属 Agent 可执行一次' : '命令提议已拒绝');
    await select(task, document.querySelector(`button.task[data-task-id="${CSS.escape(task.task_id)}"]`));
  } catch (error) {
    message(`${approving ? '批准' : '拒绝'}失败：${error.message ?? '请刷新后重试'}`, true);
    button.disabled = false;
    button.textContent = approving ? '批准执行一次' : approval.state === 'approved' ? '撤销批准' : '拒绝';
  }
}
function appendCommandField(container, label, value) {
  const row = document.createElement('div'), name = document.createElement('strong'), content = document.createElement('code');
  name.textContent = label; content.textContent = value; row.append(name, content); container.append(row);
}
function renderCommandApprovals(container, task, result) {
  const heading = document.createElement('h3'); heading.textContent = '命令批准'; container.append(heading);
  const note = document.createElement('p'); note.className = 'command-approval-note';
  note.textContent = 'Agent 请求执行结构化命令。请核对程序、参数、目录和环境变量；批准后仅可执行一次。'; container.append(note);
  if (result.status === 'rejected') {
    const failure = document.createElement('p'); failure.className = 'command-approval-error'; failure.textContent = `命令批准读取失败：${result.reason?.message ?? '请刷新后重试'}`; container.append(failure); return;
  }
  const approvals = result.value;
  if (!approvals.length) {
    const empty = document.createElement('p'); empty.className = 'command-approval-empty'; empty.textContent = '暂无待处理命令'; container.append(empty); return;
  }
  const list = document.createElement('div'); list.className = 'command-approvals';
  for (const approval of approvals) {
    const card = document.createElement('section'); card.className = 'command-approval-card';
    const state = document.createElement('p'); state.className = 'command-approval-state';
    state.textContent = approval.state === 'approved' ? '已批准，等待归属 Agent 执行' : '等待本机用户决定'; card.append(state);
    const fields = document.createElement('div'); fields.className = 'command-approval-fields';
    appendCommandField(fields, '程序', approval.preview.program);
    appendCommandField(fields, '参数', approval.preview.args.length ? approval.preview.args.map(value => JSON.stringify(value)).join(' ') : '（无）');
    appendCommandField(fields, '工作目录', approval.preview.cwd);
    const environment = Object.entries(approval.preview.env).map(([key, value]) => `${key}=${JSON.stringify(value)}`).join('\n');
    appendCommandField(fields, '环境变量', environment || '（无）');
    appendCommandField(fields, '超时', `${approval.preview.timeoutMs} ms`);
    appendCommandField(fields, '有效期', new Date(approval.expiresAtMs).toLocaleTimeString());
    card.append(fields);
    const actions = document.createElement('div'); actions.className = 'command-approval-actions';
    if (approval.state === 'awaiting-user') {
      const approve = document.createElement('button'); approve.textContent = '批准执行一次'; approve.addEventListener('click', () => decideCommandApproval(task, approval, 'approve', approve)); actions.append(approve);
    }
    const reject = document.createElement('button'); reject.textContent = approval.state === 'approved' ? '撤销批准' : '拒绝'; reject.addEventListener('click', () => decideCommandApproval(task, approval, 'reject', reject)); actions.append(reject);
    card.append(actions); list.append(card);
  }
  container.append(list);
}
function timelineText(event) {
  if (event.creation_event) return `任务创建：${sourceLabels[event.creation_event.source] ?? '来源未知'} · Agent ${event.creation_event.owner_agent_id}`;
  if (event.focus_event) {
    if (event.focus_event.phase === 'locating') return '接管：正在定位任务工作';
    if (event.focus_event.phase === 'focused') return '接管：任务工作定位成功';
    return `接管：定位失败 · ${focusFailureLabels[event.focus_event.failure] ?? '原因未提供'}`;
  }
  if (event.control_event) {
    const kind = {pause:'暂停',cancel:'取消',takeover:'接管'}[event.control_event.kind] ?? '控制';
    const phase = event.control_event.phase === 'stopped' ? '步骤边界停止已确认' : '停止请求已登记';
    return `${kind}：${phase}（尝试 ${event.control_event.attempt_id}）`;
  }
  if (event.step_declaration) return `Agent 声明步骤：${event.step_declaration.label}`;
  if (event.attempt_started) return `执行尝试已准备（步骤 ${event.attempt_started.step_id} · 尝试 ${event.attempt_started.attempt_id}）`;
  if (event.observation) {
    const result = {'matched':'已匹配','not-matched':'未匹配','unknown':'未知'}[event.observation.result] ?? '未知';
    return `Observe（步骤 ${event.observation.step_id}）：${result} · ${event.observation.summary}`;
  }
  if (event.attempt_result?.phase === 'unknown') return `执行结果未知：${unknownLabels[event.attempt_result.unknown_reason] ?? '原因未提供'}`;
  if (event.attempt_result?.phase === 'observed') return event.attempt_result.action_succeeded ? '动作已观察：成功' : '动作已观察：未达成';
  if (event.wait_reason) return `等待用户：${event.wait_reason}`;
  if (event.user_confirmation) return `用户确认结果：结果序号 ${event.user_confirmation.result_sequence} · 清单版本 ${event.user_confirmation.manifest_version}`;
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
function appendArtifacts(list, items, after) {
  let cursor = Number(after);
  for (const artifact of items) {
    if (!Number.isInteger(artifact.ordinal) || artifact.ordinal <= cursor || !artifactAvailabilityLabels[artifact.availability]) throw new Error('产物清单顺序无效');
    const item = document.createElement('li'), title = document.createElement('strong'), reference = document.createElement('small');
    title.textContent = `产物 ${artifact.ordinal} · ${artifactAvailabilityLabels[artifact.availability]}`;
    reference.textContent = `引用 ${artifact.reference_id}`;
    item.append(title, reference); list.append(item); cursor = artifact.ordinal;
  }
  return cursor;
}
function offerArtifactMore(container, list, taskId, version, after, expectedCount, current, currentRound) {
  const loaded = list.childElementCount;
  if (loaded >= expectedCount) return;
  const area = document.createElement('div'); area.className = 'artifact-more';
  const hint = document.createElement('p'); hint.textContent = `已加载 ${loaded} / ${expectedCount} 项`;
  const button = document.createElement('button'); button.textContent = '加载更多产物';
  area.append(hint, button); container.append(area);
  button.addEventListener('click', async () => {
    button.disabled = true; button.textContent = '正在加载…';
    area.querySelector('.artifact-error')?.remove();
    try {
      const page = await query('task.artifacts', { task_id: taskId, manifest_version: version, after_ordinal: after, limit: 20 });
      if (current !== selection || currentRound !== round) return;
      if (page.kind !== 'artifact-manifest-page' || page.task_id !== taskId || page.manifest_version !== version || !Array.isArray(page.items) || !page.items.length) throw new Error('后续产物为空，请刷新任务');
      const nextAfter = appendArtifacts(list, page.items, after);
      if (page.next_after_ordinal != null && page.next_after_ordinal !== nextAfter) throw new Error('产物续页游标无效');
      if (page.next_after_ordinal == null && list.childElementCount < expectedCount) throw new Error('产物清单未完整返回');
      area.remove();
      if (page.next_after_ordinal != null) offerArtifactMore(container, list, taskId, version, nextAfter, expectedCount, current, currentRound);
    } catch (error) {
      if (current !== selection || currentRound !== round) return;
      button.disabled = false; button.textContent = '重试加载产物';
      const failure = document.createElement('p'); failure.className = 'artifact-error'; failure.textContent = `产物读取失败：${error.message ?? '请重试'}`; area.append(failure);
    }
  });
}
function renderArtifacts(container, task, pageResult, current, currentRound) {
  const heading = document.createElement('h3'); heading.textContent = '产物'; container.append(heading);
  const manifest = task.artifact_manifest;
  if (!manifest) {
    const empty = document.createElement('p'); empty.className = 'artifact-empty'; empty.textContent = '暂无产物清单'; container.append(empty); return;
  }
  if (pageResult.status === 'rejected') {
    const error = document.createElement('div'); error.className = 'artifact-more';
    const message = document.createElement('p'); message.className = 'artifact-error'; message.textContent = `产物读取失败：${pageResult.reason?.message ?? '请重试'}`;
    const retry = document.createElement('button'); retry.textContent = '重试读取产物'; retry.addEventListener('click', () => select(task, document.querySelector(`button.task[data-task-id="${CSS.escape(task.task_id)}"]`)));
    error.append(message, retry); container.append(error); return;
  }
  const page = pageResult.value;
  if (page.kind !== 'artifact-manifest-page' || page.task_id !== task.task_id || page.manifest_version !== manifest.version || !Array.isArray(page.items)) throw new Error('产物清单响应不可用');
  if (!page.items.length) {
    if (manifest.item_count !== 0) throw new Error('产物清单未完整返回');
    const empty = document.createElement('p'); empty.className = 'artifact-empty'; empty.textContent = '该版本没有产物'; container.append(empty); return;
  }
  const list = document.createElement('ul'); list.className = 'artifacts';
  const after = appendArtifacts(list, page.items, 0); container.append(list);
  if (page.next_after_ordinal != null && page.next_after_ordinal !== after) throw new Error('产物续页游标无效');
  if (page.next_after_ordinal != null) offerArtifactMore(container, list, task.task_id, manifest.version, after, manifest.item_count, current, currentRound);
  else if (list.childElementCount < manifest.item_count) throw new Error('产物清单未完整返回');
}
async function select(task, button) {
  const current = ++selection, currentRound = round;
  for (const item of tasks.querySelectorAll('button.task')) item.setAttribute('aria-pressed', String(item === button));
  placeholder('正在读取详情…');
  const recentAfter = (BigInt(task.sequence) > 20n ? BigInt(task.sequence) - 20n : 0n).toString();
  const [detailResult, timelineResult, recentResult, browserResult, grantsResult, approvalsResult] = await Promise.allSettled([
    query('task.step.get', { task_id: task.task_id }),
    query('task.events', { task_id: task.task_id, after_sequence: '0', limit: 20 }),
    query('task.events', { task_id: task.task_id, after_sequence: recentAfter, limit: 20 }),
    query('task.browser.get', { task_id: task.task_id }),
    ['completed', 'failed', 'cancelled'].includes(task.status)
      ? Promise.resolve([])
      : window.__TAURI_INTERNALS__.invoke('file_grant_list', { taskId: task.task_id }),
    ['completed', 'failed', 'cancelled'].includes(task.status)
      ? Promise.resolve([])
      : loadCommandApprovals(task)
  ]);
  if (current !== selection || currentRound !== round) return;
  try {
    if (detailResult.status === 'rejected') throw detailResult.reason;
    const result = detailResult.value;
    if (result.kind !== 'step') throw new Error('任务详情响应不可用');
    const artifactResult = result.task.artifact_manifest
      ? await Promise.allSettled([query('task.artifacts', { task_id: result.task.task_id, manifest_version: result.task.artifact_manifest.version, after_ordinal: 0, limit: 20 })]).then(values => values[0])
      : { status: 'fulfilled', value: null };
    if (current !== selection || currentRound !== round) return;
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
    const terminal = ['completed', 'failed'].includes(result.task.status);
    const confirmation = result.task.user_confirmation;
    const audit = terminal
      ? confirmation
        ? `已确认 · 结果序号 ${confirmation.result_sequence} · 清单版本 ${confirmation.manifest_version}${confirmation.comment ? ` · ${confirmation.comment}` : ''}`
        : '结果待确认'
      : '仅终态任务支持确认';
    const manifestChanged = result.task.artifact_manifest && confirmation && BigInt(result.task.artifact_manifest.version) > BigInt(confirmation.manifest_version);
    const manifest = result.task.artifact_manifest
      ? `版本 ${result.task.artifact_manifest.version} · ${result.task.artifact_manifest.item_count} 项${manifestChanged ? ' · 产物已变化，需重新检查' : ''}`
      : terminal ? '确认后生成首个清单' : '不适用';
    const goalVerification = result.task.goal_verification
      ? `${result.task.goal_verification.outcome === 'achieved' ? '目标已核验' : '目标未完成'} · 依据序号 ${result.task.goal_verification.observation_sequence}`
      : '等待慢脑核验最终结果';
    for (const [name, value] of [['任务 ID', result.task.task_id], ['Agent', result.task.owner_agent_id], ['状态', labels[result.task.status] ?? result.task.status], ['状态说明', statusReason], ['序号', result.task.sequence], ['来源', sourceLabels[result.task.source] ?? '来源未知'], ['当前步骤', step?.label ?? '未声明步骤'], ['步骤标识', step ? `${step.step_id} · 接受序号 ${step.accepted_sequence}` : '未提供'], ['观察摘要', observation], ['目标核验', goalVerification], ['下一步意图', result.task.next_intent ?? '未声明意图'], ['等待原因', waitReason], ['浏览器 Task Space', browser], ['结果确认', audit], ['产物清单', manifest]]) {
      const dt = document.createElement('dt'), dd = document.createElement('dd'); dt.textContent = name; dd.textContent = value; dl.append(dt, dd);
      if (name === '浏览器 Task Space' && browserReference?.ownership === 'agent' && !browserReference.finished) {
        const open = document.createElement('button'); open.className = 'browser-open'; open.textContent = '打开 ego-lite';
        open.addEventListener('click', () => openBrowserTaskSpace(result.task, open)); dd.append(document.createElement('br'), open);
      }
    }
    detail.append(dl);
    renderCommandApprovals(detail, result.task, approvalsResult);
    renderFileGrants(detail, result.task, grantsResult);
    if (terminal && !confirmation) {
      const area = document.createElement('div'); area.className = 'confirm';
      const label = document.createElement('label'); label.textContent = '确认意见（可选）';
      const textarea = document.createElement('textarea'); textarea.maxLength = 2048;
      textarea.placeholder = '记录本次结果检查说明'; textarea.setAttribute('aria-label', '结果确认意见');
      const button = document.createElement('button'); button.textContent = '确认结果';
      button.addEventListener('click', () => confirmResult(result.task, button, textarea));
      label.append(textarea); area.append(label, button); detail.append(area);
    }
    try {
      renderArtifacts(detail, result.task, artifactResult, current, currentRound);
    } catch (error) {
      const failure = document.createElement('p'); failure.className = 'artifact-error'; failure.textContent = `产物读取失败：${error.message ?? '请重试'}`; detail.append(failure);
    }
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
    const result = await query('task.list', { after_task_id: cursor, include_finished: includeFinished, running_only: !includeFinished, newest_first: true, limit: 20 });
    if (current !== round) return;
    if (result.kind !== 'tasks' || !Array.isArray(result.tasks)) throw new Error('任务列表响应不可用');
    const visibleTasks = result.tasks;
    tasks.replaceChildren();
    for (const task of visibleTasks) {
      if (task.status !== 'running') pendingControls.delete(task.task_id);
      const pending = pendingControls.has(task.task_id);
      const li = document.createElement('li'), button = document.createElement('button');
      button.className = 'task'; button.dataset.taskId = task.task_id; button.setAttribute('aria-pressed', 'false');
      const title = document.createElement('strong'), agent = document.createElement('small'), status = document.createElement('span');
      title.textContent = task.name || "未命名历史任务";
      title.title = title.textContent; agent.textContent = `Agent · ${task.owner_agent_id}`;
      status.className = 'status'; status.textContent = labels[task.status] ?? task.status;
      button.append(title, agent, status); button.addEventListener('click', () => select(task, button)); li.append(button);
      const actions = document.createElement('div'); actions.className = 'task-actions';
      actions.append(
        action(task, pending ? '正在停止…' : '接管', pending ? '停止请求已登记' : task.status === 'running' ? '' : '仅执行中的任务支持接管', (task,button)=>controlTask(task,'takeover',button)),
        action(task, '取消任务', ['completed','failed','cancelled'].includes(task.status) ? '终态任务不能再次取消' : '', cancelTask)
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
window.addEventListener('yonda-tasks-open', event => {
  // Agent 创建后的自动透出不能夺走当前输入焦点，否则本窗口的 blur 收起策略会立刻把它关闭。
  if (event.detail?.automatic) {
    // 新建任务在首次执行前是 created；若沿用“进行中”筛选，面板虽已展开但
    // 新任务仍不可见。自动透出切到全部，手动打开继续保留用户当前筛选。
    includeFinished = true;
    byId('ongoing').setAttribute('aria-pressed', 'false');
    byId('all').setAttribute('aria-pressed', 'true');
  } else {
    byId('refresh').focus();
  }
  load();
});
load();
})();
