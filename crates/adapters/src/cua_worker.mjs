import { createInterface } from 'node:readline';
import { stat, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const sdk = await import(pathToFileURL(process.argv[2]).href);
const structured = value => value.structuredJson ? JSON.parse(value.structuredJson) : JSON.parse(value.rawJson ?? '{}').structuredContent ?? {};
const captureObservation = async (driver, descriptor, name, base, screenshotPath, session) => {
  const properties = (descriptor?.inputSchema ?? descriptor?.input_schema ?? {}).properties ?? {};
  const input = { ...base };
  if ('include_screenshot' in properties) input.include_screenshot = true;
  if ('max_elements' in properties) input.max_elements = 100;
  if ('session' in properties) input.session = session;
  if ('screenshot_out_file' in properties) input.screenshot_out_file = screenshotPath;
  if ('screenshotOutFile' in properties) input.screenshotOutFile = screenshotPath;
  const result = await driver.callTool(name, JSON.stringify(input));
  if (result.isError) return { result, elementCount: 0, screenshot: null };
  const state = structured(result);
  const elementCount = Math.min(65535, Array.isArray(state.elements) ? state.elements.length : Array.isArray(state.windows) ? state.windows.length : 0);
  if ('screenshot_out_file' in properties || 'screenshotOutFile' in properties) {
    const metadata = await stat(screenshotPath).catch(() => null);
    if (metadata?.isFile() && metadata.size <= 4 * 1024 * 1024) {
      return { result, elementCount, screenshot: { path: screenshotPath, mime: 'image/png' } };
    }
  } else {
    const image = result.images?.[0];
    if (image && ['image/png', 'image/jpeg', 'image/webp'].includes(image.mimeType)) {
      const bytes = Buffer.from(image.dataBase64, 'base64');
      if (bytes.length <= 4 * 1024 * 1024) {
        const extension = image.mimeType === 'image/png' ? 'png' : image.mimeType === 'image/jpeg' ? 'jpg' : 'webp';
        const path = screenshotPath.replace(/\.png$/, `.${extension}`);
        await writeFile(path, bytes, { flag: 'wx', mode: 0o600 });
        return { result, elementCount, screenshot: { path, mime: image.mimeType } };
      }
    }
  }
  return { result, elementCount, screenshot: null };
};
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
// trycua 的原生绑定把 ActionEffect 暴露为数值枚举，而部分 JSON/MCP
// 适配器会返回稳定字符串。两种形态来自同一 SDK 契约，必须在进入 Yonder
// 运行时事实前归一化；否则真实点击的 Confirmed(0) 会被误判为 unverifiable。
const actionEffect = (result, isError) => {
  const effect = result?.effect;
  if (typeof effect === 'string') {
    const normalized = effect.replaceAll('_', '-').replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase();
    if (['confirmed', 'partial', 'unverifiable', 'suspected-noop', 'refused'].includes(normalized)) return normalized;
  }
  return ({ 0:'confirmed', 1:'partial', 2:'unverifiable', 3:'suspected-noop', 4:'refused' })[effect]
    ?? (isError ? 'refused' : 'unverifiable');
};
const elementText = element => [element.title, element.label, element.name, element.value, element.description, element.placeholder]
  .filter(value => typeof value === 'string').join(' ').trim();
const textField = element => /textfield|edit|textbox|searchfield/i.test(element.role ?? '') && element.enabled !== false && element.element_token;
const searchField = element => textField(element) && /search|搜索|查找/i.test(elementText(element));
const uniqueElement = elements => elements.length === 1 ? elements[0] : undefined;
const driver = sdk.CuaDriver.create(undefined);
try {
  await driver.metadata();
  const inventory = JSON.parse(await driver.listToolsJson());
  const tools = Array.isArray(inventory) ? inventory : inventory.tools ?? [];
  let launchedTarget;
  let trustedVisualFocus;
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
      if (request.tool_name !== 'launch_app' && launchedTarget?.task_id === request.task_id) {
        response.failure_stage = 'target-refresh';
        await refreshLaunchedTarget(false);
      }
      response.failure_stage = 'observe-before';
      // launch_app之后的同任务窗口级动作继续绑定Driver返回的可信应用身份。
      // 这不会前置窗口；desktop scope仍由SDK自行解析当前桌面。
      const target = request.tool_name !== 'launch_app' && launchedTarget?.task_id === request.task_id ? launchedTarget : request;
      const actionTargetPid = target.pid;
      const args = { pid: target.pid, window_id: target.window_id, include_screenshot: false, max_elements: 100 };
      const before = await driver.callTool('get_window_state', JSON.stringify(args));
      if (!before.isError) {
        const schema = descriptor.inputSchema ?? descriptor.input_schema ?? {};
        const properties = schema.properties ?? {};
        const actionArgs = { ...request.arguments };
        const semanticKind = actionArgs._yonder_action_kind;
        const privateText = actionArgs._yonder_private_text;
        delete actionArgs._yonder_action_kind;
        delete actionArgs._yonder_private_text;
        const beforeElements = structured(before).elements ?? [];
        const coordinateClick = request.tool_name === 'click' && Number.isFinite(actionArgs.x) && Number.isFinite(actionArgs.y);
        const coordinateText = request.tool_name === 'type_text'
          && ['enter-target-query','draft-message-ref'].includes(semanticKind)
          && Number.isFinite(actionArgs.x)
          && Number.isFinite(actionArgs.y);
        const coordinateAction = coordinateClick || coordinateText;
        const foregroundCoordinate = coordinateAction;
        const semanticShortcut = semanticKind === 'focus-target-search'
          && request.tool_name === 'hotkey'
          && Array.isArray(actionArgs.keys)
          && actionArgs.keys.length === 2
          && actionArgs.keys[0] === 'cmd'
          && actionArgs.keys[1] === 'f';
        const focusAction = coordinateClick || semanticShortcut;
        let trustedFocusedInput = false;
        if (semanticKind) {
          response.failure_stage = 'semantic-target';
          let selected;
          if (semanticKind === 'focus-target-search' || semanticKind === 'enter-target-query') {
            const explicit = beforeElements.filter(searchField);
            const allFields = beforeElements.filter(textField);
            selected = uniqueElement(explicit) ?? uniqueElement(allFields);
          } else if (semanticKind === 'activate-target') {
            const exact = beforeElements.filter(element => element.enabled !== false && element.element_token && elementText(element) === privateText);
            const partial = beforeElements.filter(element => element.enabled !== false && element.element_token && typeof privateText === 'string' && elementText(element).includes(privateText));
            selected = uniqueElement(exact) ?? uniqueElement(partial);
          } else if (semanticKind === 'focus-message-composer' || semanticKind === 'draft-message-ref') {
            const composers = beforeElements.filter(element => textField(element) && !searchField(element));
            selected = uniqueElement(composers);
          } else if (semanticKind === 'send-message' && request.tool_name === 'click') {
            selected = uniqueElement(beforeElements.filter(element => element.enabled !== false && element.element_token && /^(发送|send)$/i.test(elementText(element))));
          }
          if (!coordinateAction && !semanticShortcut && (['focus-target-search', 'enter-target-query', 'activate-target', 'focus-message-composer', 'draft-message-ref'].includes(semanticKind) || (semanticKind === 'send-message' && request.tool_name === 'click'))) {
            trustedFocusedInput = request.tool_name === 'type_text'
              && trustedVisualFocus?.task_id === request.task_id
              && trustedVisualFocus?.pid === target.pid
              && trustedVisualFocus?.window_id === target.window_id
              && ((semanticKind === 'enter-target-query' && trustedVisualFocus.kind === 'search')
                || (semanticKind === 'draft-message-ref' && trustedVisualFocus.kind === 'composer'));
            if ((!selected || !('element_token' in properties)) && !trustedFocusedInput) {
              const screenshotPath = join(process.argv[3], `${request.task_id}-${request.attempt_id}.png`);
              const observed = await captureObservation(driver, tools.find(tool => tool.name === 'get_window_state'), 'get_window_state', args, screenshotPath, request.host_session_id);
              response.action_known = true;
              response.action_effect = 'refused';
              response.observe_valid = !observed.result.isError;
              response.element_count = observed.elementCount;
              response.screenshot_path = observed.screenshot?.path ?? null;
              response.screenshot_mime = observed.screenshot?.mime ?? null;
              response.failure_stage = 'semantic-target-ambiguous';
              process.stdout.write(JSON.stringify(response) + '\n');
              continue;
            }
            if (selected && 'element_token' in properties) actionArgs.element_token = selected.element_token;
          }
          if (semanticKind === 'enter-target-query' || semanticKind === 'draft-message-ref') {
            if (typeof privateText !== 'string' || privateText.length === 0) throw new Error('private text missing');
            actionArgs.text = privateText;
          }
          if (semanticKind === 'send-message' && request.tool_name === 'press_key') actionArgs.key = actionArgs.key ?? 'ENTER';
        }
        if (foregroundCoordinate) {
          if ('delivery_mode' in properties) actionArgs.delivery_mode = 'foreground';
          else if ('deliveryMode' in properties) actionArgs.deliveryMode = 'foreground';
          else {
            const screenshotPath = join(process.argv[3], `${request.task_id}-${request.attempt_id}.png`);
            const observed = await captureObservation(driver, tools.find(tool => tool.name === 'get_window_state'), 'get_window_state', args, screenshotPath, request.host_session_id);
            response.action_known = true;
            response.action_effect = 'refused';
            response.observe_valid = !observed.result.isError;
            response.element_count = observed.elementCount;
            response.screenshot_path = observed.screenshot?.path ?? null;
            response.screenshot_mime = observed.screenshot?.mime ?? null;
            response.failure_stage = 'foreground-delivery-unavailable';
            process.stdout.write(JSON.stringify(response) + '\n');
            continue;
          }
        }
        const desktopScope = actionArgs.scope === 'desktop';
        if (!desktopScope && 'target' in properties && Number.isInteger(target.pid) && Number.isInteger(target.window_id)) {
          actionArgs.target = { kind:'window', pid:target.pid, window_id:target.window_id };
        } else {
          if (!desktopScope && 'pid' in properties) actionArgs.pid = target.pid;
          if (!desktopScope && 'window_id' in properties && Number.isInteger(target.window_id)) actionArgs.window_id = target.window_id;
          if (!desktopScope && 'windowId' in properties && Number.isInteger(target.window_id)) actionArgs.windowId = target.window_id;
        }
        if ('session' in properties) actionArgs.session = request.host_session_id;
        if (!desktopScope && request.tool_name === 'type_text' && !trustedFocusedInput && !('x' in actionArgs) && !('y' in actionArgs) && !('element_token' in actionArgs)) {
          const fields = (structured(before).elements ?? []).filter(element => /textfield|edit/i.test(element.role ?? '') && element.enabled !== false && element.element_token);
          if (fields.length !== 1) {
            // 元素缺失或不唯一时尚未执行任何副作用。只在此分支采集一次截图，
            // 让归属慢脑按视觉证据提交坐标动作；正常元素路径不请求截图。
            response.failure_stage = 'visual-fallback-observe';
            const screenshotPath = join(process.argv[3], `${request.task_id}-${request.attempt_id}.png`);
            const observed = await captureObservation(driver, tools.find(tool => tool.name === 'get_window_state'), 'get_window_state', args, screenshotPath, request.host_session_id);
            response.action_known = true;
            response.action_effect = 'refused';
            response.observe_valid = !observed.result.isError;
            response.element_count = observed.elementCount;
            response.screenshot_path = observed.screenshot?.path ?? null;
            response.screenshot_mime = observed.screenshot?.mime ?? null;
            if (response.observe_valid && response.screenshot_path) response.failure_stage = null;
            process.stdout.write(JSON.stringify(response) + '\n');
            continue;
          }
          if ('element_token' in properties) actionArgs.element_token = fields[0].element_token;
        }
        response.failure_stage = 'action';
        // 视觉焦点凭据只允许一次派发尝试。即使Driver拒绝、超时或结果不可核实，
        // 也不能保留凭据让敏感引用文本在后续请求中隐式重放。
        const consumedVisualFocus = trustedFocusedInput;
        if (consumedVisualFocus) trustedVisualFocus = undefined;
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
        response.action_effect=launchConfirmed||bringConfirmed?'confirmed':actionEffect(actionResult,action.isError);
        const escalationReason = actionResult?.escalation?.reason;
        if (['partial','unverifiable','suspected-noop'].includes(response.action_effect)) {
          response.failure_stage = `action-${response.action_effect}-${typeof escalationReason === 'string' ? escalationReason : 'unconfirmed'}`;
        }
        response.action_known = true;
        response.action_succeeded = response.action_effect === 'confirmed';
        response.failure_stage = 'observe-after';
        const desktop = tools.find(tool => tool.name === 'get_desktop_state');
        const windowState = tools.find(tool => tool.name === 'get_window_state');
        if (request.tool_name === 'launch_app' && launchedTarget?.task_id === request.task_id) await refreshLaunchedTarget(false);
        const observationTarget = request.tool_name === 'launch_app' && launchedTarget?.task_id === request.task_id ? launchedTarget : target;
        const observationArgs = { pid: observationTarget.pid, window_id: observationTarget.window_id, include_screenshot: false, max_elements: 100 };
        const screenshotPath = join(process.argv[3], `${request.task_id}-${request.attempt_id}.png`);
        const visualAction = focusAction || coordinateText || consumedVisualFocus;
        const observe = async (descriptor, name, base) => {
          const properties = (descriptor?.inputSchema ?? descriptor?.input_schema ?? {}).properties ?? {};
          const input = { ...base };
          // 元素动作的后置核验只读取结构化状态；坐标动作才需要窗口截图验证。
          // 这样正常元素路径不会因为通用Observe而隐式升级成视觉路径。
          if ('include_screenshot' in properties) input.include_screenshot = visualAction;
          if ('max_elements' in properties) input.max_elements = 100;
          if ('session' in properties) input.session = request.host_session_id;
          if ('screenshot_out_file' in properties) input.screenshot_out_file = screenshotPath;
          if ('screenshotOutFile' in properties) input.screenshotOutFile = screenshotPath;
          return [await driver.callTool(name, JSON.stringify(input)), properties];
        };
        // 窗口级动作必须观察同一可信窗口；全桌面截图既不能证明后台动作，
        // 也会把视觉重规划引向当时的前台应用。只有显式desktop scope才观察桌面。
        let [after, afterProperties] = desktopScope && desktop
          ? await observe(desktop, 'get_desktop_state', {})
          : await observe(windowState, 'get_window_state', observationArgs);
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
          if (!['partial','unverifiable','suspected-noop'].includes(response.action_effect)) response.failure_stage = null;
          if (!action.isError && response.observe_valid && focusAction && response.action_effect === 'confirmed') {
            if (semanticKind === 'focus-target-search') trustedVisualFocus = { task_id:request.task_id, pid:target.pid, window_id:target.window_id, kind:'search' };
            if (semanticKind === 'focus-message-composer') trustedVisualFocus = { task_id:request.task_id, pid:target.pid, window_id:target.window_id, kind:'composer' };
          }
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
