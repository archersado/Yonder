// 隔离工作引用Spike：固定SDK无截图Observe，原生对象验证，不改正式任务。
import assert from 'node:assert/strict';
import {CuaDriver} from '@trycua/cua-driver';
import {spawn} from 'node:child_process';
import {createInterface} from 'node:readline';
import {once} from 'node:events';
import {mkdirSync,writeFileSync} from 'node:fs';
import {dirname,join} from 'node:path';
const [fixturePath,helperPath,output]=process.argv.slice(2);
if(process.platform!=='darwin'||!output)throw Error('需要macOS及fixture/helper/output');
mkdirSync(dirname(output),{recursive:true});mkdirSync(output);
const result={passed:false,sdk_version:'0.25.0',input_dispatched:false,recording_started:false,screenshot_requested:false,windows_verified:false,yonder_host_verified:false,samples:{}};
const children=[];
let driver;
const wait=async predicate=>{
 const end=Date.now()+5000;
 while(!predicate()&&Date.now()<end)await new Promise(r=>setTimeout(r,30));
 assert.ok(predicate(),'隔离状态超时');
};
function start(program,args){
 const process=spawn(program,args,{stdio:['pipe','pipe','ignore']});
 const channel={process,state:null,lines:[]};children.push(channel);
 process.on('error',()=>{channel.failed=true;});
 createInterface({input:process.stdout}).on('line',line=>{
  try {channel.state=JSON.parse(line);channel.lines.push(channel.state);}catch{channel.failed=true;}
 });
 return channel;
}
async function command(channel,text){
 const index=channel.lines.length;channel.process.stdin.write(text+'\n');
 await wait(()=>channel.lines.length>index||channel.failed||channel.process.exitCode!==null);
 assert.ok(channel.lines.length>index&&!channel.failed,'原生响应不可用');
 return channel.lines[index];
}
async function fixture(){
 const channel=start(fixturePath,['--focus-fixture']);
 await wait(()=>channel.state?.launched&&channel.state?.decoy_visible);
 return channel;
}
async function retain(f,id){
 let readyObserve=false;
 for(let i=0;i<8&&!readyObserve;i++){
  readyObserve=await observe(f,id);
  if(!readyObserve)await new Promise(r=>setTimeout(r,150));
 }
 assert.ok(readyObserve,'新引用前必须有效Observe');
 const channel=start(helperPath,[String(f.state.pid),String(id),'--retain']);
 await wait(()=>channel.state?.ready||channel.process.exitCode!==null);
 if(!channel.state?.ready)result.retain_failure=channel.state??{exit_code:channel.process.exitCode};
 assert.ok(channel.state?.ready&&channel.state.process_start_available);
 return channel;
}
async function stop(channel){
 if(channel.process.exitCode===null&&channel.process.signalCode===null){
  const exited=once(channel.process,'exit');channel.process.kill('SIGTERM');await exited;
 }
}
async function observe(f,id){
 const response=await driver.callTool('get_window_state',JSON.stringify({pid:f.state.pid,window_id:id,include_screenshot:false,max_elements:100}));
 const state=response.structuredJson?JSON.parse(response.structuredJson):JSON.parse(response.rawJson??'{}').structuredContent??{};
 return !response.isError&&(state.elements??[]).some(e=>/textfield|edit/i.test(e.role??''));
}
const refused=value=>value.refused===true&&value.side_effect_dispatched===false;
try{
 driver=CuaDriver.create(undefined);await driver.metadata();
 result.phase='fixture';
 const f=await fixture(),id=f.state.window_id;
 result.phase='initial_observe';
 let initialObserve=false;
 for(let i=0;i<8&&!initialObserve;i++){
  initialObserve=await observe(f,id);
  if(!initialObserve)await new Promise(r=>setTimeout(r,150));
 }
 assert.ok(initialObserve);
 result.phase='retain';const h=await retain(f,id);
 result.phase='normal';
 assert.ok(await observe(f,id));
 const normal=await command(h,'focus');
 await wait(()=>f.state.target_key&&f.state.target_on_active_space);
 assert.ok(normal.focused&&normal.geometry_unchanged);
 result.samples.normal={passed:true,retained_object:true,sdk_observe:true,geometry_unchanged:true};
 result.phase='contract_negatives';
 const changed=await command(h,'changed-start'),denied=await command(h,'deny-permission');
 assert.ok(refused(changed)&&changed.reason==='process_identity_changed');
 assert.ok(refused(denied)&&denied.reason==='permission_unavailable');
 result.samples.contract_negatives={passed:true,same_pid_changed_start_refused:true,permission_refused:true,synthetic_only:true};
 result.phase='ambiguous';
 f.process.stdin.write('overlap\n');await wait(()=>f.state.overlapping&&f.state.decoy_key);
 const ambiguous=await command(h,'focus');assert.ok(refused(ambiguous)&&ambiguous.reason==='mapping_not_unique');
 await new Promise(r=>setTimeout(r,150));assert.ok(f.state.decoy_key&&!f.state.target_key);
 result.samples.ambiguous={passed:true,reason:ambiguous.reason,decoy_untouched:true};
 f.process.stdin.write('separate\n');await wait(()=>!f.state.overlapping);
 result.phase='minimized';
 f.process.stdin.write('minimize\n');await wait(()=>f.state.target_minimized);
 const settledEnd=Date.now()+2000;
 while((await command(h,'verify')).valid!==true){
  assert.ok(Date.now()<settledEnd,'最小化后目标映射未就绪');
  await new Promise(r=>setTimeout(r,50));
 }
 const minimized=await command(h,'focus');result.minimized_native=minimized;
 result.minimized_fixture={key:f.state.target_key,minimized:f.state.target_minimized};
 await wait(()=>f.state.target_key&&!f.state.target_minimized);
 assert.ok(minimized.focused&&minimized.restored&&minimized.geometry_unchanged&&await observe(f,id));
 result.samples.minimized={passed:true,geometry_unchanged:true,sdk_observe:true};
 result.phase='closed';
 f.process.stdin.write('close\n');await wait(()=>f.state.target_closed&&!f.state.target_key);
 const decoyBefore=f.state.decoy_key;
 const closed=await command(h,'focus');assert.ok(refused(closed));
 await new Promise(r=>setTimeout(r,150));assert.equal(f.state.decoy_key,decoyBefore);
 result.samples.closed={passed:true,reason:closed.reason,decoy_untouched:true};
 result.phase='replacement';
 f.process.stdin.write('replace\n');await wait(()=>f.state.replacement_visible&&f.state.replacement_key);
 const replaced=await command(h,'focus');assert.ok(refused(replaced));
 await new Promise(r=>setTimeout(r,150));assert.ok(f.state.replacement_key);
 const replacementId=f.state.replacement_window_id,newRef=await retain(f,replacementId);
 assert.ok(await observe(f,replacementId));const refreshed=await command(newRef,'verify');assert.ok(refreshed.valid);
 result.samples.replacement={passed:true,old_reference_refused:true,reason:replaced.reason,replacement_untouched:true,new_observe_valid:true};
 result.phase='restart';
 await stop(f);
 const restarted=await fixture();
 const exited=await command(h,'focus'),oldReplacement=await command(newRef,'focus');
 assert.ok(refused(exited)&&exited.reason==='process_identity_changed'&&refused(oldReplacement));
 const restartedRef=await retain(restarted,restarted.state.window_id);
 assert.ok((await command(restartedRef,'verify')).valid&&await observe(restarted,restarted.state.window_id));
 result.samples.restart={passed:true,old_reference_refused:true,new_observe_valid:true,pid_reuse_native_verified:false};
 result.passed=true;
}catch{result.failure='隔离工作身份验证失败';}
finally{
 for(const child of children)await stop(child);
 if(driver){try{await driver.shutdown();driver.uniffiDestroy();}catch{result.cleanup_failed=true;result.passed=false;}}
 writeFileSync(join(output,'result.json'),JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));
}
if(!result.passed)process.exitCode=1;
