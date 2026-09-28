import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';

const elements=Object.fromEntries(['#title','#step','#takeover'].map(key=>[key,{textContent:'',disabled:false,listeners:{},addEventListener(type,fn){this.listeners[type]=fn;}}]));
const events={};
const calls=[];
const window={
  __TAURI_INTERNALS__:{invoke:async(command,args)=>{calls.push({command,args});return 'accepted';}},
  addEventListener(type,fn){events[type]=fn;},
};
const document={querySelector:key=>elements[key]};
vm.runInNewContext(readFileSync(new URL('./ui/cua-control.js',import.meta.url),'utf8'),{window,document});

events['yonda-cua-control-start']({detail:{taskId:'task-1',stepLabel:'打开企业微信'}});
assert.equal(elements['#title'].textContent,'Yonder 正在控制您的电脑');
assert.equal(elements['#step'].textContent,'打开企业微信');
assert.equal(elements['#takeover'].disabled,false);
await elements['#takeover'].listeners.click();
assert.equal(calls.length,1);
assert.equal(calls[0].command,'cua_control_takeover');
assert.equal(calls[0].args.taskId,'task-1');
assert.equal(elements['#title'].textContent,'正在安全停止');
assert.equal(elements['#takeover'].disabled,true);
events['yonda-cua-control-result']({detail:'failed'});
assert.equal(elements['#title'].textContent,'未能安全停止');
assert.equal(elements['#takeover'].textContent,'接管失败');
console.log(JSON.stringify({start:true,step:true,explicitTakeoverOnly:true,pending:true,failure:true}));
