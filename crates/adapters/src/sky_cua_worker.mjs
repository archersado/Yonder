import { createInterface } from 'node:readline';
import { execFile, spawn } from 'node:child_process';
import { readFile, readdir, realpath, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);
const evidence = process.argv[3];
const bridgePath = process.argv[2];
const supported = new Set([
  'click', 'drag', 'perform_secondary_action', 'press_key', 'scroll',
  'select_text', 'set_value', 'type_text',
]);
const navigationActionTimeoutMs = 5_000;
let pendingInput;

class SkyMcpBridge {
  constructor(path) {
    this.nextId = 1;
    this.pending = new Map();
    this.activeApprovalAppId = null;
    this.fullStateOption = null;
    this.process = spawn(path, ['mcp'], { stdio: ['pipe', 'pipe', 'ignore'] });
    this.lines = createInterface({ input: this.process.stdout, crlfDelay: Infinity });
    this.lines.on('line', line => this.receive(line));
    this.process.once('exit', () => this.failAll(new Error('MCP bridge exited')));
    this.process.once('error', error => this.failAll(error));
  }

  receive(line) {
    if (line.length > 32 * 1024 * 1024) return this.failAll(new Error('MCP response too large'));
    let message;
    try { message = JSON.parse(line); } catch { return this.failAll(new Error('MCP response invalid')); }
    if (typeof message.method === 'string' && message.id != null) {
      const params = message.params;
      const accepted = message.method === 'elicitation/create'
        && typeof this.activeApprovalAppId === 'string'
        && params?.requestedSchema?.type === 'object'
        && Object.keys(params?.requestedSchema?.properties ?? {}).length === 0;
      this.process.stdin.write(`${JSON.stringify({
        jsonrpc: '2.0', id: message.id,
        result: accepted ? { action: 'accept', content: {} } : { action: 'decline' },
      })}\n`);
      return;
    }
    const pending = this.pending.get(message.id);
    if (!pending) return;
    this.pending.delete(message.id);
    clearTimeout(pending.timer);
    if (message.error) pending.reject(new Error('MCP request failed'));
    else pending.resolve(message.result);
  }

  failAll(error) {
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(error);
    }
    this.pending.clear();
  }

  request(method, params, timeoutMs = 25_000) {
    if (this.process.exitCode != null || !this.process.stdin.writable) return Promise.reject(new Error('MCP bridge unavailable'));
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error('MCP request timed out'));
      }, timeoutMs);
      this.pending.set(id, { resolve, reject, timer });
      this.process.stdin.write(`${JSON.stringify({ jsonrpc: '2.0', id, method, params })}\n`, error => {
        if (!error) return;
        clearTimeout(timer);
        this.pending.delete(id);
        reject(error);
      });
    });
  }

  notify(method, params) {
    this.process.stdin.write(`${JSON.stringify({ jsonrpc: '2.0', method, params })}\n`);
  }

  async start() {
    const result = await this.request('initialize', {
      protocolVersion: '2025-06-18', capabilities: { elicitation: {} }, clientInfo: { name: 'Yonder', version: '0.1.0' },
    }, 5_000);
    if (result?.protocolVersion !== '2025-06-18') throw new Error('MCP protocol mismatch');
    this.notify('notifications/initialized', {});
    const catalog = await this.request('tools/list', {}, 5_000);
    const stateTool = catalog?.tools?.find(tool => tool?.name === 'get_app_state');
    if (!stateTool) throw new WorkerFailure('transport-schema-state-tool');
    const properties = stateTool?.inputSchema?.properties ?? {};
    this.fullStateOption = Object.hasOwn(properties, 'disableDiff')
      ? 'disableDiff'
      : Object.hasOwn(properties, 'disable_diff')
        ? 'disable_diff'
        : null;
  }

  async callTool(name, args, approvalAppId = null, timeoutMs = 25_000) {
    this.activeApprovalAppId = approvalAppId;
    try {
      const result = await this.request('tools/call', { name, arguments: args }, timeoutMs);
      if (!result || !Array.isArray(result.content)) throw new Error('MCP tool failed');
      if (result.isError === true) {
        const failure = resultText(result).toLocaleLowerCase();
        if (failure.includes('cgwindownotfound') || failure.includes('error -10005')) {
          throw new WorkerFailure('target-window-unavailable');
        }
        if (failure.includes('sender process is not authenticated')) {
          throw new WorkerFailure('transport-authentication');
        }
        if (failure.includes('timed out')) {
          throw new WorkerFailure('transport-timeout');
        }
        throw new Error('MCP tool failed');
      }
      return result;
    } finally {
      this.activeApprovalAppId = null;
    }
  }

  close() {
    this.process.stdin.end();
    this.process.kill();
  }

  stateArguments(app) {
    return this.fullStateOption ? { app, [this.fullStateOption]: true } : { app };
  }
}

