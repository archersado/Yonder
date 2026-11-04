# AG-S2 Windows Named Pipe 与原生桌宠面板验证

日期：2026-11-01
结论：**PASS（Windows 产品端到端子范围）**

## 验收映射

本报告复用同日 DS-S2 正式验证会话，因为它经同一 Agent Gateway 和 Application 用例覆盖了 AG-S2 的 Windows 产品链路，且比重新制造任务提供更完整的收口证据：

- 专用 Agent `ds-s2-win-7fe87213457c` 通过 Windows Named Pipe 协商协议 1.19，并经正式 `task.create` 创建 22 个命名任务；没有直接写任务数据库。
- 原生 Task Space 的“全部”第一页显示 20 项并可进入第二页；可访问树包含任务名称 `DS2W-7fe87213457c-*`、`Agent · ds-s2-win-7fe87213457c`、状态、`任务 ID`、`接管`和`取消任务`。
- 一个 `created` 任务通过原生 UI 取消，Gateway 回读为 `cancelled`、序号 2、事件 2 条，证明取消不删除任务或历史。
- 同会话真实 BUA 任务最终 `completed` 且 ego-lite Task Space 已关闭；其余测试任务通过正式取消收口，专用 Agent 已禁用。
- `native-task-space-result.json` 为 `passed=true`、`cua_executed=false`，清理记录无错误。

机器可读证据与截图位于 `apps/desktop/evidence/ds-s2-windows-20261101/`；主报告为 `../ds-s2-running-pagination/verification-windows-20261101.md`。AG-S6 的 Windows 原生 UI 登记/撤权另见 `../ag-s6-local-agent-registration/verification-windows-20261101.md`，但不是本项通过的替代证据。

## 完成边界

本报告关闭 AG-S2 的 Windows Named Pipe 与原生桌宠任务注册/展示子范围。生产签名、跨用户端点 ACL、云端 Connector 和完整 Story/PR Archive 仍按各自门禁保留。
