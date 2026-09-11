import { access, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

const packages = { qwen: "@qwen-code/cua-sdk", trycua: "@trycua/cua-driver" };
const provider = process.argv[2];
if (!packages[provider]) throw new Error("用法: node notepad-probe.mjs qwen|trycua");

const executable = process.env.NOTEPAD_EXE;
if (!executable) throw new Error("缺少 NOTEPAD_EXE");
await access(executable);
const marker = `YONDER_E0_${provider.toUpperCase()}_${Date.now()}`;
const file = join(tmpdir(), `${marker}.txt`);
await writeFile(file, "YONDER_E0_START\n", "utf8");

const { CuaDriver } = await import(packages[provider]);
const driver = CuaDriver.create(undefined);
const call = (name, args) => driver.callTool(name, JSON.stringify(args));
const structured = (result) => {
  if (result.structuredJson) return JSON.parse(result.structuredJson);
  if (result.rawJson) return JSON.parse(result.rawJson).structuredContent ?? {};
  return {};
};
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

try {
  const launch = await call("launch_app", { launch_path: executable, additional_arguments: [file] });
  if (launch.isError) throw new Error(launch.text);

  let target;
  for (let attempt = 0; attempt < 15 && !target; attempt += 1) {
    await pause(500);
    target = structured(await call("list_windows", {})).windows
      ?.find((window) => window.title?.includes(marker));
  }
  if (!target) throw new Error("未唯一定位测试记事本窗口");

  const snapshot = structured(await call("get_window_state", {
    pid: target.pid,
    window_id: target.window_id,
    include_screenshot: false,
    max_elements: 1000,
  }));
  const editable = snapshot.elements?.find((element) =>
    element.enabled !== false && /edit|document|text/i.test(element.role ?? "") && element.element_token);
  const typed = await call("type_text", editable ? {
    pid: target.pid,
    element_token: editable.element_token,
    text: marker,
    delivery_mode: "background",
  } : {
    pid: target.pid,
    window_id: target.window_id,
    x: target.bounds.width * 0.5,
    y: target.bounds.height * 0.5,
    text: marker,
    delivery_mode: "foreground",
  });
  if (!typed.isError) {
    await call("hotkey", { pid: target.pid, window_id: target.window_id, keys: ["ctrl", "s"], delivery_mode: "foreground" });
    await pause(750);
  }
  const saved = (await readFile(file, "utf8")).includes(marker);
  const closed = await call("hotkey", { pid: target.pid, window_id: target.window_id, keys: ["alt", "f4"], delivery_mode: "foreground" });
  const typedState = structured(typed);
  console.log(JSON.stringify({ provider, observed: true, input_path: editable ? "accessibility" : "pixel", typed_error: typed.isError, typed_effect: typedState.effect ?? null, typed_route: typedState.route ?? null, saved_marker: saved, close_error: closed.isError, input_error: typed.isError ? typed.text?.slice(0, 180) : null, close_error_message: closed.isError ? closed.text?.slice(0, 180) : null }));
} finally {
  await driver.shutdown();
  driver.uniffiDestroy();
}
