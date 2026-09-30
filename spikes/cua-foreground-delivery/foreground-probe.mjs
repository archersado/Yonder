import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';
import { once } from 'node:events';
import { pathToFileURL } from 'node:url';

const sdk = process.env.YONDER_CUA_SDK
  ? await import(pathToFileURL(process.env.YONDER_CUA_SDK).href)
  : await import('@trycua/cua-driver');
const { CuaDriver, currentMacOsPermissionStatus } = sdk;

const [targetPath, decoyPath, output] = process.argv.slice(2);
if (process.platform !== 'darwin' || !targetPath || !decoyPath || !output) throw new Error('需要macOS目标、诱饵与结果路径');
const structured = response => response.structuredJson ? JSON.parse(response.structuredJson) : JSON.parse(response.rawJson ?? '{}').structuredContent ?? {};
const wait = async (predicate, timeout = 10000) => {
  const deadline = Date.now() + timeout;
  while (!predicate() && Date.now() < deadline) await new Promise(resolve => setTimeout(resolve, 25));
  if (!predicate()) throw new Error('bounded-wait-timeout');
};
const result = { version:process.env.YONDER_CUA_VERSION ?? '0.30.4', marker_length:18, accessibility:false, screen_recording:false, wrong_target_rejected:false, target_unchanged_after_rejection:false, action_confirmed:false, sdk_observe_matches:false, fixture_matches:false, decoy_restored:false, after_shutdown_rejected:false, cleanup:false, passed:false };
let target;
let decoy;
let driver;
let targetState;
let decoyState;
try {
  const permission = currentMacOsPermissionStatus();
  result.accessibility = permission.accessibility === true;
  result.screen_recording = permission.screenRecording === true;
  if (!result.accessibility || !result.screen_recording) throw new Error('permissions-unavailable');
  target = spawn(targetPath, [], { stdio:['ignore','pipe','ignore'] });
  createInterface({ input:target.stdout }).on('line', line => { try { const value=JSON.parse(line); if(value.ready===true) targetState=value; } catch {} });
  await wait(() => targetState?.ready === true);
  decoy = spawn(decoyPath, [], { stdio:['ignore','pipe','ignore'] });
  createInterface({ input:decoy.stdout }).on('line', line => { try { const value=JSON.parse(line); if(value.ready===true) decoyState=value; } catch {} });
  await wait(() => decoyState?.active === true);
  driver = CuaDriver.create(undefined);
  await driver.metadata();
  const call = (name, args) => driver.callTool(name, JSON.stringify(args));
  const windows = structured(await call('list_windows', { pid:targetState.pid, on_screen_only:false })).windows ?? [];
  const matches = windows.filter(window => Number(window.pid)===targetState.pid && Number(window.window_id)===targetState.window_id);
  if (matches.length !== 1) throw new Error('target-not-unique');
  const targetRef = { kind:'window', pid:targetState.pid, window_id:targetState.window_id };
  const observe = () => call('get_window_state', { pid:targetState.pid, window_id:targetState.window_id, include_screenshot:true, include_accessibility_tree:true, max_elements:100 });
  const before = await observe();
  if (before.isError) throw new Error('observe-before-failed');
  const wrong = await call('type_text', { target:{...targetRef,window_id:targetState.window_id+999999}, x:220, y:92, text:'YONDER_SDK_INPUT_A', delivery_mode:'foreground' });
  result.wrong_target_rejected = wrong.isError === true;
  await new Promise(resolve => setTimeout(resolve, 200));
  result.target_unchanged_after_rejection = targetState.matches === false && targetState.length === 0;
  if (!result.wrong_target_rejected || !result.target_unchanged_after_rejection) throw new Error('wrong-target-not-closed');
  if (process.env.YONDER_DROP_DECOY === '1') {
    const exited = once(decoy, 'exit');
    decoy.kill('SIGTERM');
    await exited;
    decoyState = { ...decoyState, active:false };
  }
  const typed = await call('type_text', { target:targetRef, x:220, y:92, text:'YONDER_SDK_INPUT_A', delivery_mode:'foreground' });
  const typedResult = structured(typed);
  result.action_confirmed = typed.isError === false && (typedResult.effect === 'confirmed' || typedResult.effect === 0);
  await wait(() => targetState?.matches === true);
  result.fixture_matches = targetState.matches === true && targetState.length === 18;
  const after = await observe();
  result.sdk_observe_matches = after.isError === false && JSON.stringify(structured(after).elements ?? []).includes('YONDER_SDK_INPUT_A');
  try { await wait(() => decoyState?.active === true, 3000); } catch {}
  result.decoy_restored = decoyState?.active === true;
  await driver.shutdown();
  try {
    const rejected = await call('type_text', { target:targetRef, x:220, y:92, text:'YONDER_SDK_INPUT_A', delivery_mode:'foreground' });
    result.after_shutdown_rejected = rejected.isError === true;
  } catch { result.after_shutdown_rejected = true; }
  driver.uniffiDestroy();
  driver = undefined;
  result.passed = result.action_confirmed && result.sdk_observe_matches && result.fixture_matches && result.decoy_restored && result.after_shutdown_rejected;
} catch (error) {
  result.failure = ['permissions-unavailable','bounded-wait-timeout','target-not-unique','observe-before-failed','wrong-target-not-closed'].includes(error.message) ? error.message : 'probe-failed';
} finally {
  if (driver) { try { await driver.shutdown(); driver.uniffiDestroy(); } catch { result.cleanup_error=true; result.passed=false; } }
  for (const child of [target,decoy]) {
    if (child?.pid && child.exitCode===null && child.signalCode===null) { const exited=once(child,'exit'); child.kill('SIGTERM'); await exited; }
  }
  result.cleanup = [target,decoy].every(child => !child || child.exitCode !== null || child.signalCode !== null);
  result.passed &&= result.cleanup;
  mkdirSync(dirname(output), { recursive:true });
  writeFileSync(output, JSON.stringify(result,null,2)+'\n');
  console.log(JSON.stringify(result));
}
if (!result.passed) process.exitCode=1;
