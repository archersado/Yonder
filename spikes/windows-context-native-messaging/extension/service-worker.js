const HOST = "com.yonder.context_spike";
let port;

async function isRecording() {
  return (await chrome.storage.session.get("recording")).recording === true;
}

function send(message) {
  if (!port) {
    port = chrome.runtime.connectNative(HOST);
    port.onMessage.addListener(() => chrome.action.setBadgeBackgroundColor({ color: "#16803a" }));
    port.onDisconnect.addListener(() => {
      chrome.action.setBadgeText({ text: "ERR" });
      chrome.action.setBadgeBackgroundColor({ color: "#b42318" });
      port = undefined;
    });
  }
  port.postMessage(message);
}

chrome.action.onClicked.addListener(async () => {
  const recording = !await isRecording();
  await chrome.storage.session.set({ recording });
  chrome.action.setBadgeText({ text: recording ? "REC" : "" });
  chrome.action.setBadgeBackgroundColor({ color: "#16803a" });
  send({ type: recording ? "recording.started" : "recording.stopped" });
});

chrome.tabs.onActivated.addListener(async ({ tabId }) => {
  if (!await isRecording()) return;
  const tab = await chrome.tabs.get(tabId);
  if (!tab.incognito) send({ type: "tab.activated", url: tab.url, title: tab.title });
});

chrome.tabs.onUpdated.addListener(async (_tabId, change, tab) => {
  if (await isRecording() && !tab.incognito && (change.url || change.title)) {
    send({ type: "tab.updated", url: tab.url, title: tab.title });
  }
});

chrome.runtime.onMessage.addListener((message) => {
  if (message.type === "recording.stopped" && port) {
    port.disconnect();
    port = undefined;
  }
});
