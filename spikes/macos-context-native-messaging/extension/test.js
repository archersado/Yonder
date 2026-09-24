const resultNode = document.querySelector("#result");
const result = { status: "running", accepted: 0, incognito_rejected: false };
const port = chrome.runtime.connectNative("com.yonder.context_spike");
let finished = false;

function finish(status) {
  if (finished) return;
  finished = true;
  result.status = status;
  resultNode.textContent = JSON.stringify(result);
  document.documentElement.dataset.done = "true";
  port.disconnect();
}

port.onMessage.addListener((message) => {
  if (message.accepted === true) result.accepted += 1;
  if (result.accepted === 3) finish("passed");
});
port.onDisconnect.addListener(() => {
  if (!finished) finish("host_disconnected");
});

chrome.extension.isAllowedIncognitoAccess().then((allowed) => {
  result.incognito_rejected = allowed === false;
  port.postMessage({ type: "recording.started", incognito_rejected: result.incognito_rejected });
  port.postMessage({ type: "tab.activated", url_present: true, title_present: true });
  port.postMessage({ type: "recording.stopped" });
});
setTimeout(() => finish("timeout"), 3000);
