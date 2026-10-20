import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';

const element=()=>({textContent:'',disabled:false,hidden:false,listeners:{},addEventListener(type,fn){this.listeners[type]=fn;}});
const elements=Object.fromEntries(['#status','#preview','#program','#args','#cwd','#env','#timeout','#approve','#reject'].map(key=>[key,element()]));
const events={};
const calls=[];
const preview={commandId:'command_1',program:'/usr/bin/printf',args:['fixed value'],cwd:'/tmp',env:{LANG:'zh_CN.UTF-8'},timeoutMs:5000,expiresAtMs:Date.now()+60000};
const window={
  __TAURI_INTERNALS__:{invoke:async(command,args)=>{calls.push({command,args});if(command==='command_approval_preview')return {...preview,commandId:args.commandId};return command==='command_approval_approve'?{commandId:args.commandId,state:'approved',expiresAtMs:preview.expiresAtMs}:undefined;}},
  addEventListener(type,fn){events[type]=fn;},
};
const document={querySelector:key=>elements[key]};
const setTimeout=fn=>{fn();return 1;};
vm.runInNewContext(readFileSync(new URL('../../../apps/desktop/ui/command-approval.js',import.meta.url),'utf8'),{window,document,setTimeout,Date,JSON,Object,Promise});

await events['yonda-command-approval-start']({detail:{taskId:'task_1',commandId:'command_1'}});
await new Promise(resolve=>setImmediate(resolve));
assert.equal(calls[0].command,'command_approval_preview');
assert.equal(JSON.stringify(calls[0].args),JSON.stringify({taskId:'task_1',commandId:'command_1'}));
assert.equal(elements['#preview'].hidden,false);
assert.equal(elements['#program'].textContent,'/usr/bin/printf');
assert.equal(elements['#args'].textContent,'"fixed value"');
assert.equal(elements['#cwd'].textContent,'/tmp');
assert.equal(elements['#env'].textContent,'LANG="zh_CN.UTF-8"');
assert.equal(elements['#timeout'].textContent,'5000 ms');
assert.equal(elements['#approve'].disabled,false);
await elements['#approve'].listeners.click();
assert.equal(calls.at(-2).command,'command_approval_approve');
assert.equal(calls.at(-1).command,'command_approval_close');
assert.match(elements['#status'].textContent,/已批准/);

await events['yonda-command-approval-start']({detail:{taskId:'task_2',commandId:'command_2'}});
await new Promise(resolve=>setImmediate(resolve));
await elements['#reject'].listeners.click();
assert.equal(calls.at(-2).command,'command_approval_reject');
assert.equal(calls.at(-1).command,'command_approval_close');
assert.match(elements['#status'].textContent,/已拒绝/);
console.log(JSON.stringify({desktopApproval:true,structuredPreview:true,approveOnce:true,reject:true,localClose:true}));
