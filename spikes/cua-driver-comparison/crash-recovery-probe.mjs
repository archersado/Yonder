import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

if (process.argv[2] === "child") {
  const { CuaDriver } = await import("@trycua/cua-driver");
  const driver = CuaDriver.create(undefined);
  await driver.metadata();
  process.exit(23);
}

const crashed = spawnSync(process.execPath, [fileURLToPath(import.meta.url), "child"], {
  encoding: "utf8",
  timeout: 15_000,
});
if (crashed.status !== 23) throw new Error(`异常退出夹具未按预期结束: ${crashed.status}`);

const started = performance.now();
const { CuaDriver } = await import("@trycua/cua-driver");
const driver = CuaDriver.create(undefined);
try {
  const result = await driver.callTool("list_apps", "{}");
  console.log(JSON.stringify({
    child_exit: crashed.status,
    recovered: !result.isError,
    recovery_ms: Math.round(performance.now() - started),
  }));
} finally {
  await driver.shutdown();
  driver.uniffiDestroy();
}
