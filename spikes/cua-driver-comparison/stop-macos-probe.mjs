// 只读技术Spike，结果仅含分类/布尔；不把取消或进程退出冒充输入已停止。
import { spawn, spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const child = process.argv[2];
if (child === 'permissions') {
  if (process.platform !== 'darwin') throw Error('仅macOS权限检查');
  const { currentMacOsPermissionStatus } = await import('@trycua/cua-driver');
  const status = currentMacOsPermissionStatus();
  console.log(JSON.stringify({ package: '@trycua/cua-driver', version: '0.25.0', accessibility: status.accessibility, screen_recording: status.screenRecording, input_dispatched: false, recording_started: false, permission_requested: false, yonder_host_verified: false }));
  process.exit(0);
}
if (child === 'lifecycle' || child === 'crash' || child === 'recover' || child === 'submitted' || child === 'sdk-worker') {
  const { CuaDriver } = await import('@trycua/cua-driver');
  const driver = CuaDriver.create(undefined);
  if (child === 'crash') { await driver.metadata(); process.exit(23); }
  const capture = async fn => {
    try {
      const response = await fn();
      return { resolved: true, is_error: typeof response?.isError === 'boolean' ? response.isError : null };
    } catch (error) {
      const code = String(error?.code ?? 'unknown');
      return { resolved: false, code: /^[a-zA-Z0-9_-]{1,48}$/.test(code) ? code : 'unknown' };
    }
  };
  try {
    await driver.metadata();
    if (child === 'sdk-worker') {
      await new Promise((resolve, reject) => {
        process.once('disconnect', () => driver.shutdown().then(resolve, reject));
        process.send({ sdk_ready: true });
      });
    } else if (child === 'submitted') {
      const controller = new AbortController();
      let settled = false;
      const read = capture(() => driver.callTool('list_apps', '{}', { signal: controller.signal })).then(value => { settled = true; return value; });
      await new Promise(resolve => setImmediate(resolve));
      const abortBeforeSettlement = !settled;
      controller.abort();
      const submittedRead = await read;
      const afterAbort = await capture(() => driver.callTool('list_apps', '{}'));
      const shutdownRead = capture(() => driver.callTool('list_apps', '{}'));
      const shutdown = await capture(() => driver.shutdown());
      const afterShutdown = await capture(() => driver.callTool('list_apps', '{}'));
      console.log(JSON.stringify({ abort_before_settlement: abortBeforeSettlement, native_admission_observed: false, submitted_read: submittedRead, read_after_abort: afterAbort, shutdown_read: await shutdownRead, shutdown, after_shutdown: afterShutdown }));
    } else if (child === 'recover') {
      console.log(JSON.stringify({ read_after_crash: await capture(() => driver.callTool('list_apps', '{}')) }));
    } else {
      const unknown = await capture(() => driver.callTool('yonder_unknown_tool', '{}'));
      const controller = new AbortController(); controller.abort();
      const cancelled = await capture(() => driver.callTool('list_apps', '{}', { signal: controller.signal }));
      const shutdown = await capture(() => driver.shutdown());
      const afterShutdown = await capture(() => driver.callTool('list_apps', '{}'));
      const secondShutdown = await capture(() => driver.shutdown());
      console.log(JSON.stringify({ unknown_tool: unknown, pre_aborted_read: cancelled, shutdown, after_shutdown: afterShutdown, second_shutdown: secondShutdown }));
    }
  } finally {
    if (child === 'recover') await driver.shutdown();
    driver.uniffiDestroy();
  }
} else {
  if (process.platform !== 'darwin') throw Error('仅macOS补充Spike');
  const run = mode => {
    const result = spawnSync(process.execPath, [fileURLToPath(import.meta.url), mode], { encoding: 'utf8', timeout: 25000, maxBuffer: 65536 });
    if (mode === 'crash') return { expected_exit: result.status === 23, timeout: result.error?.code === 'ETIMEDOUT' };
    if (result.status !== 0) return { completed: false, timeout: result.error?.code === 'ETIMEDOUT', exit_code: result.status };
    try { return { completed: true, result: JSON.parse(result.stdout) }; }
    catch { return { completed: false, invalid_result: true }; }
  };
  const supervise = mode => new Promise(resolve => {
    const worker = spawn(process.execPath, [fileURLToPath(import.meta.url), 'sdk-worker'], { stdio: ['ignore', 'ignore', 'ignore', 'ipc'] });
    let ready = false;
    let timeout = false;
    const timer = setTimeout(() => { timeout = true; worker.kill('SIGKILL'); }, 25000);
    worker.once('error', () => { clearTimeout(timer); resolve({ ready, completed: false, spawn_error: true }); });
    worker.on('message', message => {
      if (ready || message?.sdk_ready !== true) return;
      ready = true;
      if (mode === 'terminate') worker.kill('SIGTERM');
      else worker.disconnect();
    });
    worker.once('exit', (code, signal) => {
      clearTimeout(timer);
      resolve({ ready, completed: !timeout && ready && (mode === 'terminate' ? signal === 'SIGTERM' : code === 0 && signal === null), timeout, exit_code: code, exit_signal: signal });
    });
  });
  const result = { package: '@trycua/cua-driver', version: '0.25.0', platform: 'macos', lifecycle: run('lifecycle'), crash: run('crash'), recovery: run('recover'), input_dispatched: false, recording_started: false, native_input_stop_verified: false };
  result.submitted = run('submitted');
  result.sdk_workers = { terminated: await supervise('terminate'), disconnected: await supervise('disconnect'), recovery: run('recover') };
  result.sdk_worker_lifecycle_passed = !!(result.sdk_workers.terminated.completed && result.sdk_workers.disconnected.completed && result.sdk_workers.recovery.completed && result.sdk_workers.recovery.result?.read_after_crash?.resolved === true && result.sdk_workers.recovery.result?.read_after_crash?.is_error === false);
  const lifecycle = result.lifecycle.result;
  const rejected = response => response?.resolved === false || response?.is_error === true;
  result.readonly_lifecycle_passed = !!(result.lifecycle.completed && rejected(lifecycle?.unknown_tool) && rejected(lifecycle?.pre_aborted_read) && lifecycle?.shutdown?.resolved === true && rejected(lifecycle?.after_shutdown) && lifecycle?.second_shutdown?.resolved === true && result.crash.expected_exit && result.recovery.completed && result.recovery.result?.read_after_crash?.resolved === true && result.recovery.result?.read_after_crash?.is_error === false);
  const submitted = result.submitted.result;
  result.submitted_lifecycle_passed = !!(result.submitted.completed && typeof submitted?.abort_before_settlement === 'boolean' && typeof submitted?.submitted_read?.resolved === 'boolean' && submitted?.read_after_abort?.resolved === true && submitted?.read_after_abort?.is_error === false && typeof submitted?.shutdown_read?.resolved === 'boolean' && submitted?.shutdown?.resolved === true && rejected(submitted?.after_shutdown));
  const output = process.argv[2];
  if (!output) throw Error('缺少结构化结果路径');
  mkdirSync(dirname(output), { recursive: true }); writeFileSync(output, JSON.stringify(result, null, 2) + '\n');
  console.log(JSON.stringify(result));
  if (!result.readonly_lifecycle_passed || !result.submitted_lifecycle_passed || !result.sdk_worker_lifecycle_passed) process.exitCode = 1;
}
