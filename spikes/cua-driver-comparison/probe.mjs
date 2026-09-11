import { mkdir, writeFile } from "node:fs/promises";
import { performance } from "node:perf_hooks";

const providers = {
  qwen: "@qwen-code/cua-sdk",
  trycua: "@trycua/cua-driver",
};
const provider = process.argv[2];
if (!providers[provider]) throw new Error("用法: node probe.mjs qwen|trycua");

const started = performance.now();
const { CuaDriver } = await import(providers[provider]);
const driver = CuaDriver.create(undefined);
try {
  const metadata = await driver.metadata();
  const tools = JSON.parse(await driver.listToolsJson());
  const listApps = await driver.callTool("list_apps", "{}");
  const listWindows = await driver.callTool("list_windows", "{}");
  const result = {
    provider,
    package: providers[provider],
    elapsed_ms: Math.round(performance.now() - started),
    metadata,
    tool_count: Array.isArray(tools.tools) ? tools.tools.length : 0,
    tools,
    list_apps: listApps,
    list_windows: listWindows,
  };
  await mkdir("evidence", { recursive: true });
  await writeFile(`evidence/${provider}.json`, `${JSON.stringify(result, null, 2)}\n`);
  console.log(JSON.stringify({ provider, tool_count: result.tool_count, elapsed_ms: result.elapsed_ms }));
} finally {
  await driver.shutdown();
  driver.uniffiDestroy();
}
