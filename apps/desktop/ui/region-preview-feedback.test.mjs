import assert from 'node:assert/strict';
import test from 'node:test';

import { applyTextOnlyPresentation, captureFeedback, transitionToTextOnly } from './region-preview-feedback.mjs';

test('捕获错误映射为可继续提交的无图反馈', () => {
  assert.match(captureFeedback('permission-required'), /允许屏幕录制/);
  assert.match(captureFeedback('capture-unsupported-scale'), /缩放暂不支持/);
  assert.match(captureFeedback('capture-too-large'), /选区过大/);
  assert.match(captureFeedback('capture-timeout'), /截图不可用/);
  for (const error of ['permission-required', 'capture-unsupported-scale', 'capture-too-large', 'capture-timeout']) {
    assert.match(captureFeedback(error), /本次不包含截图，可继续发送文字/);
  }
});

test('无图确认先完成后端转换再清除预览并开放确认卡', async () => {
  const calls = [];
  await transitionToTextOnly(async command => calls.push(command), true);
  assert.deepEqual(calls, ['region_preview_text_only']);

  const preview = { hidden: false, removeAttribute(name) { this.removed = name; } };
  const nodes = { preview, reviewTitle: {}, status: {}, selector: { hidden: false }, review: { hidden: true } };
  applyTextOnlyPresentation(nodes, '本次不包含截图，可继续发送文字。');
  assert.equal(preview.removed, 'src');
  assert.equal(preview.hidden, true);
  assert.equal(nodes.reviewTitle.textContent, '仅提问');
  assert.equal(nodes.status.textContent, '本次不包含截图，可继续发送文字。');
  assert.equal(nodes.selector.hidden, true);
  assert.equal(nodes.review.hidden, false);
});
