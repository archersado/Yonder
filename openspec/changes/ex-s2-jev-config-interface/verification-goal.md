# EX-S2 最小 Jev 配置界面 Verification Goal

Story：EX-S2  
Change：ex-s2-jev-config-interface  
日期：2026-09-22  
状态：implementing

## 验证目标

验证 Application 配置模型、SQLite 单行持久化、本机 Tauri 读取/保存命令和 Task Space 配置区。保存配置不得调用 Jev、不得修改任务状态、不得写事件或 Outbox。

## 当前证据

```bash
cargo test --offline -p yonder-application -p yonder-adapters -p yonder-desktop
```

结果：通过。

- Application：17 项通过，覆盖默认配置与非法配置。
- Adapter：27 项通过，覆盖 schema 15、单行配置持久化与既有迁移回归。
- Desktop：6 项通过，覆盖宿主与窗口基础。

## 待补证据

- macOS 真实 Task Space 截图或结构化日志，展示读取、保存、无效配置拒绝和只读安全说明。
- Windows 按用户决定暂缓，不作为本子范围阻塞。
