// ego-browser 当前 TaskSpace 中调用 checkEdgeRest(page)；加速闲置等待，不改生产三分钟阈值。
import assert from 'node:assert/strict';
export async function checkEdgeRest(page) {
  const results = [];
  await page.cdp('Emulation.setDeviceMetricsOverride', { width: 200, height: 200, deviceScaleFactor: 3, mobile: false });
  for (const edge of ['bottom', 'left', 'right', 'top']) {
    const { identifier } = await page.cdp('Page.addScriptToEvaluateOnNewDocument', { source: `
      window.calls = []; window.nativeVisible = true;
      window.__TAURI_INTERNALS__ = { invoke: async command => {
        window.calls.push(command);
        return command === 'pet_dock' ? ${JSON.stringify(edge)} : window.nativeVisible;
      }};
      const timer = window.setTimeout;
      window.setTimeout = (fn, ms, ...args) => timer(fn, ms === 180000 ? 900 : ms, ...args);
    ` });
    await page.goto(new URL('./ui/index.html', import.meta.url).href);
    const result = await page.evaluate(async () => {
      await new Promise(resolve => setTimeout(resolve, 1700));
      const pet = document.querySelector('#pet');
      if (pet.dataset.mode !== 'docked') throw Error('未自动停靠');
      const blinking = await new Promise((resolve, reject) => {
        const observer = new MutationObserver(() => {
          if (pet.classList.contains('blinking')) { observer.disconnect(); clearTimeout(timeout); resolve(true); }
        });
        const timeout = setTimeout(() => { observer.disconnect(); reject(Error('躲藏状态未自然眨眼')); }, 5500);
        observer.observe(pet, { attributes: true, attributeFilter: ['class'] });
      });
      return { edge: pet.dataset.edge, width: pet.offsetWidth, height: pet.offsetHeight, blinking };
    });
    assert.deepEqual(result, { edge, width: ['left', 'right'].includes(edge) ? 56 : 112, height: ['left', 'right'].includes(edge) ? 112 : 56, blinking: true });
    // 自然时序已验证；静态强制闭眼仅用于检查配准和透明遮罩。
    await page.evaluate(() => {
      document.body.style.background = '#243040';
      document.querySelector('.peek-eyelids').style.opacity = '1';
    });
    await page.screenshot({ path: new URL(`./evidence/animations-macos/edge-${edge}-closed.png`, import.meta.url).pathname });
    await page.mouse.click(20, 25, { label: '唤醒边缘小龙' });
    assert.equal(await page.evaluate(async () => {
      await new Promise(resolve => setTimeout(resolve, 450));
      return document.querySelector('#pet').dataset.mode;
    }), 'awake');
    await page.cdp('Page.removeScriptToEvaluateOnNewDocument', { identifier });
    results.push({ ...result, wake: true });
  }
  return results;
}
