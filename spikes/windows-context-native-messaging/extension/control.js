const host = "com.yonder.context_spike";
const status = document.querySelector("#status");

async function setRecording(recording) {
  if ((await chrome.storage.session.get("recording")).recording === recording) {
    status.textContent = `当前：${recording ? "录制中" : "关闭"}`;
    return;
  }
  await chrome.storage.session.set({ recording });
  const response = await chrome.runtime.sendNativeMessage(host, {
    type: recording ? "recording.started" : "recording.stopped",
  });
  if (!recording) chrome.runtime.sendMessage({ type: "recording.stopped" }).catch(() => {});
  status.textContent = response?.ok ? `当前：${recording ? "录制中" : "关闭"}` : "Host 未确认";
}

document.querySelector("#start").addEventListener("click", () => setRecording(true));
document.querySelector("#stop").addEventListener("click", () => setRecording(false));
