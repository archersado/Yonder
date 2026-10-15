// 独立预览验证：只读取视觉采样，不连接任务库。
import assert from 'node:assert/strict';
export async function checkAnimationPreview(page, output) {
  const top=['idle','listening','thinking','executing','waiting_for_user','paused'];
  const bottom=['recording','success','failed'];
  await page.goto('file:///private/tmp/yonda-state-animation-preview.html');
  await page.waitForFunction(()=>Object.keys(window.previewChecks).length===9);
  // 待命摇尾8秒一轮；连续观察超过两轮，不修改视觉时钟。
  await page.waitForTimeout(17000);
  const verify=async states=>{
    await page.waitForFunction(states=>states.every(s=>{const v=window.previewChecks[s];return v?.ready&&v.breathChanged&&v.blinkSeen&&(v.actionChanged||v.pawsChanged||v.headChanged)}),states,{timeout:12000});
    const checks=await page.evaluate(()=>window.previewChecks);
    for(const state of states) assert.ok(checks[state].ready&&checks[state].breathChanged&&checks[state].blinkSeen&&(checks[state].actionChanged||checks[state].pawsChanged||checks[state].headChanged),state);
    for(const state of states.filter(s=>s==='paused')) assert.ok(checks[state].gestureChanged,state+' must move limb independently');
    if(states.includes('waiting_for_user')) assert.equal(checks.waiting_for_user.poses.length,4,'waiting user must raise and lower a complete paw');
    for(const state of states.filter(s=>['listening','thinking','success','failed'].includes(s))) assert.ok(checks[state].actionChanged,state+' must move intact body');
    for(const state of states.filter(s=>s!=='waiting_for_user')) assert.equal(checks[state].maxMix,0,state+' must not crossfade whole images');
    if(states.includes('listening')) { assert.equal(checks.listening.cue,'flex');assert.ok(checks.thinking.dotsChanged);assert.equal(checks.paused.cue,'flex');assert.equal(checks.waiting_for_user.cue,'none'); }
    if(states.includes('recording')) {
      await page.waitForFunction(()=>window.previewChecks.recording?.propChanged&&window.previewChecks.recording?.flashSeen,undefined,{timeout:12000});
      const recording=await page.evaluate(()=>window.previewChecks.recording);assert.ok(recording.propChanged&&recording.flashSeen);assert.equal(recording.record,'flex');
    }
    return await page.evaluate(()=>window.previewChecks);
  };
  await verify(top);
  await page.screenshot({path:output+'/top.png'});
  await page.mouse.move(1000,700);
  await page.mouse.wheel(0,650,{label:'检查下方状态重复播放'});
  await page.waitForTimeout(11000);
  const checks=await verify(bottom);
  await page.screenshot({path:output+'/bottom.png'});
  console.log(await page.snapshot());
  await page.click('loc=role:button[name="切换浅色背景"]');
  await page.waitForFunction(()=>document.body.classList.contains('light'));
  await page.screenshot({path:output+'/light.png'});
  return {result:'PASS',scope:'九态预览视觉采样；不代表原生或真实事件接线通过',top_observation_ms:17000,bottom_observation_ms:11000,checks};
}

