import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

const marker = `YONDER_E0_EXPLORER_${Date.now()}`;
const directory = join(tmpdir(), marker);
const filename = `${marker}.txt`;
await mkdir(directory);
await writeFile(join(directory, filename), "Yonder E0 Explorer probe\n", "utf8");

const { CuaDriver } = await import("@trycua/cua-driver");
const driver = CuaDriver.create(undefined);
const call = (name, args) => driver.callTool(name, JSON.stringify(args));
const structured = (result) => result.structuredJson
  ? JSON.parse(result.structuredJson)
  : JSON.parse(result.rawJson).structuredContent ?? {};
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

try {
  const launched = await call("launch_app", {
    launch_path: "C:\\Windows\\explorer.exe",
    additional_arguments: [directory],
  });
  if (launched.isError) throw new Error(launched.text);

  let target;
  for (let attempt = 0; attempt < 15 && !target; attempt += 1) {
    await pause(500);
    target = structured(await call("list_windows", {})).windows
      ?.find((window) => window.title?.includes(marker));
  }
  if (!target) throw new Error("未唯一定位测试资源管理器窗口");

  const observe = () => call("get_window_state", {
    pid: target.pid,
    window_id: target.window_id,
    include_screenshot: false,
    max_elements: 3000,
    query: marker,
  });
  const before = structured(await observe());
  const file = before.elements?.find((element) =>
    element.role === "ListItem" && JSON.stringify(element).includes(marker) && element.element_token);
  if (!file) {
    console.log(JSON.stringify({
      observed: true,
      ax_elements: before.elements?.length ?? 0,
      marker_visible: JSON.stringify(before).includes(marker),
      roles: [...new Set(before.elements?.map((element) => element.role).filter(Boolean) ?? [])].sort(),
    }));
    throw new Error("AX 树未唯一定位测试文件");
  }

  const clicked = await call("click", {
    pid: target.pid,
    element_token: file.element_token,
    delivery_mode: "background",
  });
  const after = structured(await observe());
  const selected = after.elements?.some((element) =>
    element.role === "ListItem" && JSON.stringify(element).includes(marker) && element.selected === true) ?? false;
  const closed = await call("hotkey", {
    pid: target.pid,
    window_id: target.window_id,
    keys: ["alt", "f4"],
    delivery_mode: "foreground",
  });
  console.log(JSON.stringify({
    observed: true,
    click_error: clicked.isError,
    click_effect: structured(clicked).effect ?? null,
    click_route: structured(clicked).route ?? null,
    selected,
    close_error: closed.isError,
  }));
} finally {
  await driver.shutdown();
  driver.uniffiDestroy();
}