class WorkerFailure extends Error {
  constructor(stage) {
    super(stage);
    this.stage = stage;
  }
}

function boundedFailureStage(error, fallback) {
  if (error instanceof WorkerFailure) return error.stage;
  const messages = [];
  let current = error;
  for (let depth = 0; depth < 4 && current instanceof Error; depth += 1) {
    messages.push(current.message);
    current = current.cause;
  }
  const message = messages.join(' ').toLocaleLowerCase();
  if (message.includes('api version mismatch') || message.includes('incompatible')) return 'transport-version-mismatch';
  if (message.includes('permission denied') || message.includes('operation not permitted')) return 'transport-permission';
  if (message.includes('closed before response')) return 'transport-closed';
  if (message.includes('connection refused')) return 'transport-refused';
  if (message.includes('native pipe')) return 'transport-native-pipe';
  if (message.includes('service startup')) return 'transport-service-startup';
  if (message.includes('policy')) return 'target-policy';
  if (message.includes('approved') || message.includes('approval')) return 'target-approval';
  if (message.includes('timed out')) return 'transport-timeout';
  return fallback;
}

let bridge;
let bridgeFailureStage = null;
try {
  bridge = new SkyMcpBridge(bridgePath);
  await bridge.start();
} catch (error) {
  bridgeFailureStage = boundedFailureStage(error, 'transport-mcp-handshake');
}

async function refreshBridge() {
  bridge?.close();
  const fresh = new SkyMcpBridge(bridgePath);
  try {
    await fresh.start();
  } catch (error) {
    fresh.close();
    throw new WorkerFailure(boundedFailureStage(error, 'transport-mcp-handshake'));
  }
  bridge = fresh;
}

function resultText(result) {
  return result.content
    .filter(block => block?.type === 'text' && typeof block.text === 'string')
    .map(block => block.text)
    .join('\n');
}

function structuredResult(result) {
  if (result?.structuredContent && typeof result.structuredContent === 'object') return result.structuredContent;
  const text = resultText(result).trim();
  const fenced = text.match(/^```(?:json)?\s*([\s\S]*?)\s*```$/i);
  const payload = fenced?.[1] ?? text;
  const candidates = [payload];
  const objectStart = payload.indexOf('{');
  const objectEnd = payload.lastIndexOf('}');
  if (objectStart >= 0 && objectEnd > objectStart) candidates.push(payload.slice(objectStart, objectEnd + 1));
  for (const candidate of candidates) {
    if (!candidate.startsWith('{') && !candidate.startsWith('[') && !candidate.startsWith('"')) continue;
    try { return JSON.parse(candidate); } catch { /* 尝试下一个有界候选。 */ }
  }
  return null;
}

async function listApps() {
  const result = await bridge.callTool('list_apps', {});
  const structured = structuredResult(result);
  const apps = Array.isArray(structured) ? structured : structured?.apps ?? resultText(result).split('\n').flatMap(line => {
    const match = line.match(/^(.+?) — (.+?) — ([A-Za-z0-9.-]+)(?: \[([^\]]*)\])?$/);
    if (!match) return [];
    return [{ displayName: match[1].trim(), path: match[2].replace(/\/$/, ''), id: match[3], isRunning: match[4]?.split(', ').includes('running') === true }];
  });
  if (!Array.isArray(apps)) throw new Error('MCP apps response invalid');
  return apps;
}

