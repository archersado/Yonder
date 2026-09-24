const HOST = "com.yonder.context_spike";
let port;
let accepted = 0;

async function state(extra = {}) {
  const stored = await chrome.storage.session.get(["recording", "status", "accepted"]);
  return {
    recording: stored.recording === true,
    status: stored.status || "stopped",
    accepted: stored.accepted || 0,
    ...extra,
  };
}

async function publish(status) {
  await chrome.storage.session.set(status);
  chrome.runtime.sendMessage({ type: "state.changed", ...(await state()) }).catch(() => {});
}

function connect() {
  if (port) return port;
  port = chrome.runtime.connectNative(HOST);
  port.onMessage.addListener(async (message) => {
    if (message.accepted === true) {
      accepted += 1;
      await publish({ status: "connected", accepted });
    }
  });
  port.onDisconnect.addListener(async () => {
    port = undefined;
    await publish({ recording: false, status: "host_disconnected", accepted });
  });
  return port;
}

function send(type, metadata = {}) {
  connect().postMessage({ type, ...metadata });
}

async function setRecording(recording) {
  if (recording) {
    accepted = 0;
    await publish({ recording: true, status: "connecting", accepted });
    send("recording.started");
  } else {
    if (port) send("recording.stopped");
    await publish({ recording: false, status: "stopped", accepted });
    if (port) port.disconnect();
    port = undefined;
  }
  return state();
}

chrome.runtime.onMessage.addListener((message, _sender, respond) => {
  if (message.type === "state.get") state().then(respond);
  else if (message.type === "recording.set") setRecording(message.recording === true).then(respond);
  else return false;
  return true;
});

chrome.tabs.onActivated.addListener(async ({ tabId }) => {
  if (!(await state()).recording) return;
  const tab = await chrome.tabs.get(tabId);
  if (tab.incognito) return;
  send("tab.activated", { url_present: Boolean(tab.url), title_present: Boolean(tab.title) });
});

chrome.tabs.onUpdated.addListener(async (_tabId, change, tab) => {
  if (!(await state()).recording || tab.incognito || (!change.url && !change.title)) return;
  send("tab.updated", { url_present: Boolean(tab.url), title_present: Boolean(tab.title) });
});
