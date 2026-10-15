import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const region = readFileSync(new URL('../../../apps/desktop/ui/region-preview.js', import.meta.url), 'utf8');

const errorHandler = region.match(/const submissionError = .*?;\n/)?.[0] ?? '';
assert.ok(errorHandler, '必须定义圈选提交失败处理');
assert.ok(!errorHandler.includes("preview.removeAttribute('src')"), '提交失败不得删除预览');
assert.ok(!errorHandler.includes("question.value=''"), '提交失败不得清空问题');
assert.ok(region.includes("submissionRetryBlocked=!retryable"), '必须记录不可重试状态');
assert.ok(region.includes("submissionError('是否送达未知；为避免重复，不可重试，请关闭或重新圈选。',false)"),
  'unknown 必须保留现场并禁止重试');
assert.ok(region.includes("message.includes('attachment-too-large')"), '必须投影附件超限错误');
assert.ok(region.includes('截图超过 4 MiB；请缩小选区后重新圈选。'), '必须给出附件超限处置');

console.log(JSON.stringify({ preserves_draft: true, unknown_retry_blocked: true, attachment_limit_visible: true }));