function stateFromResult(result, app) {
  const structured = structuredResult(result);
  const image = result.content.find(block => block?.type === 'image'
    && typeof block.data === 'string' && typeof block.mimeType === 'string');
  const screenshot = image ? { url: `data:${image.mimeType};base64,${image.data}` } : structured?.screenshot;
  const resourceText = result.content.find(block => block?.type === 'resource'
    && typeof block.resource?.text === 'string')?.resource?.text;
  const text = typeof structured === 'string'
    ? structured
    : typeof structured?.text === 'string'
    ? structured.text
    : structured?.ax_tree != null
      ? String(structured.ax_tree)
      : resourceText ?? resultText(result);
  return { app: structured?.app ?? app, text, screenshot };
}

async function getAppState(app, applicationId) {
  // CU-S4要求每一步都基于新鲜transcript。只有签名Client的公开Schema支持时
  // 才关闭diff；0.7.1的完整AX树来自嵌入资源，不能用未公开字段绕过Schema。
  return stateFromResult(await bridge.callTool(
    'get_app_state', bridge.stateArguments(app), applicationId,
  ), app);
}

async function callAction(name, args, applicationId, timeoutMs = 25_000) {
  await bridge.callTool(name, args, applicationId, timeoutMs);
}

function responseFor(request) {
  return {
    task_id: request.task_id,
    step_id: request.step_id,
    attempt_id: request.attempt_id,
    worker_instance_id: request.worker_instance_id,
    host_session_id: request.host_session_id,
    action_known: false,
    action_succeeded: false,
    action_effect: null,
    observe_valid: false,
    element_count: 0,
    screenshot_path: null,
    screenshot_mime: null,
    target_visible: null,
    launched_app_id: null,
    failure_stage: 'target',
  };
}

async function macAppForPid(pid) {
  const { stdout } = await execFileAsync('/bin/ps', ['-p', String(pid), '-o', 'command='], {
    encoding: 'utf8', timeout: 2_000, maxBuffer: 16 * 1024,
  });
  const command = stdout.trim();
  const match = command.match(/^(.+?\.app)(?:\/|$)/);
  if (!match) throw new Error('pid is not owned by an app bundle');
  return match[1];
}

async function runningMacAppForBundleId(bundleId) {
  let stdout;
  try {
    ({ stdout } = await execFileAsync('/bin/ps', ['-axo', 'command='], {
      encoding: 'utf8', timeout: 2_000, maxBuffer: 1024 * 1024,
    }));
  } catch {
    throw new WorkerFailure('target-running-processes');
  }
  const rawPaths = [...new Set(stdout.split('\n').flatMap(command => {
    const match = command.trim().match(/^(.+?\.app)(?:\/|$)/);
    return match ? [match[1]] : [];
  }))];
  const paths = [...new Set((await Promise.all(rawPaths.map(path => realpath(path).catch(() => null))))
    .filter(path => typeof path === 'string'))];
  const matches = [];
  for (const path of paths) {
    try {
      const { stdout: identifier } = await execFileAsync(
        '/usr/bin/plutil',
        ['-extract', 'CFBundleIdentifier', 'raw', '-o', '-', join(path, 'Contents', 'Info.plist')],
        { encoding: 'utf8', timeout: 2_000, maxBuffer: 4 * 1024 },
      );
      if (identifier.trim() === bundleId) matches.push(path);
    } catch {
      // 进程可能在枚举后立即退出；忽略该候选并继续收敛。
    }
  }
  if (matches.length !== 1) throw new WorkerFailure('target-running-identity');
  return matches[0];
}

