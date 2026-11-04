import { checkTaskSpace } from 'file:///F:/workspace/yonder/apps/desktop/check-task-space.mjs';

const task = await taskSpace('Yonder BU-S2 Windows reference UI 20261025');
const page = task.page('p1');
await page.goto('file:///F:/workspace/yonder/apps/desktop/ui/index.html');
const checks = await checkTaskSpace(page);
console.log(JSON.stringify({ taskSpaceId: task.spaceId, checks, passed: true }));
await task.finish({ keep: [] });
