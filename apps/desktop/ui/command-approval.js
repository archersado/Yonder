(()=>{'use strict';
const invoke=window.__TAURI_INTERNALS__?.invoke;
const status=document.querySelector('#status');const preview=document.querySelector('#preview');
const program=document.querySelector('#program');const args=document.querySelector('#args');const cwd=document.querySelector('#cwd');const env=document.querySelector('#env');const timeout=document.querySelector('#timeout');
const approve=document.querySelector('#approve');const reject=document.querySelector('#reject');
let taskId=null;let commandId=null;let pending=false;
function clear(){preview.hidden=true;program.textContent='';args.textContent='';cwd.textContent='';env.textContent='';timeout.textContent='';approve.disabled=true;reject.disabled=true;}
function render(value){program.textContent=value.program;args.textContent=value.args.length?value.args.map(item=>JSON.stringify(item)).join(' '):'（无）';cwd.textContent=value.cwd;const entries=Object.entries(value.env);env.textContent=entries.length?entries.map(([key,item])=>`${key}=${JSON.stringify(item)}`).join(' '):'（无）';timeout.textContent=`${value.timeoutMs} ms`;preview.hidden=false;approve.disabled=false;reject.disabled=false;status.textContent='请核对后决定；当前尚未执行。';}
async function begin(value){taskId=value?.taskId??null;commandId=value?.commandId??null;pending=false;clear();if(!invoke||!taskId||!commandId){status.textContent='命令授权信息不可用';return;}try{const value=await invoke('command_approval_preview',{taskId,commandId});if(value.commandId!==commandId||value.expiresAtMs<=Date.now())throw new Error('expired');render(value);}catch{status.textContent='命令授权已失效，请等待 Agent 重新提议。';}}
async function decide(command,message){if(!invoke||!taskId||!commandId||pending)return;pending=true;approve.disabled=true;reject.disabled=true;try{await invoke(command,{taskId,commandId});status.textContent=message;preview.hidden=true;await new Promise(resolve=>setTimeout(resolve,700));await invoke('command_approval_close');}catch{status.textContent='授权操作未完成，请重试。';approve.disabled=false;reject.disabled=false;}finally{pending=false;}}
approve.addEventListener('click',()=>decide('command_approval_approve','已批准，等待归属 Agent 执行。'));
reject.addEventListener('click',()=>decide('command_approval_reject','已拒绝，本次命令不会执行。'));
window.addEventListener('yonda-command-approval-start',event=>void begin(event.detail));
})();