function canonicalListedApp(matches) {
  if (matches.length === 1) return matches[0];
  const running = matches.filter(candidate => candidate.isRunning === true);
  if (running.length === 1) return running[0];
  const installed = matches.filter(candidate => typeof candidate.path === 'string'
    && (candidate.path.startsWith('/Applications/') || candidate.path.startsWith('/System/Applications/')));
  return installed.length === 1 ? installed[0] : null;
}

const trustedApplicationRoots = [
  '/Applications',
  '/System/Applications',
  '/System/Applications/Utilities',
];

function validBundleId(value) {
  return typeof value === 'string'
    && value.length > 0
    && value.length <= 255
    && /^[A-Za-z0-9.-]+$/.test(value);
}

async function installedMacAppsForBundleId(bundleId) {
  if (!validBundleId(bundleId)) return [];
  const matches = [];
  for (const root of trustedApplicationRoots) {
    let entries;
    try {
      entries = await readdir(root, { withFileTypes: true });
    } catch {
      continue;
    }
    for (const entry of entries) {
      if (!entry.isDirectory() || !entry.name.endsWith('.app')) continue;
      const listedPath = join(root, entry.name);
      let canonicalPath;
      try {
        canonicalPath = await realpath(listedPath);
      } catch {
        continue;
      }
      if (canonicalPath !== listedPath || !canonicalPath.startsWith(`${root}/`)) continue;
      try {
        const { stdout } = await execFileAsync(
          '/usr/bin/plutil',
          ['-extract', 'CFBundleIdentifier', 'raw', '-o', '-', join(canonicalPath, 'Contents', 'Info.plist')],
          { encoding: 'utf8', timeout: 2_000, maxBuffer: 4 * 1024 },
        );
        if (stdout.trim() === bundleId) {
          matches.push({ id: bundleId, path: canonicalPath, isRunning: false });
        }
      } catch {
        // 固定应用根中的损坏或无标识包不是可启动候选。
      }
    }
  }
  return [...new Map(matches.map(candidate => [candidate.path, candidate])).values()];
}

async function bindRunningTarget(target) {
  try {
    return { ...target, app: await runningMacAppForBundleId(target.applicationId) };
  } catch {
    // 某些隔离夹具和未发生进程切换的普通应用没有可枚举的新实例；此时保留
    // Sky 已经验证过的完整路径。正式 macOS App Translocation 实例会在这里
    // 收敛到唯一运行路径，后续动作不再用安装源路径或歧义 bundle id。
    return target;
  }
}

async function resolveBoundTarget(applicationId, fallbackApp = null) {
  if (!validBundleId(applicationId)) throw new WorkerFailure('target-bound-identity');
  try {
    return { app: await runningMacAppForBundleId(applicationId), applicationId };
  } catch {
    let apps;
    try {
      apps = await listApps();
    } catch (error) {
      throw new WorkerFailure(boundedFailureStage(error, 'target-list-apps'));
    }
    const selected = canonicalListedApp(apps.filter(candidate => candidate.id === applicationId));
    const listedPath = typeof selected?.path === 'string' && selected.path.trim() !== ''
      ? selected.path.trim()
      : null;
    if (listedPath) return { app: listedPath, applicationId };
    if (typeof fallbackApp === 'string' && fallbackApp.trim() !== '') {
      return { app: fallbackApp.trim(), applicationId };
    }
    throw new WorkerFailure('target-running-identity');
  }
}

