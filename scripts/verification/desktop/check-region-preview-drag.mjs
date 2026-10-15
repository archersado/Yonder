import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const html = readFileSync(new URL('../../../apps/desktop/ui/region-preview.html', import.meta.url), 'utf8');
const script = readFileSync(new URL('../../../apps/desktop/ui/region-preview.js', import.meta.url), 'utf8');
const style = readFileSync(new URL('../../../apps/desktop/ui/region-preview.css', import.meta.url), 'utf8');
const config = JSON.parse(readFileSync(new URL('../../../apps/desktop/tauri.conf.json', import.meta.url), 'utf8'));
const region = config.app.security.capabilities.find(item => item.identifier === 'region-preview-local');

assert.match(html, /id="review-header"/);
assert.match(script, /plugin:window\|start_dragging/);
assert.match(script, /releasePointerCapture/);
assert.match(style, /#review-header\{[^}]*cursor:move/);
assert.ok(region?.permissions.includes('core:window:allow-start-dragging'));
assert.ok(!region?.permissions.includes('core:window:allow-set-position'));

console.log(JSON.stringify({ title_drag_region: true, native_drag_command: true, pointer_capture_released: true, least_privilege: true }));
