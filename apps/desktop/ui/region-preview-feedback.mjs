export function captureFeedback(message) {
  if (message.includes('permission-required')) return '需要允许屏幕录制；本次不包含截图，可继续发送文字。';
  if (message.includes('capture-unsupported-scale')) return '当前显示器缩放暂不支持截图；本次不包含截图，可继续发送文字。';
  if (message.includes('capture-too-large')) return '选区过大；本次不包含截图，可继续发送文字。';
  return '截图不可用；本次不包含截图，可继续发送文字。';
}

export async function transitionToTextOnly(invoke, transition) {
  if (transition) await invoke('region_preview_text_only');
}

export function applyTextOnlyPresentation(nodes, message) {
  nodes.preview.removeAttribute('src');
  nodes.preview.hidden = true;
  nodes.reviewTitle.textContent = '仅提问';
  nodes.status.textContent = message;
  nodes.selector.hidden = true;
  nodes.review.hidden = false;
}