async function targetFor(request) {
  if (request.tool_name === 'launch_app') {
    const app = request.arguments.app ?? request.arguments.bundle_id ?? request.arguments.path;
    if (typeof app !== 'string' || app.trim() === '') {
      throw new Error('launch target is unavailable');
    }
    const requested = app.trim();
    let apps;
    try {
      apps = await listApps();
    } catch (error) {
      throw new WorkerFailure(boundedFailureStage(error, 'target-list-apps'));
    }
    const matches = apps.filter(candidate => candidate.id === requested || candidate.displayName === requested);
    let selected = canonicalListedApp(matches);
    // 官方 list_apps 是近期/运行应用目录，不保证枚举所有已安装应用。只有调用方
    // 给出合法 bundle id 且列表完全未命中时，才在三个固定系统应用根按 plist
    // 唯一解析；不搜索全盘、不接受环境覆盖，也不以显示名猜测目标。
    if (!selected && matches.length === 0 && validBundleId(requested)) {
      selected = canonicalListedApp(await installedMacAppsForBundleId(requested));
    }
    if (!selected || typeof selected.id !== 'string' || selected.id === '') {
      throw new WorkerFailure('target-canonical-app');
    }
    const applicationId = selected.id;
    // Sky 的动作工具要求 app 能唯一解析。即使应用尚未运行，list_apps 返回的
    // 完整路径也比 bundle id 更稳定：同一 bundle id 可能同时存在于 DMG、
    // App Translocation 和 /Applications，启动后再使用 bundle id 会变成歧义目标。
    const listedPath = typeof selected.path === 'string' && selected.path.trim() !== ''
      ? selected.path.trim()
      : null;
    const resolvedApp = listedPath
      ?? (selected.isRunning === true ? await runningMacAppForBundleId(applicationId) : applicationId);
    return { app: resolvedApp, applicationId };
  }
  if (request.bound_application_id != null) return resolveBoundTarget(request.bound_application_id);
  if (request.tool_name === 'bring_to_front') {
    throw new WorkerFailure('target-bound-identity');
  }
  return { app: await macAppForPid(request.pid) };
}

async function observe(target) {
  try {
    return await getAppState(target.app, target.applicationId);
  } catch (error) {
    // 页面/窗口切换会让官方 MCP Client 的只读窗口引用失效。只在 Observe
    // 阶段重建签名 Client 并重读一次；动作调用从不在未知结果后自动重试。
    if (!(error instanceof WorkerFailure) || error.stage !== 'target-window-unavailable') throw error;
    await refreshBridge();
    const refreshed = await resolveBoundTarget(target.applicationId, target.app);
    return getAppState(refreshed.app, refreshed.applicationId);
  }
}

function stateText(state) {
  if (typeof state.text === 'string') return state.text;
  if (state.ax_tree != null) return String(state.ax_tree);
  return '';
}

function screenshotUrl(state) {
  const screenshot = state.screenshot ?? state.screenshots?.[0];
  return screenshot?.url ?? screenshot?.data_url ?? null;
}

function fingerprint(state) {
  return `${stateText(state)}\n${screenshotUrl(state) ?? ''}`;
}

function transcriptElements(state) {
  const elements = [];
  let text = stateText(state);
  if (!text.includes('\n') && text.includes('\\n')) {
    text = text.replaceAll('\\n', '\n').replaceAll('\\t', '\t');
  }
  for (const line of text.split('\n')) {
    const match = line.match(/^[\s│├└─>]*(?:[-*]\s*)?(?:\[(\d+)\]|(\d+))[.):]?\s+(.+)$/);
    if (match) elements.push({ index: Number(match[1] ?? match[2]), text: match[3].trim() });
  }
  return elements;
}

function uniqueElement(state, predicate) {
  const matches = transcriptElements(state).filter(element => predicate(element.text.toLocaleLowerCase()));
  return matches.length === 1 ? matches[0] : null;
}

function searchField(state) {
  return uniqueElement(state, text => {
    const editable = text.includes('文本框') || text.includes('text field') || text.includes('textfield') || text.includes('search field');
    const search = text.includes('搜索') || text.includes('search');
    return editable && search;
  });
}

function firstMatchingControl(state, query) {
  const normalize = value => value.normalize('NFKC').toLocaleLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, ' ').trim();
  const normalized = normalize(query);
  if (normalized === '') return null;
  const field = searchField(state);
  const elements = transcriptElements(state).filter(element => element.index !== field?.index);
  const direct = elements.find(element => normalize(element.text).includes(normalized));
  if (direct) return direct;
  const queryTokens = normalized.split(' ');
  for (let index = 0; index < elements.length; index += 1) {
    const group = elements.slice(index, index + 3);
    const tokens = normalize(group.map(element => element.text).join(' ')).split(' ');
    let cursor = 0;
    for (const token of tokens) {
      if (token === queryTokens[cursor]) cursor += 1;
    }
    if (cursor === queryTokens.length) return group[0];
  }
  return null;
}

