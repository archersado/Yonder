// SDK契约与隔离原生窗口前置；无用户窗口正文、输入或截图日志。
import { CuaDriver, currentMacOsPermissionStatus } from '@trycua/cua-driver';
import { spawn, spawnSync } from 'node:child_process';
import { createInterface } from 'node:readline';
import { once } from 'node:events';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';
const [mode, first, helper, nativeOutput] = process.argv.slice(2);
const output = mode === '--catalog' ? first : nativeOutput;
if (!output || !['--catalog', '--native'].includes(mode)) throw Error('需要catalog/native模式和输出路径');
const result = { sdk_only: true, input_dispatched: false, recording_started: false, screenshot_requested: false, yonder_host_verified: false, other_space_verified: false, windows_verified: false };
const structured = response => response.structuredJson ? JSON.parse(response.structuredJson) : JSON.parse(response.rawJson ?? '{}').structuredContent ?? {};
let driver, fixture, state;
const wait = async predicate => {
 const end = Date.now() + 5000;
 while (!predicate() && Date.now() < end) await new Promise(resolve => setTimeout(resolve, 50));
 if (!predicate()) throw Error('隔离目标状态超时');
};
try {
 driver = CuaDriver.create(undefined);
 if (mode === '--catalog') {
  const catalog = JSON.parse(await driver.listToolsJson());
  result.tools = (Array.isArray(catalog) ? catalog : catalog.tools ?? []).filter(tool => /focus|activate|restore|window/.test(tool.name));
  result.passed = true;
 } else {
  if (process.platform !== 'darwin' || !helper || !currentMacOsPermissionStatus().accessibility) throw Error('原生权限或环境不满足');
  fixture = spawn(first, ['--focus-fixture'], { stdio: ['pipe','pipe','ignore'] });
  fixture.on('error', () => { result.fixture_spawn_error = true; });
  createInterface({input:fixture.stdout}).on('line', line => {try { state = JSON.parse(line); } catch {result.fixture_invalid_state=true;} });
  result.phase='fixture_ready';
  await wait(() => state?.ready && state.launched && state.decoy_visible);
  const target = {pid:state.pid, window_id:state.window_id};
  const call = (tool,args) => driver.callTool(tool,JSON.stringify(args));
  const focus = () => {
   const run=spawnSync(helper,[String(target.pid),String(target.window_id)],{encoding:'utf8',timeout:10000,maxBuffer:8192});
   try {return {...JSON.parse(run.stdout), exit_code:run.status};} catch {throw Error('原生定位结果不可用');}
  };
  const observe = async () => {
   const observed=await call('get_window_state',{...target,include_screenshot:false,max_elements:100});
   return !observed.isError && (structured(observed).elements ?? []).some(element => /textfield|edit/i.test(element.role ?? ''));
  };
  const command = text => fixture.stdin.write(text+'\n');
  await driver.metadata();
  const listed=await call('list_windows',{pid:target.pid});
  result.target_listed=!listed.isError && (structured(listed).windows ?? []).some(window=>window.window_id===target.window_id && window.pid===target.pid);
  if(!result.target_listed) throw Error('target not listed');
  result.phase='initial_observe';
  result.initial_sdk_observe=false;
  for(let read=0;read<3 && !result.initial_sdk_observe;read++) {
   result.initial_observe_reads=read+1;
   result.initial_sdk_observe=await observe();
   if(!result.initial_sdk_observe) await new Promise(resolve=>setTimeout(resolve,100));
  }
  if(!result.initial_sdk_observe) throw Error('initial Observe unavailable');
  result.phase='normal_focus';
  const normal=focus();
  result.normal_native=normal;
  await wait(() => state?.target_key && state.app_active && state.target_on_active_space);
  result.normal_focus = normal.exit_code===0 && normal.focused && normal.unique_mapping;
  result.normal_geometry_unchanged = normal.geometry_unchanged;
  result.normal_sdk_observe = await observe();
  result.phase='minimize';
  command('minimize');
  await wait(() => state?.target_minimized);
  result.minimized_fixture_confirmed=true;
  result.phase='restore';
  const restored=focus();
  result.restored_native=restored;
  await wait(() => state?.target_key && !state.target_minimized && state.target_on_active_space);
  result.minimized_restore_focus = restored.exit_code===0 && restored.restored && restored.focused;
  result.restored_geometry_unchanged = restored.geometry_unchanged;
  result.restored_sdk_observe = await observe();
  result.phase='close';
  command('close');
  await wait(() => state?.target_closed && !state.target_key);
  const beforeClosedFocus={active:state.app_active,key:state.decoy_key};
  const closed=focus();
  result.closed_native=closed;
  result.closed_target_refused=[3,4].includes(closed.exit_code) && closed.refused && (closed.target_missing || closed.mapping_not_unique);
  result.same_name_decoy_untouched=state.decoy_visible===true && state.app_active===beforeClosedFocus.active && state.decoy_key===beforeClosedFocus.key;
  const missing=await call('get_window_state',{...target,include_screenshot:false,max_elements:100});
  const missingState=structured(missing);
  result.closed_sdk_error=missing.isError===true;
  result.closed_sdk_ax_empty=(missingState.elements ?? []).length===0;
  result.closed_sdk_degraded=missingState.degraded===true;
  result.closed_sdk_degraded_reason=/^[a-z_]{1,64}$/.test(missingState.degraded_reason ?? '') ? missingState.degraded_reason : 'other';
  result.closed_sdk_target_unavailable=result.closed_sdk_error || (result.closed_sdk_ax_empty && result.closed_target_refused && state.target_closed===true);
  result.passed=result.normal_focus && result.normal_geometry_unchanged && result.normal_sdk_observe && result.minimized_restore_focus && result.restored_geometry_unchanged && result.restored_sdk_observe && result.closed_target_refused && result.same_name_decoy_untouched && result.closed_sdk_target_unavailable;
 }
} catch { result.failure='隔离窗口定位验证失败';result.passed=false; }
finally {
 if(mode==='--native') result.final_fixture={ready:state?.ready===true,target_key:state?.target_key===true,decoy_key:state?.decoy_key===true,app_active:state?.app_active===true};
 if(fixture?.pid && fixture.exitCode===null && fixture.signalCode===null) {const exited=once(fixture,'exit');fixture.kill('SIGTERM');await exited;}
 if(driver) {try {await driver.shutdown();driver.uniffiDestroy();} catch {result.cleanup_error=true;result.passed=false;}}
 mkdirSync(dirname(output),{recursive:true});writeFileSync(output,JSON.stringify(result,null,2)+'\n');
 console.log(JSON.stringify(mode==='--catalog' ? {...result,tools:result.tools?.map(tool=>tool.name)} : result));
}
if(!result.passed) process.exitCode=1;
