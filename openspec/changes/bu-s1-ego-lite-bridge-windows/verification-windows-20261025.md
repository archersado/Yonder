# 独立 Verification Goal：Windows ego-lite Bridge

日期：2026-10-25
Story：BU-S1
Goal：`01a15eda-f12a-7e32-bd6c-2d3336e17351`
结论：PASS

## 验证对象

- 动态发现的 Runtime：`C:\Program Files\Citro Labs\ego lite\Application\0.6.1.11\ego-browser.exe`。
- `ego-browser skill` 使用指南版本：v2.1.0。
- Windows Adapter 通过系统目录内固定 `powershell.exe` 启动可信 Runtime，把固定 Worker 和任务数据保存在当前用户私有 UTF-8 `.mjs` 中，只向 `nodejs -e` 传递 Adapter 生成的 ASCII `file:///` import 入口。
- macOS 继续使用既有结构化直接启动，不受 Windows 启动路线影响。

## 独立验证结果

- `ego_lite_bridge_check.exe`：真实空间 `ego:5` 的 create、observe、handOff、takeOver、finish 全部通过；finish 成功清理空间。
- `ego_lite_persistence_check.exe`：真实空间 `ego:6` 的引用在 SQLite 重开后仍可恢复，finish 状态已持久化，空间已清理，观察序列为 4。
- Windows `cargo test -p yonder-adapters --locked`：35 项通过，0 项失败；覆盖缺失 Runtime 失败关闭、响应身份校验，以及中文、空格、单引号路径到 ASCII 文件 URL 的编码。
- Windows `cargo test --workspace --locked`：77 项通过，0 项失败。
- `python3 -m unittest test_check_architecture.py`（工作目录 `scripts/`）：17 项通过，0 项失败。
- `openspec validate bu-s1-ego-lite-bridge-windows --strict`：通过。
- 相关文件 `git diff --check`：通过。
- 结构化证据：`crates/adapters/evidence/bu-s1-windows-20261025/result.json`。

## 判定

2026-10-24 的 BLOCKED 记录反映当时 Runtime/bootstrap 状态，保留用于追溯；本次在更新后的 `0.6.1.11` Runtime 和用户恢复的 PowerShell 环境中完成复测，阻塞已解除。Windows Bridge 与既有 macOS 通过证据共同满足 BU-S1 双平台验证要求。本验证未执行 CUA、未回退其他浏览器引擎，也未把安装存在等同于生命周期通过。

Change 仍需正常 PR/Archive 门禁后才能归档。
