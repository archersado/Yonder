import assert from 'node:assert/strict';
// 页面合约回归；加速测试闲置，不替代原生AXPress验收。
export async function checkAccessibility(page) {
  await page.cdp('Page.addScriptToEvaluateOnNewDocument', { source: `
    window.wakes=0;
    window.__TAURI_INTERNALS__={invoke:async command=>{
      if(command==='pet_wake') window.wakes++;
      return command==='pet_dock' ? 'bottom' : true;
    }};
    const timer=window.setTimeout;
    window.setTimeout=(fn,ms,...args)=>timer(fn,ms===180000?900:ms,...args);
  ` });
  await page.goto(new URL('./ui/index.html', import.meta.url).href);
  const result=await page.evaluate(async()=>{
    const wait=ms=>new Promise(resolve=>setTimeout(resolve,ms));
    await wait(1700);
    const pet=document.querySelector('#pet');
    if(pet.dataset.mode!=='docked') throw Error('休眠前置未满足');
    pet.click(); // 原生辅助功能的语义click形态，不注入pointerdown。
    await wait(450);
    const awake=pet.dataset.mode==='awake';
    pet.dispatchEvent(new MouseEvent('click',{bubbles:true,detail:1}));
    const noDuplicate=window.wakes===1 && !pet.classList.contains('responding');
    pet.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true,cancelable:true}));
    return {awake,wakes:window.wakes,noDuplicate,keyboardFeedback:pet.classList.contains('responding')};
  });
  assert.deepEqual(result,{awake:true,wakes:1,noDuplicate:true,keyboardFeedback:true});
  return result;
}
