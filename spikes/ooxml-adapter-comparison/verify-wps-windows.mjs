const { CuaDriver } = await import("@trycua/cua-driver");
const driver = CuaDriver.create(undefined);
try {
  const result = await driver.callTool("list_windows", "{}");
  const windows = JSON.parse(result.structuredJson).windows;
  const titles = windows.map((window) => window.title ?? "");
  console.log(JSON.stringify({
    docx_window: titles.some((title) => title.includes("Yonder-E0-Rust-Docx")),
    xlsx_window: titles.some((title) => title.includes("Yonder-E0-Rust-Xlsx")),
    pptx_window: titles.some((title) => title.includes("Yonder-E0-Rust-Pptx")),
    repair_or_damage_dialog: titles.some((title) => /repair|corrupt|damage|修复|损坏|错误/i.test(title)),
  }));
} finally {
  await driver.shutdown();
  driver.uniffiDestroy();
}
