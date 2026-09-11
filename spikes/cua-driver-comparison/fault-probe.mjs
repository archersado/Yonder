const packages = { qwen: "@qwen-code/cua-sdk", trycua: "@trycua/cua-driver" };
const provider = process.argv[2];
if (!packages[provider]) throw new Error("用法: node fault-probe.mjs qwen|trycua");

const { CuaDriver } = await import(packages[provider]);
const driver = CuaDriver.create(undefined);
const capture = async (run) => {
  try {
    const value = await run();
    return { resolved: true, is_error: value?.isError ?? null };
  } catch (error) {
    return { resolved: false, code: error?.code ?? null, name: error?.name ?? null };
  }
};

const unknown = await capture(() => driver.callTool("yonder_unknown_tool", "{}"));
const controller = new AbortController();
controller.abort();
const cancelled = await capture(() => driver.callTool("list_apps", "{}", { signal: controller.signal }));
await driver.shutdown();
const secondShutdown = await capture(() => driver.shutdown());
const afterShutdown = await capture(() => driver.callTool("list_apps", "{}"));
driver.uniffiDestroy();

console.log(JSON.stringify({ provider, unknown, cancelled, second_shutdown: secondShutdown, after_shutdown: afterShutdown }));
