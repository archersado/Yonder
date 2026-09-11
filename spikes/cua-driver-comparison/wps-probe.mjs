import { access, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

const packages = { qwen: "@qwen-code/cua-sdk", trycua: "@trycua/cua-driver" };
const provider = process.argv[2];
if (!packages[provider]) throw new Error("用法: node wps-probe.mjs qwen|trycua");

const marker = `YONDER_E0_${provider.toUpperCase()}_${Date.now()}`;
const file = join(tmpdir(), `${marker}.txt`);
const wpsExecutable = process.env.WPS_EXE;
if (!wpsExecutable) throw new Error("必须通过 WPS_EXE 指定已确认的 WPS 主程序路径");
await access(wpsExecutable);
await writeFile(file, "YONDER_E0_START\n", "utf8");

const { CuaDriver } = await import(packages[provider]);
const driver = CuaDriver.create(undefined);
const call = (name, args) => driver.callTool(name, JSON.stringify(args));
const structured = (result) => {
  if (result.structuredJson) return JSON.parse(result.structuredJson);
  if (result.rawJson) return JSON.parse(result.rawJson).structuredContent ?? {};
  return {};
};
const errorCode = (result) => {
  if (!result.isError) return null;
  try {
    const value = structured(result);
    return value.code ?? value.refusal?.code ?? value.status ?? "unknown";
  } catch {
    return "unstructured";
  }
};
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
let target;
let lastWindows = [];

try {
  const before = structured(await call("list_windows", {})).windows;
  const beforeIds = new Set(before.map((window) => String(window.window_id)));
  const launch = await call("launch_app", { launch_path: wpsExecutable, additional_arguments: [file] });
  if (launch.isError) throw new Error("WPS 启动失败");

  for (let attempt = 0; attempt < 15 && !target; attempt += 1) {
    await pause(1000);
    lastWindows = structured(await call("list_windows", {})).windows;
    target = lastWindows.find((window) => window.title?.includes(marker))
      ?? lastWindows.find((window) => /\bwps\b|文字|writer/i.test(window.app_name)
        && !beforeIds.has(String(window.window_id))
        && window.title?.includes("YONDER_E0_"));
  }
  if (!target) {
    const wpsWindows = lastWindows.filter((item) => /\bwps\b|文字|writer/i.test(item.app_name));
    for (const window of wpsWindows) {
      const observation = await call("get_window_state", {
        pid: window.pid,
        window_id: window.window_id,
        include_screenshot: false,
        max_elements: 3000,
        query: marker,
      });
      if (!observation.isError && observation.structuredJson?.includes(marker)) {
        target = window;
        break;
      }
    }
    if (!target && wpsWindows.length === 1) target = wpsWindows[0];
  }
  if (!target) {
    console.log(JSON.stringify({
      provider,
      launched: true,
      discovered_windows: lastWindows.length,
      discovered_wps_windows: lastWindows.filter((window) => /\bwps\b|文字|writer/i.test(window.app_name)).length,
      discovered_marker_windows: lastWindows.filter((window) => window.title?.includes(marker)).length,
      discovered_new_windows: lastWindows.filter((window) => !beforeIds.has(String(window.window_id))).length,
    }));
    throw new Error("未找到新建的 WPS 测试窗口");
  }

  const snapshot = structured(await call("get_window_state", {
    pid: target.pid,
    window_id: target.window_id,
    include_screenshot: false,
    max_elements: 3000,
  }));
  const editable = snapshot.elements?.find((element) =>
    element.enabled !== false
    && /edit|document|paragraph|text/i.test(element.role ?? "")
    && element.element_token);
  const typed = await call("type_text", editable ? {
    pid: target.pid,
    element_token: editable.element_token,
    text: marker,
    delivery_mode: "background",
  } : {
    pid: target.pid,
    window_id: target.window_id,
    x: target.bounds.width * 0.5,
    y: target.bounds.height * 0.45,
    text: marker,
    delivery_mode: "foreground",
  });
  await call("hotkey", {
    pid: target.pid,
    window_id: target.window_id,
    keys: ["ctrl", "s"],
    delivery_mode: "background",
  });
  await pause(1000);
  const saved = await readFile(file, "utf8");
  const closed = await call("hotkey", {
    pid: target.pid,
    window_id: target.window_id,
    keys: ["alt", "f4"],
    delivery_mode: "background",
  });
  if (!closed.isError) target = undefined;

  console.log(JSON.stringify({
    provider,
    launched: true,
    observed: true,
    input_path: editable ? "accessibility" : "pixel",
    typed_error: typed.isError,
    typed_code: errorCode(typed),
    typed_message: typed.isError ? typed.text?.slice(0, 240) ?? null : null,
    typed_effect: structured(typed).effect ?? null,
    saved_marker: saved.includes(marker),
    close_error: closed.isError,
    close_code: errorCode(closed),
    close_message: closed.isError ? closed.text?.slice(0, 240) ?? null : null,
  }));
} finally {
  if (target) {
    await call("hotkey", {
      pid: target.pid,
      window_id: target.window_id,
      keys: ["alt", "f4"],
      delivery_mode: "background",
    }).catch(() => {});
  }
  await driver.shutdown();
  driver.uniffiDestroy();
}
