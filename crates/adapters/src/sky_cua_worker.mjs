import { createInterface } from 'node:readline';
import { execFile, spawn } from 'node:child_process';
import { readFile, realpath, writeFile } from 'node:fs/promises';
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
let launchedTarget;

class SkyMcpBridge {
  constructor(path) {
    this.nextId = 1;
    this.pending = new Map();
    this.activeApprovalAppId = null;
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
  }

  async callTool(name, args, approvalAppId = null) {
    this.activeApprovalAppId = approvalAppId;
    try {
      const result = await this.request('tools/call', { name, arguments: args });
      if (!result || !Array.isArray(result.content)) throw new Error('MCP tool failed');
      if (result.isError === true) {
        const failure = resultText(result).toLocaleLowerCase();
        if (failure.includes('cgwindownotfound') || failure.includes('error -10005')) {
          throw new WorkerFailure('target-window-unavailable');
        }
        if (failure.includes('sender process is not authenticated')) {
          throw new WorkerFailure('transport-authentication');
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
}

class WorkerFailure extends Error {
  constructor(stage) {
    super(stage);
    this.stage = stage;
  }
}

function boundedFailureStage(error, fallback) {
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

const bridge = new SkyMcpBridge(bridgePath);
let bridgeFailureStage = null;
try {
  await bridge.start();
} catch (error) {
  bridgeFailureStage = boundedFailureStage(error, 'transport-mcp-handshake');
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
  if (!text.startsWith('{') && !text.startsWith('[')) return null;
  try { return JSON.parse(text); } catch { return null; }
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
  const text = typeof structured?.text === 'string'
    ? structured.text
    : structured?.ax_tree != null
      ? String(structured.ax_tree)
      : resultText(result);
  return { app: structured?.app ?? app, text, screenshot };
}

async function getAppState(app, applicationId) {
  return stateFromResult(await bridge.callTool('get_app_state', { app }, applicationId), app);
}

async function callAction(name, args, applicationId) {
  await bridge.callTool(name, args, applicationId);
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
    if (matches.length !== 1 || typeof matches[0].id !== 'string' || matches[0].id === '') {
      throw new WorkerFailure('target-canonical-app');
    }
    const applicationId = matches[0].id;
    const resolvedApp = matches[0].isRunning === true && typeof matches[0].path === 'string'
      ? matches[0].path
      : matches[0].isRunning === true ? await runningMacAppForBundleId(applicationId) : applicationId;
    return { app: resolvedApp, applicationId };
  }
  if (launchedTarget?.task_id === request.task_id) {
    return { app: launchedTarget.app, applicationId: launchedTarget.applicationId };
  }
  if (request.tool_name === 'bring_to_front') {
    if (launchedTarget?.task_id !== request.task_id) throw new Error('launched target is unavailable');
    return { app: launchedTarget.app, applicationId: launchedTarget.applicationId };
  }
  return { app: await macAppForPid(request.pid) };
}

async function observe(target) {
  return getAppState(target.app, target.applicationId);
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
  for (const line of stateText(state).split('\n')) {
    const match = line.match(/^\s*(?:\[(\d+)\]|(\d+))\s+(.+)$/);
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
    await getAppState(target.app, target.applicationId);
    return;
  }
  if (request.tool_name === 'bring_to_front') {
    // Yonder WindowActivationPort 已执行并验证原生 AX raise；这里仅做后置观察。
    return;
  }
  const semantic = request.arguments._yonder_action_kind;
  if (['focus-target-search', 'focus-control'].includes(semantic)) {
    const element = searchField(before);
    if (!element) throw new WorkerFailure('target-semantic-element');
    await callAction('click', { app: target.app, element_index: String(element.index) }, target.applicationId);
    return { kind: 'element-present', index: element.index };
  }
  if (['enter-target-query', 'input-text'].includes(semantic)) {
    const element = searchField(before);
    const value = privateText(request);
    if (value == null) throw new WorkerFailure('target-semantic-element');
    if (element) {
      await callAction('set_value', { app: target.app, element_index: String(element.index), value }, target.applicationId);
      return { kind: 'text-present', value };
    }
    await callAction('type_text', { app: target.app, text: value }, target.applicationId);
    return { kind: 'changed' };
  }
  if (request.tool_name === 'hotkey') {
    const keys = request.arguments.keys;
    if (!Array.isArray(keys) || keys.join('+').toLowerCase() !== 'cmd+f') throw new WorkerFailure('target-semantic-key');
    await callAction('press_key', { app: target.app, key: 'super+f' }, target.applicationId);
    return { kind: 'changed' };
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
  if (expectation.kind === 'text-present') return stateText(after).includes(expectation.value);
  return fingerprint(before) !== fingerprint(after);
}

for await (const line of createInterface({ input: process.stdin, crlfDelay: Infinity })) {
  let request;
  try { request = JSON.parse(line); } catch { process.exitCode = 2; break; }
  const response = responseFor(request);
  try {
    if (bridgeFailureStage) throw new WorkerFailure(bridgeFailureStage);
    const target = await targetFor(request);
    const externalMacFocus = request.tool_name === 'bring_to_front';
    let before = null;
    if (request.tool_name !== 'launch_app' && !externalMacFocus) {
      response.failure_stage = 'observe-before';
      before = await observe(target);
    }
    response.failure_stage = 'action';
    const expectation = await perform(request, target, before);
    response.action_known = true;
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
      launchedTarget = { task_id: request.task_id, app: target.app, applicationId: target.applicationId };
      response.launched_app_id = target.applicationId;
    }
    await persistScreenshot(after, request, response);
    response.failure_stage = null;
  } catch (error) {
    if (error instanceof WorkerFailure) response.failure_stage = error.stage;
    // 不把应用名、辅助功能文本、截图或 SDK 错误正文带回宿主日志。
  }
  process.stdout.write(`${JSON.stringify(response)}\n`);
}
bridge.close();