async function persistScreenshot(state, request, response) {
  const url = screenshotUrl(state);
  if (typeof url !== 'string') return;
  const match = url.match(/^data:(image\/(?:png|jpeg|webp));base64,([A-Za-z0-9+/=]+)$/);
  let mime;
  let bytes;
  if (match) {
    mime = match[1];
    bytes = Buffer.from(match[2], 'base64');
  } else if (url.startsWith('file://')) {
    const path = fileURLToPath(url);
    if (!path.endsWith('.png')) return;
    mime = 'image/png';
    bytes = await readFile(path);
  } else {
    return;
  }
  if (bytes.length === 0 || bytes.length > 4 * 1024 * 1024) return;
  const extension = mime === 'image/png' ? 'png' : mime === 'image/jpeg' ? 'jpg' : 'webp';
  const path = join(evidence, `${request.task_id}-${request.attempt_id}.${extension}`);
  await writeFile(path, bytes, { flag: 'wx', mode: 0o600 });
  response.screenshot_path = path;
  response.screenshot_mime = mime;
}

function actionArguments(request, target) {
  const args = { ...request.arguments, ...target };
  delete args.pid;
  delete args.window_id;
  delete args.app;
  delete args.applicationId;
  delete args.observed_element_index;
  delete args._yonder_action_kind;
  delete args._yonder_private_text;
  if (target.app) args.app = target.app;
  return args;
}

function privateText(request) {
  const value = request.arguments._yonder_private_text ?? request.arguments.text;
  return typeof value === 'string' && value.length <= 4096 ? value : null;
}

