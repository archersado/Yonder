import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const html = readFileSync(new URL('./ui/agent-settings.html', import.meta.url), 'utf8');
const script = readFileSync(new URL('./ui/agent-settings.js', import.meta.url), 'utf8');
const regionHtml = readFileSync(new URL('./ui/region-preview.html', import.meta.url), 'utf8');
const region = readFileSync(new URL('./ui/region-preview.js', import.meta.url), 'utf8');

assert.ok(html.includes('id="agent-session-list"'), 'Agent 管理必须显示在线输入会话');
assert.ok(script.includes("invoke('agent_session_list')"), '会话列表必须来自正式 Hub 命令');
assert.ok(script.includes("invoke('agent_session_select'"), '选择按钮必须调用正式 Hub 命令');
assert.ok(script.includes('请选择圈选与语音输入的接收会话。'), '多会话时必须给出可执行提示');
assert.ok(script.includes("addEventListener('yonda-agent-sessions-changed'"), '连接变化必须刷新会话列表');
assert.ok(script.includes('当前接收目标'), '必须明确标识当前输入目标');
assert.ok(!html.includes('session_id') && !script.includes('session.session_id'), '界面不得暴露 Agent 会话标识');
assert.ok(regionHtml.includes('id="select-agent"'), '圈选目标缺失时必须提供直接选择入口');
assert.ok(region.includes("invoke('agent_settings_open')"), '圈选入口必须打开正式 Agent 管理');
assert.ok(region.includes("addEventListener('yonda-agent-target-changed'"), '选择完成后必须恢复圈选重试状态');

console.log(JSON.stringify({ list: true, explicit_selection: true, live_refresh: true, opaque_reference: true, region_recovery: true }));
