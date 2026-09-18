// 运行 node spikes/desktop-foundation/check-motion-diagnostic.mjs；只验证采样器，不代替原生动画验证。
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const source = readFileSync(new URL('./src-tauri/src/main.rs', import.meta.url), 'utf8');
const script = source.match(/webview\.eval\(r#"([\s\S]*?)"#\)/)?.[1];
assert.ok(script, '必须测试实际注入的诊断脚本');
const timers = [], observers = [], reports = [], events = {};
let transform = 'none', blinking = false;
const pet = { classList: { contains: () => blinking }, dataset: { mode: 'awake' } };
const breath = {};
const image = { decode: async () => {}, naturalWidth: 600,
  getBoundingClientRect: () => ({ width: 188, height: 188 }) };
const preference = { matches: false, addEventListener: (_, fn) => { events.change = fn; } };
vm.runInNewContext(script, {
  setTimeout: (fn, delay) => timers.push({ fn, delay }),
  document: { hidden: false, querySelector: selector =>
    ({ '.dragon': image, '#pet': pet, '.breath': breath })[selector] },
  getComputedStyle: () => ({ transform, paddingTop: '6px' }),
  matchMedia: () => preference,
  window: { __TAURI_INTERNALS__: { invoke: (command, data) => reports.push({ command, data }) },
    addEventListener: (name, fn) => { events[name] = fn; } },
  MutationObserver: class {
    constructor(fn) { this.fn = fn; this.connected = true; observers.push(this); }
    observe() {}
    disconnect() { this.connected = false; }
  }
});
await timers.shift().fn();
events['yonda-show']();
events.change();
assert.equal(timers.length, 1, '重复触发不得创建并发采样');
assert.equal(timers[0].delay, 14000);
transform = 'matrix(1,0,0,1,0,2)'; observers[0].fn();
transform = 'none'; blinking = true; observers[0].fn();
blinking = false; observers[0].fn();
await timers.shift().fn();
assert.equal(observers[0].connected, false);
assert.equal(JSON.stringify(reports.at(-1).data), JSON.stringify({ breathing: true, blinks: 1, reduced: false }),
  '首尾变换相同也必须记录期间动作');
preference.matches = true;
events.change();
assert.equal(timers.length, 1, '结束后应允许下一次采样');
await timers.shift().fn();
assert.equal(JSON.stringify(reports.at(-1).data), JSON.stringify({ breathing: false, blinks: 0, reduced: true }));
assert.ok(observers.every(observer => !observer.connected), '结束后断开所有观察器');
console.log('诊断采样检查通过：重复合并、首尾回归、静态偏好、观察器清理。');
