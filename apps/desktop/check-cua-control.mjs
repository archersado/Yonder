import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';

const element=()=>({textContent:'',disabled:false,hidden:false,className:'',listeners:{},children:[],addEventListener(type,fn){this.listeners[type]=fn;},replaceChildren(...children){this.children=children;}});
const elements=Object.fromEntries(['#title','#slow-brain','#fast-brain','#step','#takeover','#steps','#plan','#plan-note'].map(key=>[key,element()]));
const events={};
const calls=[];
const window={
  __TAURI_INTERNALS__:{invoke:async(command,args)=>{calls.push({command,args});return command==='cua_control_presentation'?presentation:'accepted';}},
  addEventListener(type,fn){events[type]=fn;},
};
const document={querySelector:key=>elements[key],createElement:()=>element()};
const timers=[];
const setInterval=fn=>{timers.push(fn);return timers.length;};
const clearInterval=()=>{};
let presentation={task_id:'task-1',desktop_control:true,current_step:'打开企业微信',planned_steps:[{step_id:'open',label:'打开企业微信',state:'executing'},{step_id:'write',label:'填写问候',state:'pending'}],remaining_steps:2,plan_status:'available',slow_brain_summary:'慢脑已提交 2 个受限步骤',fast_brain_summary:'Jev 已从 2 个候选中选择：打开应用'};
vm.runInNewContext(readFileSync(new URL('./ui/cua-control.js',import.meta.url),'utf8'),{window,document,setInterval,clearInterval});
await new Promise(resolve=>setImmediate(resolve));

assert.equal(calls[0].command,'cua_control_presentation');
assert.equal(elements['#title'].textContent,'Yonder 正在控制您的电脑');
assert.equal(elements['#slow-brain'].textContent,'慢脑已提交 2 个受限步骤');
assert.equal(elements['#fast-brain'].textContent,'Jev 已从 2 个候选中选择：打开应用');

events['yonda-cua-control-start']({detail:{presentation}});
assert.equal(elements['#title'].textContent,'Yonder 正在控制您的电脑');
assert.equal(elements['#step'].textContent,'打开企业微信');
assert.equal(elements['#takeover'].disabled,false);
assert.equal(elements['#steps'].children.length,2);
assert.equal(elements['#steps'].children[0].className,'executing');
assert.equal(elements['#plan-note'].textContent,'另有 2 步');
presentation={...presentation,current_step:'填写问候',fast_brain_summary:'Jev 已从 2 个候选中选择：输入文本',planned_steps:[{step_id:'open',label:'打开企业微信',state:'completed'},{step_id:'write',label:'填写问候',state:'executing'}],remaining_steps:1};
await timers[0]();
assert.equal(elements['#steps'].children[0].className,'completed');
assert.equal(elements['#step'].textContent,'填写问候');
assert.equal(elements['#fast-brain'].textContent,'Jev 已从 2 个候选中选择：输入文本');
await elements['#takeover'].listeners.click();
const takeoverCall=calls.at(-1);
assert.equal(takeoverCall.command,'cua_control_takeover');
assert.equal(takeoverCall.args.taskId,'task-1');
assert.equal(elements['#title'].textContent,'正在安全停止');
assert.equal(elements['#takeover'].disabled,true);
await timers[0]();
assert.equal(elements['#title'].textContent,'正在安全停止');
assert.equal(elements['#step'].textContent,'完成当前操作并观察后交回控制');
assert.equal(elements['#takeover'].disabled,true);
events['yonda-cua-control-result']({detail:'failed'});
assert.equal(elements['#title'].textContent,'未能安全停止');
assert.equal(elements['#takeover'].textContent,'接管失败');
presentation={task_id:'browser-task',desktop_control:false,current_step:'核验公开页面',planned_steps:[],remaining_steps:0,plan_status:'none',slow_brain_summary:'慢脑已提交单步执行',fast_brain_summary:'单步请求无需 Jev 选择'};
events['yonda-cua-control-start']({detail:{presentation}});
assert.equal(elements['#title'].textContent,'Yonder 正在执行任务');
assert.equal(elements['#step'].textContent,'核验公开页面');
assert.equal(elements['#takeover'].hidden,true);
assert.equal(elements['#takeover'].disabled,true);
assert.equal(elements['#plan-note'].textContent,'未提供后续计划');
console.log(JSON.stringify({projectionReplay:true,slowBrain:true,fastBrain:true,start:true,plannedSteps:true,executionProgress:true,explicitTakeoverOnly:true,nonCuaVisible:true,nonCuaTakeoverHidden:true,pending:true,failure:true}));
