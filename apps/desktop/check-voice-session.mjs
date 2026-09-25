import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const direct = readFileSync(new URL('./ui/voice-input.js', import.meta.url), 'utf8');
const region = readFileSync(new URL('./ui/region-preview.js', import.meta.url), 'utf8');
const hint = '正在聆听，说完短暂停顿后自动发送整段';

assert.ok(direct.includes(hint), '直接语音卡必须说明停顿后整段发送');
assert.ok(region.includes(hint), '圈选语音必须说明停顿后整段发送');
assert.ok(!direct.includes('正在聆听（最长 20 秒）'), '总时限不得冒充静音结束语义');
assert.ok(!region.includes('正在聆听（最长 20 秒）'), '圈选入口不得把总时限冒充静音结束语义');
assert.equal((region.match(/if\(detail\.phase==='final'\)\{setVoiceActive\(false\);submitQuestion/g) || []).length, 1,
  '圈选入口只能从单个final分支提交整段文字');
assert.ok(region.includes("if(detail.text&&['listening','final'].includes(detail.phase))"),
  '部分结果只更新草稿，final复用整段草稿');

console.log(JSON.stringify({ direct_hint: true, region_hint: true, single_final_submit_branch: true }));
