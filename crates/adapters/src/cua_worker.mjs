import { createInterface } from 'node:readline';
import { stat, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const sdk = await import(pathToFileURL(process.argv[2]).href);
const structured = value => value.structuredJson ? JSON.parse(value.structuredJson) : JSON.parse(value.rawJson ?? '{}').structuredContent ?? {};
// trycua 0.25.0 的 launch_state 在不同平台实现中可能是旧版枚举字符串，
// 也可能是包含 requested/process_running/window_ready 的结构化状态。两种形态
// 表达的是同一契约；只要 SDK 已返回可信 PID 且确认进程运行或窗口就绪，就可
// 将 launch_app 判为已确认，不能因响应形态差异误报 observe-failed。
const launchStateConfirmed = state =>
  ['process_running', 'window_ready'].includes(state)
  || (state && typeof state === 'object'
    && (state.process_running === true || state.window_ready === true));
// bring_to_front 的平台扩展结果没有通用 effect；精确窗口成功由 verified 与
// 稳定 code 共同表达。partial/unverified 仍由 isError 收敛为 refused，不能因
// 进程已激活或请求已接受而提升为成功。
const bringToFrontConfirmed = result =>
  result?.exact_window_effect?.verified === true
  && result?.code === 'bring_to_front_exact_window_verified';
const applicationFrontConfirmed = (result, targetPid) =>
  result?.request_accepted === true
  && result?.process_activated === true
  && result?.observed?.front_process_matches_target === true
  && result?.observed?.workspace_frontmost_pid === targetPid;
const driver = sdk.CuaDriver.create(undefined);
try {
  await driver.metadata();
  const inventory = JSON.parse(await driver.listToolsJson());
  const tools = Array.isArray(inventory) ? inventory : inventory.tools ?? [];
  let launchedTarget;
  const refreshLaunchedTarget = async preferVisible => {
    if (!launchedTarget) return undefined;
    const apps = await driver.callTool('list_apps', '{}');
    const app = !apps.isError ? (structured(apps).apps ?? []).find(item =>
      item.running && (launchedTarget.bundle_id ? item.bundle_id === launchedTarget.bundle_id : item.pid === launchedTarget.pid)) : undefined;
    if (Number.isInteger(app?.pid)) launchedTarget.pid = app.pid;
    const windows = await driver.callTool('list_windows', JSON.stringify({ pid: launchedTarget.pid, on_screen_only: false }));
    if (apps.isError || windows.isError) return undefined;
    const listedWindows = (structured(windows).windows ?? []).filter(item => Number.isInteger(item.window_id));
    const byArea = (left, right) => (right.bounds?.width ?? 0) * (right.bounds?.height ?? 0) - (left.bounds?.width ?? 0) * (left.bounds?.height ?? 0);
    const visible = listedWindows.filter(item => item.is_on_screen === true && item.on_current_space !== false).sort(byArea);
    const window = preferVisible && visible.length ? visible[0] : listedWindows.sort(byArea)[0];
    if (window) launchedTarget.window_id = window.window_id;
    return { window, visible: visible.length > 0 };
  };
  for await (const line of createInterface({ input: process.stdin, crlfDelay: Infinity })) {
    let request;
    try { request = JSON.parse(line); } catch { process.exitCode = 2; break; }
    const response = {
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
      failure_stage: 'metadata',
    };
    try {
      const descriptor = tools.find(tool => tool.name === request.tool_name);
      if (!descriptor) throw new Error('tool unavailable');
      if (request.tool_name === 'bring_to_front' && launchedTarget?.task_id === request.task_id) {
        response.failure_stage = 'target-refresh';
        await refreshLaunchedTarget(false);
      }
      response.failure_stage = 'observe-before';
      const target = request.tool_name === 'bring_to_front' && launchedTarget?.task_id === request.task_id ? launchedTarget : request;
      const actionTargetPid = target.pid;
      const args = { pid: target.pid, window_id: target.window_id, include_screenshot: false, max_elements: 100 };
      const before = await driver.callTool('get_window_state', JSON.stringify(args));
      if (!before.isError) {
        const schema = descriptor.inputSchema ?? descriptor.input_schema ?? {};
        const properties = schema.properties ?? {};
        const actionArgs = { ...request.arguments };
        const desktopScope = actionArgs.scope === 'desktop';
        if (!desktopScope && 'pid' in properties) actionArgs.pid = target.pid;
        if (!desktopScope && 'window_id' in properties && Number.isInteger(target.window_id)) actionArgs.window_id = target.window_id;
        if (!desktopScope && 'windowId' in properties && Number.isInteger(target.window_id)) actionArgs.windowId = target.window_id;
        if ('session' in properties) actionArgs.session = request.host_session_id;
        if (!desktopScope && request.tool_name === 'type_text' && !('x' in actionArgs) && !('y' in actionArgs)) {
          const fields = (structured(before).elements ?? []).filter(element => /textfield|edit/i.test(element.role ?? '') && element.enabled !== false && element.element_token);
          if (fields.length !== 1) throw new Error('editable target unavailable');
          if ('element_token' in properties) actionArgs.element_token = fields[0].element_token;
        }
        response.failure_stage = 'action';
        const action = await driver.callTool(request.tool_name, JSON.stringify(actionArgs));
        // trycua 会用 isError=true 表达 partial/refused，但结构化正文仍携带
        // request_accepted、前台进程和精确窗口后置事实。错误位决定动作不能
        // 直接成功，不代表这些证据可以丢弃；应用级前置收敛仍需后续可见
        // Observe 与这些同次动作事实共同成立。
        const actionResult = structured(action);
        if (!action.isError && request.tool_name === 'launch_app') {
          const launched = actionResult;
          const window = launched.windows?.filter(item => Number.isInteger(item.window_id))
            .sort((left, right) => (right.bounds?.width ?? 0) * (right.bounds?.height ?? 0) - (left.bounds?.width ?? 0) * (left.bounds?.height ?? 0))[0];
          launchedTarget = Number.isInteger(launched.pid) ? {
            task_id: request.task_id,
            pid: launched.pid,
            ...(typeof (actionArgs.bundle_id ?? launched.bundle_id) === 'string' ? { bundle_id: actionArgs.bundle_id ?? launched.bundle_id } : {}),
            ...(window ? { window_id: window.window_id } : {}),
          } : undefined;
        }
        const launchConfirmed=request.tool_name==='launch_app' && Number.isInteger(actionResult.pid) && launchStateConfirmed(actionResult.launch_state);
        const bringConfirmed=request.tool_name==='bring_to_front' && !action.isError && bringToFrontConfirmed(actionResult);
        const applicationFront=request.tool_name==='bring_to_front' && applicationFrontConfirmed(actionResult,actionTargetPid);
        response.action_effect=launchConfirmed||bringConfirmed?'confirmed':typeof actionResult.effect==='string'?actionResult.effect:action.isError?'refused':'unverifiable';
        response.action_known = true;
        response.action_succeeded = response.action_effect === 'confirmed';
        response.failure_stage = 'observe-after';
        const desktop = tools.find(tool => tool.name === 'get_desktop_state');
        const screenshotPath = join(process.argv[3], `${request.task_id}-${request.attempt_id}.png`);
        const observe = async (descriptor, name, base) => {
          const properties = (descriptor?.inputSchema ?? descriptor?.input_schema ?? {}).properties ?? {};
          const input = { ...base };
          if ('include_screenshot' in properties) input.include_screenshot = true;
          if ('max_elements' in properties) input.max_elements = 100;
          if ('session' in properties) input.session = request.host_session_id;
          if ('screenshot_out_file' in properties) input.screenshot_out_file = screenshotPath;
          if ('screenshotOutFile' in properties) input.screenshotOutFile = screenshotPath;
          return [await driver.callTool(name, JSON.stringify(input)), properties];
        };
        let [after, afterProperties] = desktop ? await observe(desktop, 'get_desktop_state', {}) : await observe(tools.find(tool => tool.name === 'get_window_state'), 'get_window_state', args);
        if (after.isError && desktop) [after, afterProperties] = await observe(tools.find(tool => tool.name === 'get_window_state'), 'get_window_state', args);
        response.observe_valid = !after.isError;
        if (response.observe_valid) {
          const state = structured(after);
          response.element_count = Math.min(65535, Array.isArray(state.elements) ? state.elements.length : Array.isArray(state.windows) ? state.windows.length : 0);
          if ('screenshot_out_file' in afterProperties || 'screenshotOutFile' in afterProperties) {
            const metadata = await stat(screenshotPath).catch(() => null);
            if (metadata?.isFile() && metadata.size <= 4 * 1024 * 1024) {
              response.screenshot_path = screenshotPath;
              response.screenshot_mime = 'image/png';
            }
          } else {
            const image = after.images?.[0];
            if (image && ['image/png', 'image/jpeg', 'image/webp'].includes(image.mimeType)) {
            const bytes = Buffer.from(image.dataBase64, 'base64');
            if (bytes.length <= 4 * 1024 * 1024) {
              const extension = image.mimeType === 'image/png' ? 'png' : image.mimeType === 'image/jpeg' ? 'jpg' : 'webp';
              response.screenshot_path = join(process.argv[3], `${request.task_id}-${request.attempt_id}.${extension}`);
              response.screenshot_mime = image.mimeType;
              await writeFile(response.screenshot_path, bytes, { flag: 'wx', mode: 0o600 });
            }
            }
          }
          if (launchedTarget?.task_id === request.task_id && ['launch_app', 'bring_to_front'].includes(request.tool_name)) {
            const deadline = Date.now() + (request.tool_name === 'launch_app' ? 0 : 2000);
            do {
              const refreshed = await refreshLaunchedTarget(true);
              response.target_visible = request.tool_name === 'bring_to_front' && refreshed?.visible === true && (bringConfirmed || applicationFront);
              if (response.target_visible === true && applicationFront) {
                response.action_effect = 'confirmed';
                response.action_succeeded = true;
              }
              if (response.target_visible === true || Date.now() >= deadline) break;
              await new Promise(resolve => setTimeout(resolve, 100));
            } while (true);
          }
          response.failure_stage = null;
        }
      }
    } catch {
      // Rust 将异常、缺失 Observe 和进程失败统一收敛为 unknown；不输出正文或 SDK Payload。
    }
    process.stdout.write(JSON.stringify(response) + '\n');
  }
} finally {
  try { await driver.shutdown(); } catch {}
  driver.uniffiDestroy();
}
