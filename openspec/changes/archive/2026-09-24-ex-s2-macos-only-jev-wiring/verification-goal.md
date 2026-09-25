# EX-S2 macOS-only Jev 决策接线 Verification Goal

Story：EX-S2
Change：ex-s2-macos-only-jev-wiring
日期：2026-09-24
状态：macOS-only 决策接线 PASS；计划片段执行与 Windows 不在本 Goal 范围

## 验证目标

验证 macOS-only Jev 决策接线：Application 候选/置信策略、macOS 原生 Keychain 读取与一次性写入边界、官方 SDK Worker、组合根接线与 Windows 不注册边界。本 Goal 不验证真实任务执行、事件/Outbox 集成或 Windows。

## 结果

- `/Users/archersado/.cargo/bin/cargo test --offline -p yonder-application -p yonder-adapters -p yonder-desktop`
- `node --check crates/adapters/src/jev_worker.mjs`
- `node --check apps/desktop/ui/jev-settings.js`
- `python3 scripts/check_architecture.py`
- `python3 scripts/test_check_architecture.py`
- `openspec validate --all`

通过。

- Application：25 项通过，覆盖远端 HTTPS 配置、URL 凭据/查询串拒绝、有界候选、低置信交回、禁用配置与能力门禁。
- Adapter：38 项通过，覆盖 Keychain 缺失凭据、写入前凭据校验、Worker 调用、3000ms 上限与既有迁移/任务回归。实现使用 `security-framework` 原生 API，不再通过 `security` CLI 让密钥经过子进程 stdout、argv、环境变量或临时文件。
- Desktop：11 项通过，覆盖宿主接线与既有回归；设置页与 Worker 语法检查通过。
- 设置页仅请求 `credential_configured` 布尔状态；非空密码框经本地调用直写 Keychain 后清空，空字段保留既有凭据。
- 原生预览验证：密码框在窗口每次打开时为空；即使 `TaskHost` 未就绪，页面仍独立显示 `API Key：未配置`。证据见 `apps/desktop/evidence/jev-settings-20260924/jev-settings-empty.png` 与 `apps/desktop/evidence/jev-settings-20260924/jev-settings-status.png`。
- 本机遗留库含未过门禁的实验 schema v19；经用户确认、完整备份和空表核验后事务性恢复为正式 v17，630 个任务、2382 个事件与 2382 个 Outbox 记录保持不变，Jev 配置重新可读。审计与截图见 `apps/desktop/evidence/jev-settings-20260924/README.md`。
- 2026-09-24 使用产品 `MacosJevPort` 和已配置 Keychain 凭据执行真实远端验证。首次按 1500ms 上限返回 `TimedOut`，无凭据端点探针显示 TLS 约 1349ms、总耗时约 1795ms；据此先修订 AD-EX-02、Story 与 OpenSpec，将 macOS SDK/Worker 上限调整为 3000ms、重试仍为 0。随后发起一次新的人工验证，远端成功返回受支持候选 `handback`，端到端耗时 4823ms（含 Keychain 读取与 Worker 启动）。全程未自动重试，也未记录密钥或请求/响应正文，见 `apps/desktop/evidence/jev-remote-20260924/result.json`。
- 架构与关联检查通过：依赖方向、Story/OpenSpec 双向关联和 Story 状态门禁均无违规；`scripts/test_check_architecture.py` 18 项通过。
- OpenSpec 全量校验通过，Windows 仍未接线。

## 未完成边界与验证门禁

本 Goal 的 macOS-only 决策接线、真实 Keychain 凭据读取和远端调用门禁已经通过。Windows 接线、费用证据、计划片段执行、任务状态/事件/Outbox 集成和真实执行链路验证不在本 Goal 范围，仍保持未完成；后续片段执行必须另建 Change 与独立 Verification Goal。
