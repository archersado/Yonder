// 仅用于ego-browser测试：注入明确测试夹具，不进入产品任务库。
import assert from 'node:assert/strict';
export async function checkPetInteraction(page) {
  // 固定大于小龙的测试画布，body拖放终点不能与小龙中心重合。
  await page.cdp('Emulation.setDeviceMetricsOverride',{width:640,height:480,deviceScaleFactor:1,mobile:false});
  await page.cdp('Page.addScriptToEvaluateOnNewDocument', {source:`
    window.nativeCalls=[];window.nativeArguments=[];
    window.fixtureHasTasks=false;window.fixtureState='idle';window.fixtureStepLabel=null;window.fixtureFailure=false;
    window.emitPresentation=(hasTasks,state,extra={})=>window.dispatchEvent(new CustomEvent('yonda-presentation',{detail:{hasTasks,state,...extra}}));
    window.__TAURI_INTERNALS__={invoke:async (command,args)=>{
      window.nativeCalls.push(command);
      window.nativeArguments.push({command,args});
      if(command==='pet_is_visible')return true;
      if(command==='pet_task_state') {if(window.fixtureFailure)throw Error('test');return [window.fixtureHasTasks,window.fixtureState,window.fixtureStepLabel];}
      if(command==='task_menu_show')return window.fixtureHasTasks;
      if(command==='pet_dock')return 'bottom';
    }};`});
  await page.reload();
  await page.waitForFunction(() => document.getElementById('pet').dataset.state === 'idle');
  await page.waitForTimeout(300);
  assert.equal(await page.evaluate(() => window.nativeCalls.filter(c=>c==='task_menu_show').length),0);
  await page.click('#pet', {button: 'right'});
  assert.equal(await page.evaluate(() => window.nativeCalls.filter(c=>c==='task_menu_show').length),1);
  await page.waitForFunction(() => document.getElementById('pet').classList.contains('frames-ready'));
  const idleSamples=[];
  for(let i=0;i<6;i++) {
    idleSamples.push(await page.evaluate(() => {
      const tail=document.querySelector('.state-tail'), style=getComputedStyle(tail);
      return {breath:document.querySelector('.breath').style.transform,
        tailClip:getComputedStyle(tail.querySelector('img')).clipPath,
        originX:parseFloat(style.transformOrigin),cutX:tail.getBoundingClientRect().width*.26};
    }));
    await page.waitForTimeout(100);
  }
  assert.ok(idleSamples.every(s=>s.breath.includes('translateY(')&&s.breath.includes('rotate(')&&s.breath.includes('scale(')));
  assert.ok(new Set(idleSamples.map(s=>s.breath)).size>=4);
  // 剪切边界必须与零位移轴重合，避免尾巴分层接缝随摆动张开。
  assert.ok(idleSamples.every(s=>s.tailClip==='inset(55% 73.75% 0px 0px)' && Math.abs(s.originX-s.cutX)<2));
  await page.evaluate(() => {window.fixtureHasTasks=true;window.fixtureState='executing';window.fixtureStepLabel='打开企业微信';window.emitPresentation(true,'executing',{stepLabel:window.fixtureStepLabel});});
  await page.waitForFunction(() => document.getElementById('pet').dataset.state==='executing');
  await page.waitForFunction(() => document.getElementById('task-step-status').textContent==='正在：打开企业微信');
  assert.ok((await page.evaluate(() => document.getElementById('pet').getAttribute('aria-label'))).includes('当前步骤：打开企业微信'));
  await page.evaluate(() => window.emitPresentation(true,'executing',{stepLabel:'给宫健的分身发送 hi'}));
  await page.waitForFunction(() => document.getElementById('task-step-status').textContent==='正在：给宫健的分身发送 hi');
  await page.waitForFunction(() => document.getElementById('pet').classList.contains('frames-ready'));
  const before=await page.evaluate(() => document.getElementById('pet').style.getPropertyValue('--state-paw-right')+document.getElementById('pet').style.getPropertyValue('--state-paw-left'));
  await page.waitForTimeout(400);
  assert.notEqual(await page.evaluate(() => document.getElementById('pet').style.getPropertyValue('--state-paw-right')+document.getElementById('pet').style.getPropertyValue('--state-paw-left')),before);
  assert.ok(await page.evaluate(() => document.querySelector('.state-frame').src.endsWith('executing-body.png')));
  const blendSamples=[];
  for(let i=0;i<12;i++) {
    blendSamples.push(await page.evaluate(() => ({current:Number(document.querySelector('.state-frame').style.opacity),next:Number(document.querySelector('.state-frame-next').style.opacity),mode:getComputedStyle(document.querySelector('.state-frame')).mixBlendMode,paw:getComputedStyle(document.querySelector('.state-paw-right')).transform+getComputedStyle(document.querySelector('.state-paw-left')).transform})));
    await page.waitForTimeout(30);
  }
  assert.ok(blendSamples.every(s=>Math.abs(s.current+s.next-1)<.00001 && s.mode==='normal'));
  assert.ok(new Set(blendSamples.map(s=>s.paw)).size>=4);
  await page.click('#pet');
  assert.equal(await page.evaluate(() => window.nativeCalls.filter(c=>c==='task_menu_show').length),0);
  await page.click('#pet', {button: 'right'});
  assert.equal(await page.evaluate(() => window.nativeCalls.filter(c=>c==='task_menu_show').length),2);
  await page.evaluate(() => {window.fixtureState='waiting_for_user';window.emitPresentation(true,'waiting_for_user');});
  await page.waitForFunction(() => document.getElementById('pet').dataset.state==='waiting_for_user');
  assert.equal(await page.evaluate(() => document.getElementById('task-step-status').textContent),'');
  assert.ok(!(await page.evaluate(() => document.getElementById('pet').getAttribute('aria-label'))).includes('当前步骤'));
  await page.waitForFunction(() => document.querySelector('.state-frame').src.includes('/lifecycle-v10/waiting_for_user-'));
  await page.evaluate(() => {window.fixtureState='paused';window.emitPresentation(true,'paused');});
  await page.waitForFunction(() => document.getElementById('pet').dataset.state==='paused');
  await page.waitForFunction(() => document.querySelector('.state-frame').src.endsWith('paused-03.png'));
  const wing=await page.evaluate(()=>document.querySelector('.state-gesture').style.transform);
  await page.waitForTimeout(200);
  assert.notEqual(await page.evaluate(()=>document.querySelector('.state-gesture').style.transform),wing);
  await page.evaluate(() => window.emitPresentation(false,'success',{eventId:'task-1:5:success',resumeHasTasks:false,resumeState:'idle'}));
  await page.waitForFunction(() => document.getElementById('pet').dataset.state==='success');
  await page.evaluate(() => window.emitPresentation(false,'success',{eventId:'task-1:5:success',resumeHasTasks:false,resumeState:'paused'}));
  await page.waitForTimeout(1850);
  assert.equal(await page.evaluate(() => document.getElementById('pet').dataset.state),'idle');
  await page.evaluate(() => window.emitPresentation(false,'failed',{eventId:'task-2:7:failed',resumeHasTasks:false,resumeState:'idle'}));
  await page.waitForFunction(() => document.getElementById('pet').dataset.state==='failed');
  await page.evaluate(() => window.emitPresentation(false,'failed',{eventId:'task-2:7:failed',resumeHasTasks:false,resumeState:'paused'}));
  await page.waitForTimeout(1850);
  assert.equal(await page.evaluate(() => document.getElementById('pet').dataset.state),'idle');
  await page.evaluate(() => window.emitPresentation(true,'paused'));
  await page.waitForFunction(() => document.getElementById('pet').dataset.state==='paused');
  await page.cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-reduced-motion',value:'reduce'}]});
  await page.waitForFunction(() => document.querySelector('.state-frame').src.endsWith('paused-03.png'));
  await page.waitForTimeout(450);
  assert.ok(await page.evaluate(() => document.querySelector('.state-frame').src.endsWith('paused-03.png')));
  await page.cdp('Emulation.setEmulatedMedia',{features:[{name:'prefers-reduced-motion',value:'reduce'}]});
  await page.evaluate(() => {window.fixtureState='executing';window.emitPresentation(true,'executing');});
  await page.waitForFunction(() => document.querySelector('.state-frame').src.endsWith('executing-body.png'));
  await page.waitForTimeout(600);
  assert.ok(await page.evaluate(() => document.querySelector('.state-frame').src.endsWith('executing-body.png')));
  await page.press('#pet','Enter');
  assert.equal(await page.evaluate(() => window.nativeCalls.filter(c=>c==='task_menu_show').length),3);
  await page.dragAndDrop('#pet','body');
  assert.equal(await page.evaluate(() => window.nativeCalls.filter(c=>c==='task_menu_show').length),4);
  assert.ok(await page.evaluate(() => window.nativeCalls.includes('plugin:window|start_dragging')));
  await page.evaluate(() => window.emitPresentation(true,'unknown'));
  await page.waitForFunction(() => document.getElementById('pet').dataset.state==='unknown');
  await page.waitForFunction(() => document.querySelector('.state-frame').src.endsWith('idle-00.png'));
  await page.cdp('Emulation.setEmulatedMedia',{features:[]});
  return {hoverDoesNotOpen:true,executingAnimation:true,currentStepOnPet:true,stepClearedOutsideRunning:true,continuousLocalPaws:true,stableExecutingBody:true,rightClickOpens:true,waitingAndPaused:true,continuousPausedWing:true,terminalSuccessAndFailure:true,reducedMotionStaticExecuting:true,clickDoesNotOpen:true,keyboardAndDrag:true,unknownOnFailure:true,fixtureOnly:true};
}
export async function checkTaskSpace(page) {
  await page.waitForFunction(() => document.getElementById('notice').textContent.includes('未提供') || document.querySelectorAll('.task').length > 0);
  assert.equal(await page.evaluate(() => document.querySelector('link[href*="jev-settings"], script[src*="jev-settings"]')), null);
  assert.equal(await page.evaluate(() => document.querySelector('#jev-form, .jev-capability, [aria-label$="Jev 设置"]')), null);
  await page.cdp('Page.addScriptToEvaluateOnNewDocument', { source: `
    window.nativeCalls=[]; window.fixtureError = false; window.fixtureTimelineError = false; window.fixtureTimelinePageError = false; window.fixtureArtifactPageError = false; window.fixtureObserveHistory = false; window.fixtureCreationHistory = false; window.fixtureAttemptStartHistory = false; window.fixtureBrowserError = false; window.fixtureListDelay = 0; window.fixtureControlRequests=[]; window.fixtureBrowserOpenRequests=[]; window.fixtureListRequests=[]; window.fixtureEventRequests=[]; window.fixtureArtifactRequests=[]; window.fixtureArtifactItems={}; window.fixtureConfirmRequests=[]; window.fixtureFileGrantRequests=[]; window.fixtureFileGrants=[]; window.fixtureCommandDecisions=[];
    window.fixtureCommandApprovals=[{taskId:'test-task-20',commandId:'command_fixture_1',state:'awaiting-user',expiresAtMs:Date.now()+60000,preview:{commandId:'command_fixture_1',program:'/usr/bin/printf',args:['%s','hello world'],cwd:'/tmp',env:{LANG:'zh_CN.UTF-8'},timeoutMs:1000,expiresAtMs:0}}];
    window.fixtureCommandApprovals[0].preview.expiresAtMs=window.fixtureCommandApprovals[0].expiresAtMs;
    const tasks = window.fixtureTasks = Array.from({length:21}, (_,i) => ({task_id:'test-task-'+String(i).padStart(2,'0'), name:'test-task-'+String(i).padStart(2,'0'), owner_agent_id:'test-agent', source:'local-agent', status:'running', sequence:'4'}));
    tasks[20].sequence='5';
    tasks[20].current_step={step_id:'open-document',label:'打开目标文档',accepted_sequence:'2'};
    tasks[20].observation={step_id:'open-document',result:'matched',summary:'文档已打开'};
    tasks[20].next_intent='编辑目标文档';
    tasks.splice(18,0,{task_id:'test-task--1',name:'test-task--1',owner_agent_id:'test-agent',source:'local-agent',status:'running',sequence:'4'});
    tasks.find(task=>task.task_id==='test-task-18').status='completed';
    window.__TAURI_INTERNALS__ = {invoke:async (command,input) => {
      window.nativeCalls.push(command);
      if (command === 'task_confirm') {
        window.fixtureConfirmRequests.push(input);
        const task=tasks.find(task=>task.task_id===input.taskId);
        if (!task) throw new Error('任务不存在');
        task.status='completed'; task.sequence='5';
        task.user_confirmation={task_id:task.task_id,confirmation_id:input.confirmationId,result_sequence:'4',manifest_version:'1',comment:input.comment,confirmed_by:'desktop'};
        task.artifact_manifest={task_id:task.task_id,version:'1',item_count:0};
        return;
      }
      if (command === 'user_takeover') {
        window.fixtureControlRequests.push(input);
        const task=tasks.find(task=>task.task_id===input.taskId);
        return JSON.stringify({jsonrpc:'2.0',id:'local-takeover',result:{kind:'control',task:{...task,sequence:'3'},control:{attempt_id:'attempt-1',control_id:'control_2',kind:'takeover',phase:'pending',accepted_sequence:'3',stopped_sequence:null}}});
      }
      if (command === 'browser_task_space_open') { window.fixtureBrowserOpenRequests.push(input); return; }
      if (command === 'file_grant_list') return window.fixtureFileGrants;
      if (command === 'file_grant_choose') {
        window.fixtureFileGrantRequests.push(input);
        const grant={grantId:'file_grant_fixture_'+window.fixtureFileGrantRequests.length,purpose:input.purpose,expiresAtMs:Date.now()+60000};
        window.fixtureFileGrants.push(grant); return grant;
      }
      if (command === 'file_grant_revoke') { window.fixtureFileGrants=window.fixtureFileGrants.filter(grant=>grant.grantId!==input.grantId); return; }
      if (command === 'command_approval_list') return window.fixtureCommandApprovals.filter(item=>item.taskId===input.taskId).map(({commandId,state,expiresAtMs})=>({commandId,state,expiresAtMs}));
      if (command === 'command_approval_preview') return window.fixtureCommandApprovals.find(item=>item.taskId===input.taskId&&item.commandId===input.commandId)?.preview;
      if (command === 'command_approval_approve') {
        window.fixtureCommandDecisions.push({decision:'approve',...input});
        const approval=window.fixtureCommandApprovals.find(item=>item.taskId===input.taskId&&item.commandId===input.commandId); approval.state='approved';
        return {commandId:approval.commandId,state:approval.state,expiresAtMs:approval.expiresAtMs};
      }
      if (command === 'command_approval_reject') { window.fixtureCommandDecisions.push({decision:'reject',...input}); window.fixtureCommandApprovals=window.fixtureCommandApprovals.filter(item=>item.taskId!==input.taskId||item.commandId!==input.commandId); return; }
      if (command !== 'task_query') return;
      if (window.fixtureError) throw new Error('测试读取失败');
      const {method,params} = JSON.parse(input.request);
      if (method === 'task.list' && window.fixtureListDelay) {
        const delay = window.fixtureListDelay; window.fixtureListDelay = 0;
        await new Promise(resolve => setTimeout(resolve, delay));
      }
      if (method === 'task.events' && window.fixtureTimelineError) throw new Error('测试时间线失败');
      if (method === 'task.events' && params.after_sequence !== '0' && window.fixtureTimelinePageError) throw new Error('任务历史不完整，请刷新后重试');
      if (method === 'task.events') window.fixtureEventRequests.push(params);
      if (method === 'task.list') window.fixtureListRequests.push(params);
      if (method === 'task.artifacts') {
        window.fixtureArtifactRequests.push(params);
        if (params.after_ordinal > 0 && window.fixtureArtifactPageError) throw new Error('测试产物分页失败');
        const items = window.fixtureArtifactItems[params.task_id] ?? [];
        const start = items.findIndex(item => item.ordinal > params.after_ordinal);
        const page = start < 0 ? [] : items.slice(start,start+params.limit);
        const next = start >= 0 && start+params.limit < items.length ? page.at(-1).ordinal : null;
        return JSON.stringify({jsonrpc:'2.0',id:'test-response',result:{kind:'artifact-manifest-page',task_id:params.task_id,manifest_version:params.manifest_version,items:page,next_after_ordinal:next}});
      }
      if (method === 'task.browser.get' && window.fixtureBrowserError) throw new Error('测试浏览器引用失败');
      const listed = method === 'task.list' && params.running_only ? tasks.filter(task => task.status === 'running') : tasks;
      const start = params.after_task_id ? listed.findIndex(task => task.task_id === params.after_task_id)+1 : 0;
      const result = method === 'task.control' ? (window.fixtureControlRequests.push(params),{kind:'control',task:{...tasks.find(task=>task.task_id===params.task_id),sequence:'3'},control:{attempt_id:'attempt-1',control_id:'control_2',kind:params.kind,phase:'pending',accepted_sequence:'3',stopped_sequence:null}})
        : method === 'task.step.get' ? {kind:'step',task:tasks.find(task => task.task_id === params.task_id),step:{step_id:'open-document',label:'打开目标文档',accepted_sequence:'2'}}
        : method === 'task.browser.get' ? {kind:'browser-state',task:tasks.find(task => task.task_id === params.task_id),reference:{external_task_ref:'ego:49',ownership:'agent',managed_pages:1,finished:false,updated_sequence:'3'}}
        : method === 'task.events' && window.fixtureCreationHistory ? {kind:'events',task_id:params.task_id,events:params.after_sequence === '0' ? [
          {previous:'created',status:'created',sequence:'1',creation_event:{owner_agent_id:'test-agent',source:'cloud-agent'}}] : []}
        : method === 'task.events' && window.fixtureAttemptStartHistory ? {kind:'events',task_id:params.task_id,events:params.after_sequence === '0' ? [
          {previous:'created',status:'running',sequence:'1',attempt_started:{step_id:'step-one',attempt_id:'attempt-one',worker_instance_id:'worker-secret',host_session_id:'host-secret'}}] : []}
        : method === 'task.events' && window.fixtureObserveHistory ? {kind:'events',task_id:params.task_id,events:params.after_sequence === '0' ? [
          {previous:'created',status:'created',sequence:'1',observation:{step_id:'first',result:'matched',summary:'已打开 <img src=x>'}},
          {previous:'running',status:'running',sequence:'2',observation:{step_id:'second',result:'not-matched',summary:'未找到目标'}},
          {previous:'running',status:'running',sequence:'3',observation:{step_id:'second',result:'unknown',summary:'观察中断'}}] : []}
        : method === 'task.events' ? {kind:'events',task_id:params.task_id,events:params.after_sequence === '0' ? [
          {previous:'created',status:'created',sequence:'1',step_declaration:{step_id:'open-document',label:'打开目标文档',accepted_sequence:'1'}},
          {previous:'running',status:'running',sequence:'2',attempt_result:{step_id:'open-document',attempt_id:'attempt-1',worker_instance_id:'worker-1',host_session_id:'host-1',phase:'observed',action_succeeded:true,observe_valid:true}},
          {previous:'running',status:'running',sequence:'3',attempt_result:{step_id:'open-document',attempt_id:'attempt-2',worker_instance_id:'worker-1',host_session_id:'host-1',phase:'unknown',observe_valid:false,unknown_reason:'timed-out'}}] : [
          {previous:'running',status:'waiting-for-user',sequence:'4',wait_reason:'请确认发送内容'},
          {previous:'waiting-for-user',status:'running',sequence:'5',attempt_result:{step_id:'open-document',attempt_id:'attempt-3',worker_instance_id:'worker-1',host_session_id:'host-1',phase:'observed',action_succeeded:false,observe_valid:true}}]}
        : {kind:'tasks',tasks:listed.slice(start,start+params.limit),next_after_task_id:start+params.limit<listed.length ? listed[start+params.limit-1].task_id : null};
      return JSON.stringify({jsonrpc:'2.0',id:'test-response',result});
    }};
  ` });
  await page.reload();
  await page.waitForFunction(() => document.querySelectorAll('.task').length === 20);
  assert.equal(await page.evaluate(() => window.fixtureListRequests.every(params => params.newest_first === true)), true);
  await page.evaluate(() => {
    window.fixtureTasks.unshift({task_id:'new-created-task',name:'新建任务即时可见',owner_agent_id:'test-agent',source:'local-agent',status:'created',sequence:'1'});
    window.dispatchEvent(new CustomEvent('yonda-tasks-open',{detail:{automatic:true}}));
  });
  await page.waitForFunction(() => document.getElementById('all').getAttribute('aria-pressed') === 'true' && [...document.querySelectorAll('.task strong')].some(item => item.textContent === '新建任务即时可见'));
  assert.equal(await page.evaluate(() => document.activeElement?.id === 'refresh'), false);
  await page.evaluate(() => { window.fixtureTasks.splice(window.fixtureTasks.findIndex(task => task.task_id === 'new-created-task'), 1); });
  await page.click('#ongoing');
  await page.waitForFunction(() => document.querySelectorAll('.task').length === 20);
  await page.click('.task-actions button >> nth=0');
  await page.waitForFunction(() => document.querySelector('.task-actions button').textContent === '正在停止…');
  assert.deepEqual(await page.evaluate(() => window.fixtureControlRequests),[{taskId:'test-task-00',expectedSequence:'4'}]);
  assert.ok((await page.evaluate(() => document.getElementById('notice').textContent)).includes('确认边界后可接管'));
  await page.click('#refresh');
  await page.waitForFunction(() => [...document.querySelectorAll('.task-actions')][0].querySelectorAll('button')[0].textContent === '正在停止…' && [...document.querySelectorAll('.task-actions')][0].querySelectorAll('button')[1].disabled);
  await page.click('#next');
  await page.waitForFunction(() => document.querySelectorAll('.task').length === 1);
  assert.equal(await page.evaluate(() => document.querySelector('.task strong').textContent), 'test-task-20');
  await page.click('.task >> nth=0');
  await page.waitForFunction(() => document.querySelector('#detail h2'));
  const detailText=await page.evaluate(() => document.getElementById('detail').textContent);
  assert.ok(detailText.includes('打开目标文档') && detailText.includes('open-document · 接受序号 2'));
  const details=await page.evaluate(() => Object.fromEntries([...document.querySelectorAll('#detail dt')].map(item => [item.textContent,item.nextElementSibling.textContent])));
  assert.equal(details['来源'],'本地 Agent');
  assert.equal(details['当前步骤'],'打开目标文档');
  assert.equal(details['观察摘要'],'已匹配 · 文档已打开');
  assert.equal(details['下一步意图'],'编辑目标文档');
  assert.ok(detailText.includes('Agent 声明步骤：打开目标文档') && detailText.includes('动作已观察：成功') && detailText.includes('执行结果未知：执行超时'));
  assert.ok(detailText.includes('状态说明') && detailText.includes('执行结果未知：执行超时'));
  assert.ok(detailText.includes('ego:49 · Agent控制 · 1个托管页面 · 活动 · 更新序号 3'));
  assert.ok(detailText.includes('Agent 请求执行结构化命令') && detailText.includes('/usr/bin/printf') && detailText.includes('"hello world"') && detailText.includes('LANG="zh_CN.UTF-8"'));
  await page.click('.command-approval-actions button:has-text("批准执行一次")');
  await page.waitForFunction(() => document.getElementById('notice').textContent.includes('仅归属 Agent 可执行一次'));
  assert.deepEqual(await page.evaluate(() => window.fixtureCommandDecisions[0]),{decision:'approve',taskId:'test-task-20',commandId:'command_fixture_1'});
  assert.ok(await page.evaluate(() => document.querySelector('.command-approval-state').textContent.includes('已批准')));
  await page.click('.command-approval-actions button:has-text("撤销批准")');
  await page.waitForFunction(() => document.getElementById('notice').textContent.includes('命令提议已拒绝'));
  assert.deepEqual(await page.evaluate(() => window.fixtureCommandDecisions[1]),{decision:'reject',taskId:'test-task-20',commandId:'command_fixture_1'});
  assert.ok(await page.evaluate(() => document.querySelector('.command-approval-empty').textContent.includes('暂无待处理命令')));
  assert.equal(await page.evaluate(() => document.querySelectorAll('.file-grant-actions button').length), 4);
  await page.click('.file-grant-actions button >> nth=0');
  await page.waitForFunction(() => document.getElementById('notice').textContent.includes('文件授权已签发'));
  assert.deepEqual(await page.evaluate(() => window.fixtureFileGrantRequests), [{taskId:'test-task-20',purpose:'read'}]);
  assert.ok(await page.evaluate(() => document.querySelector('.file-grants').textContent.includes('读取')));
  await page.click('.file-grants button');
  await page.waitForFunction(() => document.getElementById('notice').textContent.includes('文件授权已撤销'));
  assert.equal(await page.evaluate(() => document.querySelector('.file-grant-note').textContent.includes('文件位置不会显示')), true);
  await page.click('.browser-open');
  assert.deepEqual(await page.evaluate(() => window.fixtureBrowserOpenRequests),[{taskId:'test-task-20',expectedSequence:'5'}]);
  assert.ok(detailText.includes('仍有记录未加载'));
  await page.evaluate(() => { window.fixtureTimelinePageError=true; });
  await page.click('.timeline-more button');
  await page.waitForFunction(() => document.querySelector('.timeline-more .timeline-error'));
  assert.equal(await page.evaluate(() => document.querySelectorAll('.timeline li').length),3);
  assert.ok((await page.evaluate(() => document.querySelector('.timeline-more .timeline-error').textContent)).includes('任务历史不完整'));
  assert.equal(await page.evaluate(() => document.querySelector('.timeline-more button').textContent),'重试加载时间线');
  await page.evaluate(() => { window.fixtureTimelinePageError=false; });
  await page.click('.timeline-more button');
  await page.waitForFunction(() => document.querySelectorAll('.timeline li').length === 5 && !document.querySelector('.timeline-more'));
  assert.deepEqual(await page.evaluate(() => { const {task_id,after_sequence,limit}=window.fixtureEventRequests.slice(-1)[0]; return {task_id,after_sequence,limit}; }),{task_id:'test-task-20',after_sequence:'3',limit:20});
  assert.ok((await page.evaluate(() => document.getElementById('detail').textContent)).includes('等待用户：请确认发送内容') && (await page.evaluate(() => document.getElementById('detail').textContent)).includes('动作已观察：未达成'));
  await page.evaluate(() => { window.fixtureTimelineError=true; });
  await page.click('.task');
  await page.waitForFunction(() => document.querySelector('.timeline-error'));
  assert.ok((await page.evaluate(() => document.getElementById('detail').textContent)).includes('任务 ID'));
  await page.evaluate(() => { window.fixtureTimelineError=false; });
  await page.evaluate(() => { window.fixtureBrowserError=true; });
  await page.click('.task');
  await page.waitForFunction(() => document.getElementById('detail').textContent.includes('关联状态读取失败'));
  assert.ok((await page.evaluate(() => document.getElementById('detail').textContent)).includes('任务 ID'));
  await page.evaluate(() => { window.fixtureBrowserError=false; });
  await page.evaluate(() => { window.fixtureError=true; });
  await page.click('#refresh');
  await page.waitForFunction(() => document.getElementById('notice').textContent.includes('过期'));
  assert.equal(await page.evaluate(() => document.querySelectorAll('.task').length), 1);
  await page.evaluate(() => { window.fixtureError=false; });
  await page.click('#all');
  await page.waitForFunction(() => document.querySelectorAll('.task').length === 20);
  assert.equal(await page.evaluate(() => document.getElementById('all').getAttribute('aria-pressed')), 'true');
  assert.equal(await page.evaluate(() => document.getElementById('page-label').textContent), '第 1 页 · 20 项');
  await page.waitForTimeout(100);
  await page.click('.task:has-text("test-task-18")');
  await page.waitForFunction(() => document.getElementById('detail').textContent.includes('结果待确认'));
  assert.ok(await page.evaluate(() => document.getElementById('detail').textContent.includes('确认后生成首个清单')));
  await page.fill('[aria-label="结果确认意见"]', '原生验证：结果可用');
  await page.evaluate(() => document.querySelector('.confirm button')?.scrollIntoView({block: 'center'}));
  await page.click('.confirm button');
  await page.waitForFunction(() => document.getElementById('notice').textContent.includes('结果已确认，任务终态未改变'));
  const confirmRequest = await page.evaluate(() => window.fixtureConfirmRequests[0]);
  assert.equal(confirmRequest.taskId, 'test-task-18');
  assert.equal(confirmRequest.expectedSequence, '4');
  assert.equal(confirmRequest.comment, '原生验证：结果可用');
  assert.ok(typeof confirmRequest.confirmationId === 'string' && confirmRequest.confirmationId.length > 0);
  await page.waitForFunction(() => !document.getElementById('refresh').disabled && [...document.querySelectorAll('.task')].some(task => task.textContent.includes('test-task-18')));
  await page.click('.task:has-text("test-task-18")');
  await page.waitForFunction(() => document.getElementById('detail').textContent.includes('已确认 · 结果序号 4 · 清单版本 1 · 原生验证：结果可用'), undefined, {timeout:20000});
  assert.ok(await page.evaluate(() => document.getElementById('detail').textContent.includes('版本 1 · 0 项')));
  assert.ok(await page.evaluate(() => document.getElementById('detail').textContent.includes('该版本没有产物')));
  assert.equal(await page.evaluate(() => document.getElementById('detail').textContent.includes('产物已变化，需重新检查')),false);
  await page.evaluate(() => {
    const task=window.fixtureTasks.find(task=>task.task_id==='test-task-18');
    task.sequence='6'; task.artifact_manifest={task_id:task.task_id,version:'2',item_count:21};
    window.fixtureArtifactItems[task.task_id]=Array.from({length:21},(_,index)=>({ordinal:index+1,reference_id:'artifact-'+String(index+1).padStart(2,'0'),availability:['available','missing','changed','unverified'][index%4]}));
  });
  await page.click('.task:has-text("test-task-18")');
  await page.waitForFunction(() => document.querySelectorAll('.artifacts li').length === 20);
  assert.ok(await page.evaluate(() => document.getElementById('detail').textContent.includes('版本 2 · 21 项 · 产物已变化，需重新检查')));
  assert.ok(await page.evaluate(() => document.querySelector('.artifacts').textContent.includes('产物 2 · 缺失') && document.querySelector('.artifacts').textContent.includes('引用 artifact-03')));
  await page.evaluate(() => { window.fixtureArtifactPageError=true; });
  await page.click('.artifact-more button');
  await page.waitForFunction(() => document.querySelector('.artifact-more .artifact-error'));
  assert.equal(await page.evaluate(() => document.querySelectorAll('.artifacts li').length),20);
  assert.equal(await page.evaluate(() => document.querySelector('.artifact-more button').textContent),'重试加载产物');
  await page.evaluate(() => { window.fixtureArtifactPageError=false; });
  await page.click('.artifact-more button');
  await page.waitForFunction(() => document.querySelectorAll('.artifacts li').length === 21 && !document.querySelector('.artifact-more'));
  assert.deepEqual(await page.evaluate(() => { const {task_id,manifest_version,after_ordinal,limit}=window.fixtureArtifactRequests.slice(-1)[0]; return {task_id,manifest_version,after_ordinal,limit}; }),{task_id:'test-task-18',manifest_version:'2',after_ordinal:20,limit:20});
  await page.evaluate(() => {
    window.fixtureTasks.slice(0,21).forEach(task => { task.status='interrupted'; });
    window.fixtureTasks[0].status='paused'; window.fixtureTasks[1].status='cancelled';
  });
  await page.click('#ongoing');
  await page.waitForFunction(() => document.querySelectorAll('.task').length === 1);
  assert.ok(await page.evaluate(() => [...document.querySelectorAll('.status')].every(status => status.textContent === '执行中')));
  assert.equal(await page.evaluate(() => document.querySelector('.task strong').textContent), 'test-task-20');
  await page.click('#all');
  await page.waitForFunction(() => document.querySelectorAll('.task').length === 20);
  assert.deepEqual(await page.evaluate(() => [...document.querySelectorAll('.status')].slice(0,2).map(status => status.textContent)), ['已暂停','已取消']);
  await page.evaluate(() => {
    window.fixtureListDelay = 80;
    document.getElementById('ongoing').click();
    document.getElementById('all').click();
  });
  await page.waitForTimeout(120);
  assert.equal(await page.evaluate(() => document.querySelectorAll('.task').length), 20);
  assert.equal(await page.evaluate(() => document.getElementById('all').getAttribute('aria-pressed')), 'true');
  assert.equal(await page.evaluate(() => window.nativeCalls.some(call => call.startsWith('jev_'))), false);
  await page.evaluate(() => { window.fixtureObserveHistory = true; });
  await page.click('.task >> nth=0');
  await page.waitForFunction(() => document.querySelector('.timeline')?.textContent.includes('Observe（步骤 first）'));
  const observedHistory = await page.evaluate(() => ({text:document.querySelector('.timeline').textContent, imageCount:document.querySelector('.timeline').querySelectorAll('img').length}));
  assert.ok(observedHistory.text.includes('Observe（步骤 first）：已匹配 · 已打开 <img src=x>'));
  assert.ok(observedHistory.text.includes('Observe（步骤 second）：未匹配 · 未找到目标'));
  assert.ok(observedHistory.text.includes('Observe（步骤 second）：未知 · 观察中断'));
  assert.equal(observedHistory.imageCount,0);
  await page.evaluate(() => { window.fixtureObserveHistory = false; window.fixtureCreationHistory = true; });
  await page.click('.task >> nth=0');
  await page.waitForFunction(() => document.querySelector('.timeline')?.textContent.includes('任务创建：云端 Agent · Agent test-agent'));
  assert.ok((await page.evaluate(() => document.querySelector('.timeline').textContent)).includes('任务创建：云端 Agent · Agent test-agent'));
  await page.evaluate(() => { window.fixtureCreationHistory = false; window.fixtureAttemptStartHistory = true; });
  await page.click('.task >> nth=0');
  await page.waitForFunction(() => document.querySelector('.timeline')?.textContent.includes('执行尝试已准备'));
  const attemptStartText = await page.evaluate(() => document.querySelector('.timeline').textContent);
  assert.ok(attemptStartText.includes('执行尝试已准备（步骤 step-one · 尝试 attempt-one）'));
  assert.ok(!attemptStartText.includes('worker-secret') && !attemptStartText.includes('host-secret'));
  await page.evaluate(() => { window.fixtureAttemptStartHistory = false; });
  return {capabilityUnavailable:true,pendingTakeover:true,pendingSurvivesRefresh:true,paging:true,detail:true,presentationMetadata:true,commandApprovalPreview:true,commandApprovalApprove:true,commandApprovalRevoke:true,auditConfirmation:true,auditConfirmedProjection:true,artifactEmpty:true,artifactVersionChange:true,artifactPagination:true,artifactPartialFailure:true,timeline:true,timelinePagination:true,timelinePartialFailure:true,historicalObservation:true,historicalCreation:true,historicalAttemptStart:true,browserReference:true,browserOpen:true,browserPartialFailure:true,staleError:true,latestResponseWins:true,filterReset:true,runningOnly:true,otherStatesInAll:true,fixtureOnly:true};
}
