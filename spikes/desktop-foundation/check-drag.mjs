import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

// 配置回归检查，不替代 Windows/macOS 原生鼠标拖动验证。
const config = JSON.parse(readFileSync(new URL('./src-tauri/tauri.conf.json', import.meta.url), 'utf8'));
const html = readFileSync(new URL('./ui/index.html', import.meta.url), 'utf8');
const css = readFileSync(new URL('./ui/pet.css', import.meta.url), 'utf8');
const script = readFileSync(new URL('./ui/pet.js', import.meta.url), 'utf8');
assert.match(html, /<main\s+id="pet"/);
assert.match(css, /user-select:\s*none/);
assert.match(script, /plugin:window\|start_dragging/);
assert.doesNotMatch(html, /data-tauri-drag-region/); // 避免按下即拖动吞掉单击。
// 外置样式必须被原生 CSP 放行，否则图片按原始尺寸溢出小窗口。
assert.match(html, /rel="stylesheet"\s+href="pet.css"/);
const stylePolicy = config.app.security.csp.split(';').map(part => part.trim()).find(part => part.startsWith('style-src '));
assert.ok(stylePolicy?.split(/\s+/).includes("'self'"), '原生 CSP 必须允许同源样式');
assert.equal(config.app.windows.find(window => window.label === 'pet').decorations, false);
assert.deepEqual(config.app.security.capabilities.map(({ description, ...capability }) => capability), [{
  identifier: 'pet-drag', local: true, windows: ['pet'], permissions: ['core:window:allow-start-dragging'],
}]);
console.log('通过：桌宠拖动区域、禁用文本选择和最小本地窗口权限');
