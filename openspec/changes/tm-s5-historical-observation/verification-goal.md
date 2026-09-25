# TM-S5 历史 Observe 事实 Verification Goal

性质：独立验证记录；日期：2026-09-24。Story：TM-S5；Change：`tm-s5-historical-observation`；依据：Accepted AD-TM-17、TM5-AC01/03。

## 验证目标

- 两步已提交 Observe 历史各自保持事件序号、步骤、结论和获准摘要；最新详情不能覆盖旧事件。
- 1.20 及以下会话不输出新增字段，1.21 才输出；越权先拒绝、畸形历史明确失败。
- Task Space 区分 matched/not-matched/unknown，摘要纯文本渲染，分页和字节预算继续有效。
- macOS 原生界面有可见证据；正式宿主真实任务端到端证据与 Windows 证据分别记录，不互相替代。

## 自动与隔离验证结果

- macOS arm64，真实 SQLite 事务在隔离内存库生成两步动作/Observe；Adapter/Agent Gateway 合约测试通过：两个历史摘要均保留，当前快照为第二步，协议 1.20 隐藏字段而 1.21 返回字段，越权 `-32004`，畸形 payload `-32603`。
- `cargo test --workspace --quiet`：91 项通过；`openspec validate --all`：68 项通过；架构检查器及其 18 项测试、`git diff --check`、前端语法检查通过。Rust 协议已生成 TypeScript 与 JSON Schema。
- ego-browser TaskSpace 120：时间线三种 Observe 文案和 HTML 字符串纯文本渲染通过；第一次试图直接访问 IIFE 内部函数失败，在同一空间改用界面夹具后通过。Ego Lite 有可用升级，未执行升级。
- macOS 原生 WKWebView 隔离夹具 PASS，结构化日志与可见截图在 `apps/desktop/evidence/tm-s5-historical-observation-visible-20260924/`。首次截图未滚到时间线，诊断截图保留在 `apps/desktop/evidence/tm-s5-historical-observation-20260924/`。此夹具不连接正式宿主或真实任务库，不能冒充正式端到端验证。
- `cargo build -p yonder-desktop -p yonder-cli` 通过，本机预览包重建并重启（PID 31993）。正式任务库只读核对无观察事件，未为验证向真实库注入测试数据。
- 以独立 Tauri bundle identifier `com.yonder.observation.fixture` 启动正式桌面宿主；仅在其专用 Application Support 目录生成任务库，两步 Observe 已提交，宿主重启恢复后任务转为 `interrupted`。通过该宿主私有 Unix Socket 的只读 Agent Gateway 验证：1.20 返回 11 条事件且无 `observation`；1.21 返回 11 条事件，其中两条历史 Observe 分别为 `step-one/matched/目标已打开` 和 `step-two/unknown/核实超时，结果未知`。证据：`/private/tmp/yonda-observation-gateway-20260924/result.json`。测试 Agent 只登记在隔离库，未写入真实任务库。
- 隔离宿主原生 Task Space 经桌宠键盘入口打开，切换“全部”并选中真实持久化的测试任务；AX 检查两个不同序号的 Observe 文案，随后分别滚动并截图。`step-one` 截图显示“已匹配 · 目标已打开”，`step-two` 截图显示“未知 · 核实超时，结果未知”。正式 Tauri 宿主 macOS 原生可见 E2E PASS；截图、结构化结果和 Gateway 结果在 `apps/desktop/evidence/tm-s5-historical-observation-native-host-20260924/`。测试脚本：`apps/desktop/check-historical-observation-host-macos.swift`。

## 当前结论与保留项

实现、隔离 WKWebView、正式 Tauri 宿主 Gateway 和宿主 Task Space macOS 原生可见 E2E 均通过。Windows 按用户决定暂缓，故本 Change 仍留在 implementing/verification-pending，不 Archive。完整 TM-S5 的产物身份/不可变清单、总配额、附件清理与删除同步仍未完成；FI-S1 Windows 证据未通过前，不开始依赖文件身份路线的产物实施。
