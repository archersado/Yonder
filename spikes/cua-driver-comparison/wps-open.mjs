import { access, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

const file = join(tmpdir(), "Yonder-E0-WPS-Test.txt");
const wpsExecutable = process.env.WPS_EXE;
if (!wpsExecutable) throw new Error("必须通过 WPS_EXE 指定已确认的 WPS 主程序路径");
await access(wpsExecutable);
await writeFile(file, "Yonder E0 WPS CUA 验证文档\n", "utf8");

const { CuaDriver } = await import("@qwen-code/cua-sdk");
const driver = CuaDriver.create(undefined);
try {
  const result = await driver.callTool("launch_app", JSON.stringify({
    launch_path: wpsExecutable,
    additional_arguments: [file],
  }));
  if (result.isError) throw new Error(result.text);
  console.log(JSON.stringify({ opened: true, file }));
} finally {
  await driver.shutdown();
  driver.uniffiDestroy();
}
