import assert from 'node:assert/strict';

const task = await taskSpace('Yonder DS-S1 Windows idle dock repair 20261025');
const page = task.page('p1');
await page.cdp('Page.addScriptToEvaluateOnNewDocument', { source: `
  window.nativeCalls = [];
  window.presentationQueriedBeforeFramesReady = false;
  window.__TAURI_INTERNALS__ = { invoke: async command => {
    window.nativeCalls.push(command);
    if (command === 'pet_is_visible') return true;
    if (command === 'pet_task_state') {
      window.presentationQueriedBeforeFramesReady = document.getElementById('pet')?.classList.contains('frames-ready') !== true;
      return [false, 'idle'];
    }
    if (command === 'pet_agent_connected') return false;
    if (command === 'pet_dock') return 'bottom';
  }};
` });
await page.goto('file:///F:/workspace/yonder/apps/desktop/ui/pet.html');
await page.waitForFunction(() => document.getElementById('pet')?.classList.contains('frames-ready'));
await page.waitForFunction(() => document.getElementById('pet')?.dataset.state === 'idle');
const result = await page.evaluate(() => ({
  presentationQueriedBeforeFramesReady: window.presentationQueriedBeforeFramesReady,
  presentationQueryCount: window.nativeCalls.filter(command => command === 'pet_task_state').length,
  taskState: document.getElementById('pet').dataset.state,
}));
assert.equal(result.presentationQueriedBeforeFramesReady, false);
assert.equal(result.presentationQueryCount, 1);
assert.equal(result.taskState, 'idle');
console.log(JSON.stringify({ taskSpaceId: task.spaceId, ...result, passed: true }));
await task.finish({ keep: [] });
