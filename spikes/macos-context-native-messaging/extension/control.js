const statusNode = document.querySelector("#status");
const acceptedNode = document.querySelector("#accepted");
const toggle = document.querySelector("#toggle");
let recording = false;

function render(state) {
  recording = state.recording === true;
  statusNode.textContent = state.status;
  acceptedNode.textContent = String(state.accepted || 0);
  toggle.textContent = recording ? "停止验证" : "开始验证";
}

chrome.runtime.sendMessage({ type: "state.get" }, render);
chrome.runtime.onMessage.addListener((message) => {
  if (message.type === "state.changed") render(message);
});
toggle.addEventListener("click", () => {
  chrome.runtime.sendMessage({ type: "recording.set", recording: !recording }, render);
});