export async function checkLifecycleInterruptions(page, output) {
  await page.cdp('Emulation.setEmulatedMedia',{media:'',features:[{name:'prefers-reduced-motion',value:'no-preference'}]});
  await page.goto('file:///private/tmp/yonda-transition-check.html');
  await page.waitForFunction(()=>document.querySelector('#pet').classList.contains('frames-ready')&&document.querySelector('#pet').dataset.state==='recording');
  await page.evaluate(()=>{window.flashObserved=false;const el=document.querySelector('.camera-flash');window.flashObserver=new MutationObserver(()=>{if(Number(el.style.opacity)>.3)window.flashObserved=true;});window.flashObserver.observe(el,{attributes:true,attributeFilter:['style']});});
  await page.waitForFunction(()=>window.flashObserved,undefined,{timeout:12000});
  await page.evaluate(()=>window.flashObserver.disconnect());
  await page.evaluate(()=>{window.previewState='waiting_for_user';});
  await page.waitForFunction(()=>document.querySelector('#pet').dataset.state==='waiting_for_user');
  assert.equal(await page.evaluate(()=>getComputedStyle(document.querySelector('.state-prop')).display),'none');
  assert.equal(await page.evaluate(()=>Number(document.querySelector('.camera-flash').style.opacity)),0);
  await page.evaluate(()=>{window.previewState='recording';});
  await page.waitForFunction(()=>document.querySelector('#pet').dataset.state==='recording');
  await page.cdp('Emulation.setEmulatedMedia',{media:'',features:[{name:'prefers-reduced-motion',value:'reduce'}]});
  await page.waitForFunction(()=>matchMedia('(prefers-reduced-motion: reduce)').matches&&document.querySelector('.state-prop').style.transform==='');
  const still=()=>page.evaluate(()=>({prop:getComputedStyle(document.querySelector('.state-prop')).display,record:getComputedStyle(document.querySelector('.record-badge')).display,transform:document.querySelector('.state-prop').style.transform,flash:Number(document.querySelector('.camera-flash').style.opacity),breath:document.querySelector('.breath').style.transform}));
  const before=await still();await page.waitForTimeout(1200);assert.deepEqual(await still(),before);
  assert.equal(before.prop,'block');assert.equal(before.record,'flex');assert.equal(before.flash,0);
  await page.screenshot({path:output+'/recording-reduced.png'});
  await page.evaluate(()=>{window.previewState='paused';});
  await page.waitForFunction(()=>document.querySelector('#pet').dataset.state==='paused');
  assert.equal(await page.evaluate(()=>getComputedStyle(document.querySelector('.state-gesture')).visibility),'hidden','暂停静态姿态不能被展开翼片覆盖');
  await page.screenshot({path:output+'/paused-reduced.png'});
  await page.cdp('Emulation.setEmulatedMedia',{media:'',features:[{name:'prefers-reduced-motion',value:'no-preference'}]});
  await page.waitForFunction(()=>document.querySelector('.state-gesture').style.transform.includes('rotateY'));
  await page.evaluate(()=>{window.previewState='idle';});
  await page.waitForFunction(()=>document.querySelector('#pet').dataset.state==='idle');
  assert.equal(await page.evaluate(()=>getComputedStyle(document.querySelector('.state-gesture')).display),'none');
  return {result:'PASS',scope:'独立夹具，录制打断与减少动态效果；不覆盖真实系统设置',flashClearedOnExit:true,recordingStatic:true,pausedWingsFolded:true,decorationRemoved:true};
}

export async function checkDockedStateWake(page, output) {
  await page.goto('file:///private/tmp/yonda-docked-state-change.html');
  await page.waitForFunction(()=>document.querySelector('#pet').dataset.mode==='docked');
  assert.equal(await page.evaluate(()=>window.previewWakeCount),0);
  await page.evaluate(()=>window.previewState='waiting_for_user');
  await page.waitForFunction(()=>window.previewWakeCount===1&&document.querySelector('#pet').dataset.mode==='awake',undefined,{timeout:3000});
  await page.waitForFunction(()=>document.querySelector('#pet').dataset.mode==='docked',undefined,{timeout:3000});
  assert.equal(await page.evaluate(()=>window.previewWakeCount),1,'相同状态轮询不能重复唤醒');
  await page.evaluate(()=>window.previewFail=true);
  await page.waitForFunction(()=>window.previewWakeCount===2&&document.querySelector('#pet').dataset.state==='unknown',undefined,{timeout:3000});
  await page.screenshot({path:output+'/docked-state-wake.png'});
  return {result:'PASS',stateChangeWakeCount:1,unknownChangeWakeCount:1,repeatedStateWakeCount:0};
}
