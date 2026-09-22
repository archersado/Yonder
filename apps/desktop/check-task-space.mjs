// 仅用于ego-browser测试：注入明确测试夹具，不进入产品任务库。
import assert from 'node:assert/strict';
export async function checkPetInteraction(page) {
  // 固定大于小龙的测试画布，body拖放终点不能与小龙中心重合。
  await page.cdp('Emulation.setDeviceMetricsOverride',{width:640,height:480,deviceScaleFactor:1,mobile:false});
  await page.cdp('Page.addScriptToEvaluateOnNewDocument', {source:`
    window.nativeCalls=[];window.nativeArguments=[];
    window.fixtureHasTasks=false;window.fixtureState='idle';window.fixtureFailure=false;
    window.emitPresentation=(hasTasks,state,extra={})=>window.dispatchEvent(new CustomEvent('yonda-presentation',{detail:{hasTasks,state,...extra}}));
    window.__TAURI_INTERNALS__={invoke:async (command,args)=>{
      window.nativeCalls.push(command);
      window.nativeArguments.push({command,args});
      if(command==='pet_is_visible')return true;
      if(command==='pet_task_state') {if(window.fixtureFailure)throw Error('test');return [window.fixtureHasTasks,window.fixtureState];}
      if(command==='task_menu_show')return window.fixtureHasTasks;
      if(command==='pet_dock')return 'bottom';
    }};`});
  await page.reload();
  await page.waitForFunction(() => document.getElementById('pet').dataset.state === 'idle');
  await page.waitForTimeout(300);
  assert.equal(await page.evaluate(() => window.nativeCalls.filter(c=>c==='task_menu_show').length),0);
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
  await page.evaluate(() => {window.fixtureHasTasks=true;window.fixtureState='executing';window.emitPresentation(true,'executing');});
  await page.waitForFunction(() => document.getElementById('pet').dataset.state==='executing');
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
  assert.equal(await page.evaluate(() => window.nativeCalls.filter(c=>c==='task_menu_show').length),1);
  await page.evaluate(() => {window.fixtureState='waiting_for_user';window.emitPresentation(true,'waiting_for_user');});
  await page.waitForFunction(() => document.getElementById('pet').dataset.state==='waiting_for_user');
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
  assert.equal(await page.evaluate(() => window.nativeCalls.filter(c=>c==='task_menu_show').length),2);
  await page.dragAndDrop('#pet','body');
  assert.equal(await page.evaluate(() => window.nativeCalls.filter(c=>c==='task_menu_show').length),2);
  assert.ok(await page.evaluate(() => window.nativeCalls.includes('plugin:window|start_dragging')));
  await page.evaluate(() => window.emitPresentation(true,'unknown'));
  await page.waitForFunction(() => document.getElementById('pet').dataset.state==='unknown');
  await page.waitForFunction(() => document.querySelector('.state-frame').src.endsWith('idle-00.png'));
  await page.cdp('Emulation.setEmulatedMedia',{features:[]});
  return {hoverDoesNotOpen:true,executingAnimation:true,continuousLocalPaws:true,stableExecutingBody:true,rightClickOpens:true,waitingAndPaused:true,continuousPausedWing:true,terminalSuccessAndFailure:true,reducedMotionStaticExecuting:true,clickDoesNotOpen:true,keyboardAndDrag:true,unknownOnFailure:true,fixtureOnly:true};
}
export async function checkTaskSpace(page) {
  await page.waitForFunction(() => document.getElementById('notice').textContent.includes('未提供') || document.querySelectorAll('.task').length > 0);
  await page.cdp('Page.addScriptToEvaluateOnNewDocument', { source: `
    window.fixtureError = false; window.fixtureTimelineError = false; window.fixtureTimelinePageError = false; window.fixtureBrowserError = false; window.fixtureListDelay = 0; window.fixtureControlRequests=[]; window.fixtureBrowserOpenRequests=[]; window.fixtureEventRequests=[];
    const tasks = window.fixtureTasks = Array.from({length:21}, (_,i) => ({task_id:'test-task-'+String(i).padStart(2,'0'), name:'test-task-'+String(i).padStart(2,'0'), owner_agent_id:'test-agent', status:'running', sequence:'4'}));
    tasks[20].sequence='5';
    window.__TAURI_INTERNALS__ = {invoke:async (command,input) => {
      if (command === 'user_takeover') {
        window.fixtureControlRequests.push(input);
        const task=tasks.find(task=>task.task_id===input.taskId);
        return JSON.stringify({jsonrpc:'2.0',id:'local-takeover',result:{kind:'control',task:{...task,sequence:'3'},control:{attempt_id:'attempt-1',control_id:'control_2',kind:'takeover',phase:'pending',accepted_sequence:'3',stopped_sequence:null}}});
      }
      if (command === 'browser_task_space_open') { window.fixtureBrowserOpenRequests.push(input); return; }
      if (command !== 'task_query') return;
      if (window.fixtureError) throw new Error('测试读取失败');
      const {method,params} = JSON.parse(input.request);
      if (method === 'task.list' && window.fixtureListDelay) {
        const delay = window.fixtureListDelay; window.fixtureListDelay = 0;
        await new Promise(resolve => setTimeout(resolve, delay));
      }
      if (method === 'task.events' && window.fixtureTimelineError) throw new Error('测试时间线失败');
      if (method === 'task.events' && params.after_sequence !== '0' && window.fixtureTimelinePageError) throw new Error('测试后续页失败');
      if (method === 'task.events') window.fixtureEventRequests.push(params);
      if (method === 'task.browser.get' && window.fixtureBrowserError) throw new Error('测试浏览器引用失败');
      const listed = method === 'task.list' && params.running_only ? tasks.filter(task => task.status === 'running') : tasks;
      const start = params.after_task_id ? listed.findIndex(task => task.task_id === params.after_task_id)+1 : 0;
      const result = method === 'task.control' ? (window.fixtureControlRequests.push(params),{kind:'control',task:{...tasks.find(task=>task.task_id===params.task_id),sequence:'3'},control:{attempt_id:'attempt-1',control_id:'control_2',kind:params.kind,phase:'pending',accepted_sequence:'3',stopped_sequence:null}})
        : method === 'task.step.get' ? {kind:'step',task:tasks.find(task => task.task_id === params.task_id),step:{step_id:'open-document',label:'打开目标文档',accepted_sequence:'2'}}
        : method === 'task.browser.get' ? {kind:'browser-state',task:tasks.find(task => task.task_id === params.task_id),reference:{external_task_ref:'ego:49',ownership:'agent',managed_pages:1,finished:false,updated_sequence:'3'}}
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
  await page.click('.task-actions button >> nth=0');
  await page.waitForFunction(() => document.querySelector('.task-actions button').textContent === '正在停止…');
  assert.deepEqual(await page.evaluate(() => window.fixtureControlRequests),[{taskId:'test-task-00',expectedSequence:'4'}]);
  assert.ok((await page.evaluate(() => document.getElementById('notice').textContent)).includes('确认边界后可接管'));
  await page.click('#refresh');
  await page.waitForFunction(() => [...document.querySelectorAll('.task-actions')][0].querySelectorAll('button')[0].textContent === '正在停止…' && [...document.querySelectorAll('.task-actions')][0].querySelectorAll('button')[1].disabled);
  await page.click('#next');
  await page.waitForFunction(() => document.querySelectorAll('.task').length === 1);
  assert.equal(await page.evaluate(() => document.querySelector('.task strong').textContent), 'test-task-20');
  await page.click('.task');
  await page.waitForFunction(() => document.querySelector('#detail h2'));
  const detailText=await page.evaluate(() => document.getElementById('detail').textContent);
  assert.ok(detailText.includes('打开目标文档') && detailText.includes('open-document · 接受序号 2'));
  assert.ok(detailText.includes('Agent 声明步骤：打开目标文档') && detailText.includes('动作已观察：成功') && detailText.includes('执行结果未知：执行超时'));
  assert.ok(detailText.includes('状态说明') && detailText.includes('执行结果未知：执行超时'));
  assert.ok(detailText.includes('ego:49 · Agent控制 · 1个托管页面 · 活动 · 更新序号 3'));
  await page.click('.browser-open');
  assert.deepEqual(await page.evaluate(() => window.fixtureBrowserOpenRequests),[{taskId:'test-task-20',expectedSequence:'5'}]);
  assert.ok(detailText.includes('仍有记录未加载'));
  await page.evaluate(() => { window.fixtureTimelinePageError=true; });
  await page.click('.timeline-more button');
  await page.waitForFunction(() => document.querySelector('.timeline-more .timeline-error'));
  assert.equal(await page.evaluate(() => document.querySelectorAll('.timeline li').length),3);
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
  await page.evaluate(() => {
    window.fixtureTasks.slice(0,20).forEach(task => { task.status='interrupted'; });
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
  return {capabilityUnavailable:true,pendingTakeover:true,pendingSurvivesRefresh:true,paging:true,detail:true,timeline:true,timelinePagination:true,timelinePartialFailure:true,browserReference:true,browserOpen:true,browserPartialFailure:true,staleError:true,latestResponseWins:true,filterReset:true,runningOnly:true,otherStatesInAll:true,fixtureOnly:true};
}
