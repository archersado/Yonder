import assert from 'node:assert/strict';

const task = await taskSpace('Yonder BU-S2 Windows reference UI 20261025');
const page = task.page('p1');
await page.cdp('Emulation.setDeviceMetricsOverride', { width: 1280, height: 800, deviceScaleFactor: 1, mobile: false });
await page.cdp('Page.addScriptToEvaluateOnNewDocument', { source: `
  window.fixtureBrowserError = false;
  window.browserOpenRequests = [];
  const task = {task_id:'bu-s2-windows-ui',name:'BU-S2 Windows引用',owner_agent_id:'bu-s2-agent',source:'local-agent',status:'running',sequence:'5',current_step:{step_id:'open-browser',label:'打开浏览器',accepted_sequence:'2'}};
  window.__TAURI_INTERNALS__ = { invoke: async (command, input) => {
    if (command === 'browser_task_space_open') { window.browserOpenRequests.push(input); return; }
    if (command !== 'task_query') return;
    const request = JSON.parse(input.request);
    const result = request.method === 'task.list' ? {kind:'tasks',tasks:[task],next_after_task_id:null}
      : request.method === 'task.step.get' ? {kind:'step',task,step:task.current_step}
      : request.method === 'task.events' ? {kind:'events',task_id:task.task_id,events:[{previous:'created',status:'running',sequence:'1'}]}
      : request.method === 'task.browser.get' ? (window.fixtureBrowserError ? (()=>{throw new Error('引用读取失败')})() : {kind:'browser-state',task,reference:{external_task_ref:'ego:8',ownership:'agent',managed_pages:1,finished:false,updated_sequence:'4'}})
      : null;
    return JSON.stringify({jsonrpc:'2.0',id:'bu-s2-ui',result});
  }};
` });
await page.goto('file:///F:/workspace/yonder/apps/desktop/ui/index.html');
await page.waitForFunction(() => document.querySelectorAll('.task').length === 1);
await page.click('.task', { label: '打开 Browser 引用详情' });
await page.waitForFunction(() => document.getElementById('detail').textContent.includes('ego:8 · Agent控制 · 1个托管页面 · 活动 · 更新序号 4'));
assert.equal(await page.evaluate(() => document.querySelector('.browser-open')?.textContent), '打开 ego-lite');
await page.click('.browser-open', { label: '交接 Browser Task Space' });
assert.deepEqual(await page.evaluate(() => window.browserOpenRequests), [{taskId:'bu-s2-windows-ui',expectedSequence:'5'}]);
assert.ok((await page.evaluate(() => document.getElementById('notice').textContent)).includes('已在 ego-lite 中打开对应任务'));
await page.evaluate(() => { window.fixtureBrowserError = true; });
await page.click('.task', { label: '刷新引用错误状态' });
await page.waitForFunction(() => document.getElementById('detail').textContent.includes('关联状态读取失败'));
assert.ok((await page.evaluate(() => document.getElementById('detail').textContent)).includes('任务 ID'));
console.log(JSON.stringify({ taskSpaceId: task.spaceId, browserReference: true, browserOpen: true, browserPartialFailure: true, passed: true }));
await task.finish({ keep: [] });
