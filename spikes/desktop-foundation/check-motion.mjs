// 在 ego-browser 的当前 TaskSpace 页面调用 checkMotion(page)，不创建另一个浏览器。
import assert from 'node:assert/strict';
export async function checkMotion(page) {
  await page.cdp('Page.addScriptToEvaluateOnNewDocument', { source: `
    window.nativeVisible = true;
    window.__TAURI_INTERNALS__ = { invoke: async () => window.nativeVisible };
    Object.defineProperty(document, 'hidden', { get: () => true });
  ` });
  await page.goto(new URL('./ui/index.html', import.meta.url).href);
  const result = await page.evaluate(async () => {
    const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
    await wait(600);
    const body = document.querySelector('.breath');
    const tail = document.querySelector('.tail');
    const before = [body.style.transform, tail.style.transform];
    await wait(300);
    const moving = before[0] !== body.style.transform;
    const wagging = before[1] !== tail.style.transform;
    window.nativeVisible = false;
    dispatchEvent(new Event('blur'));
    await wait(200);
    const stopped = body.style.transform === '' && tail.style.transform === '';
    return { moving, wagging, stopped };
  });
  assert.deepEqual(result, { moving: true, wagging: true, stopped: true });
  return result;
}