async function perform(request, target, before) {
  if (request.tool_name === 'launch_app') {
    // macOS Sky 将启动封装在 get_app_state 中，且不会把应用抢到前台。
    // 已运行应用的窗口代次可能在宿主重启后失效；启动同样属于只读Observe，
    // 复用一次有界Client刷新，不重放任何键鼠或其他副作用动作。
    await observe(target);
    pendingInput = undefined;
    return;
  }
  if (request.tool_name === 'bring_to_front') {
    // Yonder WindowActivationPort 已执行并验证原生 AX raise。原生置前会改变
    // WindowServer 的 key-window 绑定；旧的签名 MCP Client 仍可能保留置前前
    // 的截图坐标映射。此处只重建 Client 并由后续 Observe 重新绑定窗口，绝不
    // 重试已经完成的置前或任何副作用动作。
    await refreshBridge();
    return;
  }
  const semantic = request.arguments._yonder_action_kind;
  if (semantic === 'activate-control' && request.tool_name === 'click'
      && Number.isInteger(request.arguments.observed_element_index)) {
    const index = request.arguments.observed_element_index;
    if (index < 1 || index > 65_535
        || !transcriptElements(before).some(element => element.index === index)) {
      throw new WorkerFailure('target-semantic-element');
    }
    await callAction('click', { app: target.app, element_index: String(index) }, target.applicationId);
    return { kind: 'transcript-changed' };
  }
  if (semantic === 'activate-control' && request.tool_name === 'click'
      && Number.isFinite(request.arguments.x) && Number.isFinite(request.arguments.y)) {
    const clickArguments = actionArguments(request, target);
    // Sky 的 MCP Schema虽把这两个字段标为缺省值，但正式Client对自绘控件
    // 可能只完成指针移动并悬挂，未形成应用可接收的按下/抬起。Yonder的
    // 通用控件激活固定为左键，且协议只允许1/2次点击；Adapter在调用边界
    // 显式归一化，避免把SDK的可选字段差异泄漏到外部协议。
    clickArguments.mouse_button = 'left';
    clickArguments.click_count ??= 1;
    let timedOut = false;
    try {
      await callAction('click', clickArguments, target.applicationId, navigationActionTimeoutMs);
    } catch (error) {
      const stage = error instanceof WorkerFailure
        ? error.stage
        : boundedFailureStage(error, 'action-coordinate-click');
      if (stage !== 'transport-timeout') throw error;
      // 只对无业务副作用的控件激活收敛“已投递但Client不返回”。旧Client
      // 立即销毁，点击绝不重放；后续统一由新Client只读Observe并交回。
      await refreshBridge();
      timedOut = true;
    }
    return { kind: timedOut ? 'coordinate-timeout' : 'transcript-changed' };
  }
  if (request.tool_name === 'click' && ['focus-target-search', 'focus-control'].includes(semantic)) {
    const element = searchField(before);
    if (!element) throw new WorkerFailure('target-semantic-element');
    await callAction('click', { app: target.app, element_index: String(element.index) }, target.applicationId);
    return { kind: 'element-present', index: element.index };
  }
  if (['enter-target-query', 'input-text'].includes(semantic)) {
    const element = searchField(before);
    const value = privateText(request);
    if (value == null) throw new WorkerFailure('target-semantic-element');
    if (Number.isFinite(request.arguments.x) && Number.isFinite(request.arguments.y)) {
      // 慢脑只有在 AX 元素缺失或属性写入不可信时才提交截图坐标。坐标已由
      // 协议限制为 input-text 语义；先聚焦该点，再发送字面文本，并由动作后
      // Observe 交给慢脑核验。这里不隐式清空、不重试，也不猜测其他控件。
      await callAction('click', { app: target.app, x: request.arguments.x, y: request.arguments.y }, target.applicationId);
      await callAction('type_text', { app: target.app, text: value }, target.applicationId);
      pendingInput = { taskId: request.task_id, app: target.app, value };
      return { kind: 'text-present', value };
    }
    if (element) {
      // QQ 音乐等自绘文本框会让 AX set_value 报告成功但不刷新界面；而
      // type_text 也不一定覆盖已选文本。显式聚焦、全选并删除选区后再键入，
      // 每个调用只执行一次；任一调用失败都会以未知结果交回，不自动重试。
      await callAction('click', { app: target.app, element_index: String(element.index) }, target.applicationId);
      await callAction('press_key', { app: target.app, key: 'super+a' }, target.applicationId);
      await callAction('press_key', { app: target.app, key: 'BackSpace' }, target.applicationId);
      await callAction('type_text', { app: target.app, text: value }, target.applicationId);
      pendingInput = { taskId: request.task_id, app: target.app, value };
      return { kind: 'text-present', value };
    }
    await callAction('type_text', { app: target.app, text: value }, target.applicationId);
    pendingInput = { taskId: request.task_id, app: target.app, value };
    return { kind: 'changed' };
  }
  if (request.tool_name === 'hotkey') {
    const keys = request.arguments.keys;
    if (!Array.isArray(keys) || keys.join('+').toLowerCase() !== 'cmd+f') throw new WorkerFailure('target-semantic-key');
    await callAction('press_key', { app: target.app, key: 'super+f' }, target.applicationId);
    return { kind: 'changed' };
  }
  if (semantic === 'activate-control' && request.tool_name === 'click'
      && request.arguments.element_index == null
      && !Number.isFinite(request.arguments.x) && !Number.isFinite(request.arguments.y)) {
    const query = pendingInput?.taskId === request.task_id && pendingInput.app === target.app
      ? pendingInput.value
      : null;
    if (typeof query !== 'string') throw new WorkerFailure('target-semantic-query');
    const element = firstMatchingControl(before, query);
    if (!element) throw new WorkerFailure('target-semantic-element');
    await callAction('click', { app: target.app, element_index: String(element.index) }, target.applicationId);
    return { kind: 'transcript-changed' };
  }
  if (semantic === 'activate-control' && request.tool_name === 'press_key') {
    const key = request.arguments.key;
    if (typeof key !== 'string' || !['ENTER', 'RETURN', 'SPACE', 'ARROWDOWN'].includes(key.toUpperCase())) {
      throw new WorkerFailure('target-semantic-key');
    }
    const normalizedKey = key.toUpperCase() === 'SPACE'
      ? 'space'
      : key.toUpperCase() === 'ARROWDOWN'
        ? 'Down'
        : 'Return';
    await callAction('press_key', { app: target.app, key: normalizedKey }, target.applicationId);
    return { kind: key.toUpperCase() === 'ARROWDOWN' ? 'changed' : 'transcript-changed' };
  }
  const name = request.tool_name;
  if (!supported.has(name)) {
    throw new Error('tool unavailable');
  }
  const args = actionArguments(request, target);
  if (args.element_index != null) args.element_index = String(args.element_index);
  if (name === 'press_key' && typeof args.key === 'string' && ['ENTER', 'RETURN'].includes(args.key.toUpperCase())) args.key = 'Return';
  await callAction(name, args, target.applicationId);
  return { kind: 'changed' };
}

