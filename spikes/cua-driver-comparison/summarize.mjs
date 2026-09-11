import { readFile } from "node:fs/promises";

for (const provider of ["qwen", "trycua"]) {
  const evidence = JSON.parse(await readFile(`evidence/${provider}.json`, "utf8"));
  const state = JSON.parse(evidence.list_apps.structuredJson);
  const names = state.apps.map((app) => app.name).join("\n");
  const windows = JSON.parse(evidence.list_windows.structuredJson);
  const windowNames = JSON.stringify(windows);
  console.log(JSON.stringify({
    provider,
    apps: state.apps.length,
    processes: state.processes.length,
    explorer: /explorer|文件资源管理器/i.test(names),
    word: /\bword\b/i.test(names),
    excel: /\bexcel\b/i.test(names),
    powerpoint: /powerpoint/i.test(names),
    wps: /\bwps\b|kingsoft/i.test(names),
    window_count: windows.windows?.length ?? 0,
    window_fields: Object.keys(windows.windows?.[0] ?? {}).sort(),
    explorer_window: /explorer|文件资源管理器/i.test(windowNames),
    wps_window: /\bwps\b|kingsoft/i.test(windowNames),
    is_error: evidence.list_apps.isError,
    degraded: evidence.list_apps.degraded,
  }));
}
