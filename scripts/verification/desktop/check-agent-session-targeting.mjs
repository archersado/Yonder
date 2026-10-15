import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const html = readFileSync(new URL('../../../apps/desktop/ui/agent-settings.html', import.meta.url), 'utf8');
const settings = readFileSync(new URL('../../../apps/desktop/ui/agent-settings.js', import.meta.url), 'utf8');
const regionHtml = readFileSync(new URL('../../../apps/desktop/ui/region-preview.html', import.meta.url), 'utf8');
const region = readFileSync(new URL('../../../apps/desktop/ui/region-preview.js', import.meta.url), 'utf8');
const main = readFileSync(new URL('../../../apps/desktop/src/main.rs', import.meta.url), 'utf8');

for (const forbidden of ['在线输入会话', '当前接收目标', '选择接收会话', 'agent_session_list', 'agent_session_select', 'agent-target-required']) {
  assert.ok(!html.includes(forbidden), `Agent 管理不得包含 ${forbidden}`);
  assert.ok(!settings.includes(forbidden), `Agent 管理脚本不得包含 ${forbidden}`);
  assert.ok(!regionHtml.includes(forbidden), `圈选卡不得包含 ${forbidden}`);
  assert.ok(!region.includes(forbidden), `圈选脚本不得包含 ${forbidden}`);
  assert.ok(!main.includes(forbidden), `Desktop 命令不得包含 ${forbidden}`);
}
assert.ok(html.includes('已登记 Agent'), 'Agent 管理必须保留注册授权');
assert.ok(html.includes('语音回复'), 'Agent 管理必须保留语音回复设置');

console.log(JSON.stringify({ single_active_surface: true, session_picker_removed: true, settings_preserved: true }));