function actionConfirmed(expectation, before, after) {
  if (!expectation) return true;
  if (expectation.kind === 'element-present') {
    return transcriptElements(after).some(element => element.index === expectation.index);
  }
  if (expectation.kind === 'text-present') {
    // AXValue 与自绘界面可能分裂（QQ 音乐已复现：AX 报告新值但截图仍是
    // 旧值）。Driver 没有可靠视觉断言时必须交回带截图的 UnknownObserved，
    // 由慢脑读取新鲜事实后决定下一片段，不能凭 AX 自报继续副作用链。
    return false;
  }
  if (expectation.kind === 'transcript-changed') {
    return stateText(before) !== stateText(after);
  }
  if (expectation.kind === 'coordinate-timeout') return false;
  return fingerprint(before) !== fingerprint(after);
}

for await (const line of createInterface({ input: process.stdin, crlfDelay: Infinity })) {
  let request;
  try { request = JSON.parse(line); } catch { process.exitCode = 2; break; }
  const response = responseFor(request);
  try {
    if (bridgeFailureStage) throw new WorkerFailure(bridgeFailureStage);
    let target = await targetFor(request);
    const externalMacFocus = request.tool_name === 'bring_to_front';
    let before = null;
    if (request.tool_name !== 'launch_app' && !externalMacFocus) {
      response.failure_stage = 'observe-before';
      before = await observe(target);
    }
    response.failure_stage = 'action';
    const expectation = await perform(request, target, before);
    response.action_known = true;
    if (request.tool_name === 'launch_app') target = await bindRunningTarget(target);
    response.failure_stage = 'observe-after';
    const after = await observe(target);
    response.observe_valid = true;
    response.target_visible = request.tool_name === 'launch_app' ? false : externalMacFocus ? true : null;
    response.element_count = Math.min(65_535, transcriptElements(after).length);
    response.action_effect = request.tool_name === 'launch_app' || externalMacFocus || actionConfirmed(expectation, before, after)
      ? 'confirmed'
      : 'suspected_noop';
    response.action_succeeded = response.action_effect === 'confirmed';
    if (request.tool_name === 'launch_app' && response.action_succeeded) {
      response.launched_app_id = target.applicationId;
    }
    await persistScreenshot(after, request, response);
    response.failure_stage = response.action_succeeded
      ? null
      : stateText(after).trim() === ''
        ? 'action-unconfirmed-empty-transcript'
        : 'action-unconfirmed-unparsed-transcript';
  } catch (error) {
    if (error instanceof WorkerFailure) response.failure_stage = error.stage;
    // 不把应用名、辅助功能文本、截图或 SDK 错误正文带回宿主日志。
  }
  process.stdout.write(`${JSON.stringify(response)}\n`);
}
bridge.close();
