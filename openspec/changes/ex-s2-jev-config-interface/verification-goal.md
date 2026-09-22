# EX-S2 最小 Jev 配置界面 Verification Goal

Story：EX-S2  
Change：ex-s2-jev-config-interface  
日期：2026-09-22  
状态：PASS

## 验证目标

验证 Application 配置模型、SQLite 单行持久化、本机 Tauri 读取/保存命令和独立 Jev 设置窗口。保存配置不得调用 Jev、不得修改任务状态、不得写事件或 Outbox。

## 当前证据

```bash
cargo test --offline -p yonder-application -p yonder-adapters -p yonder-desktop
```

结果：通过。

- Application：17 项通过，覆盖默认配置与非法配置。
- Adapter：27 项通过，覆盖 schema 15、单行配置持久化与既有迁移回归。
- Desktop：6 项通过，覆盖宿主与窗口基础。

## 待补证据

- macOS 独立 Jev 设置窗口截图或结构化日志，展示托盘入口、读取、保存、无效配置拒绝和只读安全说明。
- Windows 按用户决定暂缓，不作为本子范围阻塞。

## macOS 独立窗口证据

2026-09-22 通过真实 `com.yonder.desktop` 进程与系统托盘入口执行 [原生验证脚本](../../../../apps/desktop/check-jev-config-macos.swift)。

- 读取配置：[截图](../../../../apps/desktop/evidence/ex-s2-jev-config-macos-20260922/independent-221211/jev-config-read.png)
- 有效保存：[截图](../../../../apps/desktop/evidence/ex-s2-jev-config-macos-20260922/independent-221211/jev-config-saved.png)
- 无效拒绝：[截图](../../../../apps/desktop/evidence/ex-s2-jev-config-macos-20260922/independent-221211/jev-config-invalid.png)
- 结构化结果：[result.json](../../../../apps/desktop/evidence/ex-s2-jev-config-macos-20260922/independent-221211/result.json)

结果：

- 托盘可打开独立 Jev 设置窗口；Task Space 不再包含配置区。
- 读取当前配置成功。
- 有效配置保存后 `jev_config` 保持单行，且 `enabled=true`、端点不含查询串。
- 查询串端点被拒绝，已保存配置不被覆盖。
- 只读安全说明可见。
- `tasks/events/outbox` 保存前后计数均为 `189/1437/1437`，配置保存不写任务状态、事件或 Outbox。
- 未调用 Jev、未创建模型请求、未写凭据或敏感日志。

```bash
swift apps/desktop/check-jev-config-macos.swift <新的空结果目录>
```

结果：PASS。
