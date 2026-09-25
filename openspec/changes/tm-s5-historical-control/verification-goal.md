# TM-S5 控制历史投影 Verification Goal

性质：独立验证记录；日期：2026-09-24。Story：TM-S5；Change：`tm-s5-historical-control`；依据：Accepted AD-TM-18、TM5-AC02/07/08。

## 目标

- 已提交的 pending 请求和步骤边界 stopped 确认按原序号、同一尝试/控制身份展示；直接停止同序号不伪造 pending。
- 协议 1.21 隐藏新增字段，1.22 才返回；非归属 Agent 不能读取；损坏控制标识明确失败。
- 时间线文案不把 pending 称为已接管；分页、事件连续性和编码预算继续适用。macOS 正式 Tauri 宿主与隔离 UI 夹具分别取证。

## 结果

- SQLite/Application/Gateway 回归：pending `#5` 与 stopped `#6` 保持不同历史阶段，直接停止只投影 stopped；1.21 无 `control_event`、1.22 两条；另一 Agent 返回 `-32004`，损坏身份返回存储错误。全仓 91 项测试通过；Rust 唯一生成 TypeScript 与 JSON Schema。
- macOS 原生 WKWebView 隔离夹具显示两种文案，证据在 `apps/desktop/evidence/tm-s5-historical-control-webkit-20260924/`；此夹具不连接真实任务库。
- 独立 bundle identifier `com.yonder.control.fixture` 的正式 Tauri 宿主从专用 SQLite 读取同一条真实提交链。私有 Unix Socket：1.21 的 6 条事件均无控制字段；1.22 的 `#5/#6` 分别为 `takeover/pending` 和 `takeover/stopped`。Task Space 原生界面分别滚动显示两条文案并截图。证据在 `apps/desktop/evidence/tm-s5-historical-control-native-host-20260924/`，未向用户正式任务库写入测试任务。
- 架构检查、OpenSpec 校验、前端语法与 `git diff --check` 通过；隔离测试进程关闭后恢复普通开发构建。

## 结论

macOS 子范围 PASS。Windows 原生证据按用户决定暂缓，因此本 Change 保持 verification-pending，不 Archive；定位、录制、交回和完整 TM-S5 仍未完成。
