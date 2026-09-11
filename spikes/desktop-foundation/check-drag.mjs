import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

// 配置回归检查，不替代 Windows/macOS 原生鼠标拖动验证。
const config = JSON.parse(readFileSync(new URL('./src-tauri/tauri.conf.json', import.meta.url), 'utf8'));
const html = readFileSync(new URL('./ui/index.html', import.meta.url), 'utf8');
assert.match(html, /<main\s+data-tauri-drag-region(?:\s|>)/);
assert.match(html, /user-select:\s*none/);
assert.equal(config.app.windows.find(window => window.label === 'pet').decorations, false);
assert.deepEqual(config.app.security.capabilities.map(({ description, ...capability }) => capability), [{
  identifier: 'pet-drag', local: true, windows: ['pet'], permissions: ['core:window:allow-start-dragging'],
}]);
console.log('通过：桌宠拖动区域、禁用文本选择和最小本地窗口权限');
