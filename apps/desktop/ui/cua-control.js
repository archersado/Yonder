(()=>{'use strict';const invoke=window.__TAURI_INTERNALS__?.invoke;const title=document.querySelector('#title');const slowBrain=document.querySelector('#slow-brain');const fastBrain=document.querySelector('#fast-brain');const step=document.querySelector('#step');const steps=document.querySelector('#steps');const plan=document.querySelector('#plan');const planNote=document.querySelector('#plan-note');const takeover=document.querySelector('#takeover');const confirmation=document.querySelector('#confirmation');const confirmationTarget=document.querySelector('#confirmation-target');const confirmationMessage=document.querySelector('#confirmation-message');const approveSend=document.querySelector('#approve-send');const rejectSend=document.querySelector('#reject-send');let taskId=null;let intentRef=null;let refreshTimer=null;let takeoverPending=false;
function render(value){taskId=value?.task_id??null;const desktopControl=value?.desktop_control===true;const replanning=value?.plan_status==='replanning';title.textContent=takeoverPending?'正在安全停止':replanning?'Yonder 正在重新规划':desktopControl?'Yonder 正在控制您的电脑':'Yonder 正在执行任务';takeover.hidden=!desktopControl;takeover.disabled=takeoverPending||!desktopControl||!taskId;slowBrain.textContent=value?.slow_brain_summary||'等待慢脑提交计划';fastBrain.textContent=value?.fast_brain_summary||'等待快脑评估当前候选';if(takeoverPending){step.textContent='完成当前操作并观察后交回控制';}else{step.textContent=value?.current_step||'正在执行当前步骤';}const items=value?.planned_steps||[];const status={completed:'已完成',executing:'执行中',deciding:'决策中',unverified:'待核实',pending:'等待'};steps.replaceChildren(...items.map(item=>{const node=document.createElement('li');node.className=item.state||'pending';node.textContent=item.label;node.setAttribute('aria-label',`${status[item.state]||status.pending}：${item.label}`);return node;}));plan.hidden=false;if(value?.plan_status==='none')planNote.textContent='未提供后续计划';else if(value?.plan_status==='unavailable')planNote.textContent='步骤信息暂不可用';else if(value?.plan_status==='awaiting-goal-verification')planNote.textContent='步骤已执行，等待慢脑核验最终结果';else if(replanning)planNote.textContent='旧计划已停止，等待慢脑提交新计划';else planNote.textContent=value?.remaining_steps?`另有 ${value.remaining_steps} 步`:'';}
let confirmationTask=null;let confirmationExpires=0;let confirmationStatus='';let confirmationPending=false;let confirmationRequest=0;
function clearConfirmation(){intentRef=null;confirmation.hidden=true;confirmationTarget.textContent='';confirmationMessage.textContent='';approveSend.disabled=true;rejectSend.disabled=true;}
function showConfirmationStatus(){if(confirmationStatus&&fastBrain.textContent.includes('等待用户在顶部浮窗确认发送')){fastBrain.textContent=confirmationStatus;if(!takeoverPending)step.textContent=confirmationStatus;}}
function begin(value){takeoverPending=false;takeover.textContent='接管电脑';render(value);clearInterval(refreshTimer);refreshTimer=setInterval(refresh,180);void refreshConfirmation();}
async function refresh(){if(!invoke)return;try{render(await invoke('cua_control_presentation'));await refreshConfirmation();}catch{}}
async function refreshConfirmation(){
 if(confirmationTask!==taskId){confirmationTask=taskId;confirmationExpires=0;confirmationStatus='';confirmationPending=false;clearConfirmation();}
 if(!invoke||!taskId)return;
 if(confirmationPending){showConfirmationStatus();return;}
 const requestedTask=taskId;const request=++confirmationRequest;let preview;
 try{preview=await invoke('cua_intent_confirmation',{taskId:requestedTask});}
 catch{if(request!==confirmationRequest||requestedTask!==taskId||confirmationPending)return;clearConfirmation();confirmationStatus='发送确认暂不可用，请稍后重试';showConfirmationStatus();return;}
 if(request!==confirmationRequest||requestedTask!==taskId||confirmationPending)return;
 if(preview&&preview.expiresAtMs>Date.now()){
  confirmationExpires=preview.expiresAtMs;confirmationStatus='';intentRef=preview.intentRef;confirmation.hidden=false;
  confirmationTarget.textContent=`发送给 ${preview.target}`;confirmationMessage.textContent=preview.message;approveSend.disabled=false;rejectSend.disabled=false;return;
 }
 if(preview)confirmationExpires=preview.expiresAtMs;
 clearConfirmation();
 if(!confirmationStatus.startsWith('已确认')&&!confirmationStatus.startsWith('已取消'))confirmationStatus=(preview||confirmationExpires&&confirmationExpires<=Date.now())?'发送确认已过期，请等待慢脑重新准备':'发送确认不可用，请等待慢脑重新准备';
 showConfirmationStatus();
}
window.addEventListener('yonda-cua-control-start',event=>begin(event.detail?.presentation));
window.addEventListener('yonda-cua-control-result',event=>{clearInterval(refreshTimer);if(event.detail==='failed'){title.textContent='未能安全停止';step.textContent='任务结果待核实，请在任务详情中查看';takeover.textContent='接管失败';takeover.disabled=true;}});
takeover.addEventListener('click',async()=>{if(!invoke||!taskId||takeover.disabled)return;takeoverPending=true;takeover.disabled=true;takeover.textContent='正在停止';title.textContent='正在安全停止';step.textContent='完成当前操作并观察后交回控制';try{await invoke('cua_control_takeover',{taskId});}catch{title.textContent='未能安全停止';step.textContent='任务结果待核实，请在任务详情中查看';takeover.textContent='接管失败';}});
async function confirmSend(command,status){if(!invoke||!taskId||!intentRef||confirmationPending||confirmationExpires<=Date.now())return;const requestedTask=taskId;const requestedIntent=intentRef;confirmationPending=true;++confirmationRequest;approveSend.disabled=true;rejectSend.disabled=true;try{await invoke(command,{taskId:requestedTask,intentRef:requestedIntent});if(taskId!==requestedTask)return;clearConfirmation();confirmationStatus=status;step.textContent=status;}catch{if(taskId!==requestedTask)return;clearConfirmation();confirmationStatus='发送确认未完成，请重新核对确认卡';step.textContent=confirmationStatus;}finally{if(taskId===requestedTask)confirmationPending=false;}}
approveSend.addEventListener('click',()=>confirmSend('cua_intent_approve','已确认发送，等待执行'));
rejectSend.addEventListener('click',()=>confirmSend('cua_intent_reject','已取消发送'));
async function bootstrap(){if(!invoke)return;try{begin(await invoke('cua_control_presentation'));}catch{}}
void bootstrap();
})();
