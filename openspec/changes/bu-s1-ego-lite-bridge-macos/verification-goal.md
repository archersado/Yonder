# Verification Goal：BU-S1 macOS ego-lite Bridge

状态：通过（macOS，2026-09-16）。

目标：在真实 macOS ego-lite Runtime 上证明同一 Task Space 完成 create → observe → handOff → takeOver → finish；并以自动化测试证明 Prepared attempt/pending control 门禁、响应身份校验和依赖失败分类。Windows 按用户决定暂缓，保持 capability unavailable。

## 结果

- `cargo test -p yonder-application -p yonder-adapters`：26项通过，覆盖引用校验和响应操作/空间身份核对。
- `ego_lite_bridge_check`：真实空间 `ego:40` 的 create、observe、handOff、takeOver、finish 全部通过，finish 已清理空间。
- 诊断阶段产生的 `Yonder Bridge 验证 *` 空间31、32、33、35、36、37、38、39均已完成清理；既有空间未改动。
- 结构化证据：`apps/desktop/evidence/ego-lite-bridge-20260916/result.json`。

## 剩余边界

本 Goal 验证 BU-S1 Bridge，不宣称 Agent Gateway 已提供 BUA 动作方法，也不宣称外部引用已进入 SQLite；两者需等待各自 Story 定案。Windows 暂缓。
