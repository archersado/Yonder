import { createInterface } from 'node:readline';
import { execFile, spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { realpath, writeFile } from 'node:fs/promises';
import net from 'node:net';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);
const evidence = process.argv[3];
const installedService = join(homedir(), '.codex', 'computer-use', 'Codex Computer Use.app');

// @oai/sky 的 macOS transport 只依赖这组受信任宿主能力。Yonder 在进入
// worker 前已经完成任务准入，因此此处的 accept 只承接同一授权边界；服务端
// 的组织策略/禁止列表仍会先执行，无法被该 shim 绕过。
const nodeReplShim = {
  env: {
    ...process.env,
    NODE_REPL_DISABLE_ANALYTICS: '1',
    BROWSER_USE_DISABLE_AMBIENT_NETWORK: '1',
    SKY_CUA_SERVICE_PATH: process.env.SKY_CUA_SERVICE_PATH
      ?? (existsSync(installedService) ? installedService : undefined),
  },
  nativePipe: {
    createConnection(path) {
      return new Promise((resolve, reject) => {
        const socket = net.createConnection(path);
        socket.once('connect', () => resolve(socket));
        socket.once('error', reject);
      });
    },
  },
  launchServices: {
    openApplication(application) {
      const args = application.applicationPath
        ? [application.applicationPath]
        : ['-b', application.bundleIdentifier];
      return new Promise((resolve, reject) => {
        const child = spawn('/usr/bin/open', args, { stdio: 'ignore' });
        child.once('error', reject);
        child.once('exit', code => code === 0 ? resolve() : reject(new Error('service launch failed')));
      });
    },
  },
  createElicitation: async () => ({ action: 'accept', _meta: { persist: 'session' } }),
  withSuspendedTimeout: async callback => callback(),
  setResponseMeta() {},
};

// sky.js 在加载时用 nodeRepl.rpc 判断是否运行于 Codex trusted-RPC 宿主。
// 独立 driver 必须先选择 direct client，再为 mac transport 注入最小能力集。
const { sky } = await import(pathToFileURL(process.argv[2]).href);
globalThis.nodeRepl = nodeReplShim;
const supported = new Set([
  'click', 'drag', 'paste', 'perform_secondary_action', 'press_key', 'scroll',
  'select_text', 'set_value', 'type_text', 'launch_app', 'activate_window', 'bring_to_front',
]);
let launchedTarget;

class WorkerFailure extends Error {
  constructor(stage) {
    super(stage);
    this.stage = stage;
  }
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
    if (sky.target !== 'mac' || typeof app !== 'string' || app.trim() === '') {
      throw new Error('launch target is unavailable');
    }
    const requested = app.trim();
    let apps;
    try {
      apps = await sky.list_apps();
    } catch {
      throw new WorkerFailure('target-list-apps');
    }
    const matches = apps.filter(candidate => candidate.id === requested || candidate.displayName === requested);
    if (matches.length !== 1 || typeof matches[0].id !== 'string' || matches[0].id === '') {
      throw new WorkerFailure('target-canonical-app');
    }
    const applicationId = matches[0].id;
    const resolvedApp = matches[0].isRunning === true
      ? await runningMacAppForBundleId(applicationId)
      : applicationId;
    return { app: resolvedApp, applicationId };
  }
  if (request.tool_name === 'bring_to_front' && sky.target === 'mac') {
    if (launchedTarget?.task_id !== request.task_id) throw new Error('launched target is unavailable');
    return { app: launchedTarget.app };
  }
  if (sky.target === 'mac') return { app: await macAppForPid(request.pid) };
  if (sky.target === 'linux' || sky.target === 'windows') {
    const windows = await sky.list_windows();
    const window = windows.find(candidate => Number(candidate.id) === request.window_id);
    if (!window) throw new Error('exact window is unavailable');
    return { window };
  }
  throw new Error('unsupported sky target');
}

async function observe(target) {
  if (sky.target === 'mac') return sky.get_app_state({ app: target.app, disableDiff: true });
  return sky.get_window_state({ window: target.window, include_text: true, include_screenshot: true });
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

async function persistScreenshot(state, request, response) {
  const url = screenshotUrl(state);
  if (typeof url !== 'string') return;
  const match = url.match(/^data:(image\/(?:png|jpeg|webp));base64,([A-Za-z0-9+/=]+)$/);
  if (!match) return;
  const bytes = Buffer.from(match[2], 'base64');
  if (bytes.length === 0 || bytes.length > 4 * 1024 * 1024) return;
  const extension = match[1] === 'image/png' ? 'png' : match[1] === 'image/jpeg' ? 'jpg' : 'webp';
  const path = join(evidence, `${request.task_id}-${request.attempt_id}.${extension}`);
  await writeFile(path, bytes, { flag: 'wx', mode: 0o600 });
  response.screenshot_path = path;
  response.screenshot_mime = match[1];
}

function actionArguments(request, target) {
  const args = { ...request.arguments, ...target };
  delete args.pid;
  delete args.window_id;
  delete args.app;
  if (target.app) args.app = target.app;
  return args;
}

async function perform(request, target) {
  if (request.tool_name === 'launch_app') {
    // macOS Sky 将启动封装在 get_app_state 中，且不会把应用抢到前台。
    await sky.get_app_state({ app: target.app, disableDiff: true });
    return;
  }
  if (request.tool_name === 'bring_to_front' && sky.target === 'mac') {
    // Rust 已用精确 pid/window_id 执行并验证原生 AX raise；这里仅做后置观察。
    return;
  }
  const name = request.tool_name === 'bring_to_front' ? 'activate_window' : request.tool_name;
  if (!supported.has(request.tool_name) || typeof sky[name] !== 'function') {
    throw new Error('tool unavailable');
  }
  await sky[name](actionArguments(request, target));
}

for await (const line of createInterface({ input: process.stdin, crlfDelay: Infinity })) {
  let request;
  try { request = JSON.parse(line); } catch { process.exitCode = 2; break; }
  const response = responseFor(request);
  try {
    const target = await targetFor(request);
    const externalMacFocus = request.tool_name === 'bring_to_front' && sky.target === 'mac';
    let before = null;
    if (request.tool_name !== 'launch_app' && !externalMacFocus) {
      response.failure_stage = 'observe-before';
      before = await observe(target);
    }
    response.failure_stage = 'action';
    await perform(request, target);
    response.action_known = true;
    response.failure_stage = 'observe-after';
    const after = await observe(target);
    response.observe_valid = true;
    response.target_visible = request.tool_name === 'launch_app' ? false : externalMacFocus ? true : null;
    const indexes = [...stateText(after).matchAll(/\[(\d+)\]/g)].map(match => Number(match[1]));
    response.element_count = Math.min(65_535, new Set(indexes).size);
    response.action_effect = request.tool_name === 'launch_app' || externalMacFocus || fingerprint(before) !== fingerprint(after)
      ? 'confirmed'
      : 'suspected_noop';
    response.action_succeeded = response.action_effect === 'confirmed';
    if (request.tool_name === 'launch_app' && response.action_succeeded) {
      launchedTarget = { task_id: request.task_id, app: target.app };
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
