import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const settingsHtml = readFileSync(new URL('./ui/agent-settings.html', import.meta.url), 'utf8');
const settings = readFileSync(new URL('./ui/agent-settings.js', import.meta.url), 'utf8');
const regionHtml = readFileSync(new URL('./ui/region-preview.html', import.meta.url), 'utf8');
const region = readFileSync(new URL('./ui/region-preview.js', import.meta.url), 'utf8');

assert.ok(settingsHtml.includes('id="voice-reply-enabled"'), 'Agent设置必须提供显式语音回复开关');
assert.ok(settingsHtml.includes('id="voice-reply-preview"'), 'Agent设置必须提供独立试听入口');
assert.ok(settings.includes("invoke('voice_reply_config_get')"), '设置打开时必须读取正式配置');
assert.ok(settings.includes("invoke('voice_reply_config_save'"), '设置变化必须保存正式配置');
assert.ok(settings.includes("invoke('voice_reply_preview')"), '试听必须调用正式语音 Runtime');
assert.ok(settings.includes('只使用本机已安装的中文声音'), '设置必须明确本地声音边界');
assert.ok(regionHtml.includes('id="voice-reply-stop"'), '回复卡必须直接提供停止朗读入口');
assert.ok(region.includes("invoke('voice_reply_stop')"), '停止入口必须调用正式运行时');
assert.ok(region.includes("addEventListener('yonda-voice-reply'"), '回复卡必须消费有界播放状态');
assert.ok(region.includes('本地语音不可用，已保留文字回复。'), 'TTS失败必须明确降级到文字');

console.log(JSON.stringify({ explicit_toggle: true, visible_stop: true, bounded_status: true }));
