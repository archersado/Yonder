import { createInterface } from 'node:readline';
import { stat, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const sdk = await import(pathToFileURL(process.argv[2]).href);
const structured = value => value.structuredJson ? JSON.parse(value.structuredJson) : JSON.parse(value.rawJson ?? '{}').structuredContent ?? {};
const driver = sdk.CuaDriver.create(undefined);
try {
  await driver.metadata();
  const inventory = JSON.parse(await driver.listToolsJson());
  const tools = Array.isArray(inventory) ? inventory : inventory.tools ?? [];
  let launchedTarget;
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
      response.failure_stage = 'observe-before';
      const args = { pid: request.pid, window_id: request.window_id, include_screenshot: false, max_elements: 100 };
      const before = await driver.callTool('get_window_state', JSON.stringify(args));
      if (!before.isError) {
        const schema = descriptor.inputSchema ?? descriptor.input_schema ?? {};
        const properties = schema.properties ?? {};
        const actionArgs = { ...request.arguments };
        const desktopScope = actionArgs.scope === 'desktop';
        const target = request.tool_name === 'bring_to_front' && launchedTarget?.task_id === request.task_id ? launchedTarget : request;
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
        if (!action.isError && request.tool_name === 'launch_app') {
          const launched = structured(action);
          const window = launched.windows?.filter(item => Number.isInteger(item.window_id))
            .sort((left, right) => (right.bounds?.width ?? 0) * (right.bounds?.height ?? 0) - (left.bounds?.width ?? 0) * (left.bounds?.height ?? 0))[0];
          launchedTarget = Number.isInteger(launched.pid) ? {
            task_id: request.task_id,
            pid: launched.pid,
            ...(typeof (actionArgs.bundle_id ?? launched.bundle_id) === 'string' ? { bundle_id: actionArgs.bundle_id ?? launched.bundle_id } : {}),
            ...(window ? { window_id: window.window_id } : {}),
          } : undefined;
        }
        response.action_known = true;
        response.action_succeeded = !action.isError;
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
              const apps = await driver.callTool('list_apps', '{}');
              const app = !apps.isError ? (structured(apps).apps ?? []).find(item =>
                item.running && (launchedTarget.bundle_id ? item.bundle_id === launchedTarget.bundle_id : item.pid === launchedTarget.pid)) : undefined;
              if (Number.isInteger(app?.pid)) launchedTarget.pid = app.pid;
              const windows = await driver.callTool('list_windows', JSON.stringify({ pid: launchedTarget.pid, on_screen_only: false }));
              if (!apps.isError && !windows.isError) {
                const listedWindows = structured(windows).windows ?? [];
                const window = listedWindows.find(item => item.is_on_screen === true && item.on_current_space !== false);
                if (Number.isInteger(window?.window_id)) launchedTarget.window_id = window.window_id;
                const visible = window?.is_on_screen === true && window?.on_current_space !== false;
                response.target_visible = request.tool_name === 'bring_to_front' && !action.isError && visible;
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
