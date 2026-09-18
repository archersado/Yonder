// 固定隔离目标AX输入/Observe；禁止前台、剪贴板或其他窗口兜底。
import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';
import { once } from 'node:events';
import { fileURLToPath } from 'node:url';
import { CuaDriver, currentMacOsPermissionStatus } from '@trycua/cua-driver';

const structured = response => response.structuredJson ? JSON.parse(response.structuredJson) : JSON.parse(response.rawJson ?? '{}').structuredContent ?? {};
if (process.argv[2] === '--sdk-input-worker') {
  const driver = CuaDriver.create(undefined);
  const pid = Number(process.argv[3]);
  const windowId = Number(process.argv[4]);
  await driver.metadata();
  await driver.callTool('list_windows', '{}');
  const before = await driver.callTool('get_window_state', JSON.stringify({ pid, window_id: windowId, include_screenshot: false, max_elements: 100 }));
  const fields = (structured(before).elements ?? []).filter(element => /textfield|edit/i.test(element.role ?? '') && element.enabled !== false && element.element_token);
  if (before.isError || fields.length !== 1) {
    process.send({ target_error: true, observe_error: before.isError, field_count: fields.length }, () => process.exit(2));
    await new Promise(() => {});
  }
  let inputStarted = false;
  let stopping = false;
  process.on('message', async message => {
    if (message?.input === true && !inputStarted && !stopping) {
      inputStarted = true;
      const typed = await driver.callTool('type_text', JSON.stringify({ pid, element_token: fields[0].element_token, text: 'YONDER_SDK_INPUT_A', delivery_mode: 'background' }));
      const action = structured(typed);
      process.send({ input_completed: true, input_error: typed.isError, route: action.route === 'accessibility' ? 'accessibility' : 'other', effect: action.effect === 'confirmed' ? 'confirmed' : 'other' });
    } else if (message?.stop === true && !stopping) {
      stopping = true;
      await driver.shutdown();
      process.send({ shutdown_completed: true, time_ms: Date.now() });
      driver.uniffiDestroy();
      process.disconnect();
    }
  });
  process.send({ sdk_ready: true });
  await new Promise(resolve => process.once('disconnect', resolve));
  process.exit(0);
}
const [fixturePath, output, mode] = process.argv.slice(2);
if (process.platform !== 'darwin' || !fixturePath || !output) throw Error('需要macOS测试可执行文件与结果路径');
const permission = currentMacOsPermissionStatus();
const result = { sdk_only: true, accessibility: permission.accessibility, input_dispatched: false, recording_started: false, screenshot_requested: false, native_inflight_stop_verified: false, yonder_host_verified: false };
let fixture;
let driver;
let state;
let native = {};
let worker;
let workerState = {};
let workerExit;
const call = (name, args) => driver.callTool(name, JSON.stringify(args));
const wait = async predicate => {
  const end = Date.now() + 30000;
  while (!predicate() && Date.now() < end) {
    if (workerState.target_error || (workerState.exited && !workerState.shutdown_completed)) throw Error('SDK子进程目标初始化失败');
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  if (!predicate()) throw Error('隔离目标观察超时');
};
try {
  if (!permission.accessibility) throw Error('SDK宿主Accessibility未授权');
  fixture = spawn(fixturePath, mode === 'inflight-drain' ? ['--delay-ax'] : [], { stdio: ['ignore', 'pipe', 'ignore'] });
  fixture.on('error', () => { result.fixture_spawn_error = true; });
  createInterface({ input: fixture.stdout }).on('line', line => {
    try {
      const value = JSON.parse(line);
      if (value.ready === true) state = value;
      if (value.native_write_started === true) native.started_ms = value.time_ms;
      if (value.native_write_applied === true) native.applied_ms = value.time_ms;
    } catch { result.fixture_invalid_result = true; }
  });
  await wait(() => state?.ready === true);
  driver = CuaDriver.create(undefined);
  await driver.metadata();
  const windows = structured(await call('list_windows', {})).windows ?? [];
  const targets = windows.filter(window => Number(window.pid) === state.pid && Number(window.window_id) === state.window_id);
  if (targets.length !== 1) throw Error('隔离窗口身份不唯一');
  const target = targets[0];
  const observe = () => call('get_window_state', { pid: target.pid, window_id: target.window_id, include_screenshot: false, max_elements: 100 });
  const before = await observe();
  if (before.isError) throw Error('输入前SDK Observe拒绝');
  const fields = (structured(before).elements ?? []).filter(element => /textfield|edit/i.test(element.role ?? '') && element.enabled !== false && element.element_token);
  result.ax_roles = (structured(before).elements ?? []).slice(0,100).map(element => ({ role: /^[A-Za-z_ ]{1,64}$/.test(element.role ?? '') ? element.role : 'other', enabled: element.enabled !== false, token_present: !!element.element_token }));
  if (fields.length !== 1) throw Error('唯一可编辑AX token未取得');
  result.target_unique = true;
  if (mode === 'inflight-drain') {
    worker = spawn(process.execPath, [fileURLToPath(import.meta.url), '--sdk-input-worker', String(state.pid), String(state.window_id)], { stdio: ['ignore', 'ignore', 'ignore', 'ipc'] });
    workerExit = once(worker, 'exit');
    workerExit.then(([code, signal]) => { workerState.exited = true; workerState.exit_code = code; workerState.exit_signal = signal; });
    worker.on('message', value => { workerState = { ...workerState, ...value }; });
    await wait(() => workerState.sdk_ready === true);
    result.input_dispatched = true;
    worker.send({ input: true });
    await wait(() => native.started_ms !== undefined);
    if (workerState.input_completed || native.applied_ms !== undefined) throw Error('隔离原生写入未处于执行中');
    result.native_write_started = true;
    result.shutdown_requested_ms = Date.now();
    worker.send({ stop: true });
    await wait(() => workerState.shutdown_completed === true && native.applied_ms !== undefined && state?.matches === true);
    const [code, signal] = await workerExit;
    result.worker_exited = code === 0 && signal === null;
    result.native_started_ms = native.started_ms;
    result.native_applied_ms = native.applied_ms;
    result.shutdown_completed_ms = workerState.time_ms;
    result.shutdown_after_native_apply = workerState.time_ms >= native.applied_ms;
    result.shutdown_during_native_write = result.shutdown_requested_ms < native.applied_ms;
    result.input_route = workerState.route;
    result.input_effect = workerState.effect;
    const after = await observe();
    result.sdk_observe_matches = !after.isError && JSON.stringify(structured(after).elements ?? []).includes('YONDER_SDK_INPUT_A');
    result.fixture_matches = state.matches;
    result.inflight_ax_drain_verified = result.worker_exited && result.shutdown_after_native_apply && result.shutdown_during_native_write && workerState.input_completed === true && workerState.input_error === false && result.sdk_observe_matches && result.fixture_matches;
    result.passed = result.inflight_ax_drain_verified;
  } else {
  const args = { pid: target.pid, element_token: fields[0].element_token, text: 'YONDER_SDK_INPUT_A', delivery_mode: 'background' };
  result.input_dispatched = true;
  const typed = await call('type_text', args);
  result.input_error = typed.isError;
  if (typed.isError) throw Error('SDK AX输入拒绝');
  const after = await observe();
  result.sdk_observe_matches = !after.isError && JSON.stringify(structured(after).elements ?? []).includes('YONDER_SDK_INPUT_A');
  await wait(() => state?.matches === true);
  result.fixture_matches = state.matches;
  result.input_length = state.length;
  await driver.shutdown();
  try {
    const rejected = await call('type_text', { ...args, text: 'YONDER_SDK_INPUT_B' });
    result.after_shutdown_input_rejected = rejected.isError === true;
  } catch { result.after_shutdown_input_rejected = true; }
  driver.uniffiDestroy();
  driver = undefined;
  driver = CuaDriver.create(undefined);
  const stopped = await observe();
  result.stopped_sdk_observe_matches = !stopped.isError && JSON.stringify(structured(stopped).elements ?? []).includes('YONDER_SDK_INPUT_A');
  let stable = true;
  const end = Date.now() + 750;
  while (Date.now() < end) {
    stable &&= state?.matches === true && state?.length === result.input_length;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  result.stopped_fixture_stable = stable;
  result.passed = result.sdk_observe_matches && result.fixture_matches && result.after_shutdown_input_rejected && result.stopped_sdk_observe_matches && stable;
  }
} catch (error) {
  // 仅本探针固定错误说明，不保存SDK正文或Payload。
  result.failure = error.message.startsWith('SDK') || error.message.startsWith('隔离') || error.message.startsWith('唯一') || error.message.startsWith('输入前') ? error.message : '探针调用失败';
  result.passed = false;
} finally {
  if (mode === 'inflight-drain') {
    result.native_write_started = native.started_ms !== undefined;
    result.native_write_applied = native.applied_ms !== undefined;
    result.worker_input_completed = workerState.input_completed === true;
    result.worker_input_error = workerState.input_error ?? null;
    result.input_route = workerState.route ?? null;
    result.input_effect = workerState.effect ?? null;
    result.fixture_matches = state?.matches === true;
    result.worker_target_error = workerState.target_error === true;
    result.worker_exit_code = workerState.exit_code ?? null;
  }
  if (worker?.pid && worker.exitCode === null && worker.signalCode === null) { worker.kill('SIGKILL'); await workerExit; }
  if (fixture?.pid && fixture.exitCode === null && fixture.signalCode === null) {
    const exited = once(fixture, 'exit');
    fixture.kill('SIGTERM');
    await exited;
  }
  if (driver) {
    try { await driver.shutdown(); driver.uniffiDestroy(); }
    catch { result.cleanup_error = true; result.passed = false; }
  }
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, JSON.stringify(result, null, 2) + '\n');
  console.log(JSON.stringify(result));
}
if (!result.passed) process.exitCode = 1;
