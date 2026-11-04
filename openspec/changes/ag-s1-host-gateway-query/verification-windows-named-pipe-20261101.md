# AG-S1 Host Gateway Windows Named Pipe 验证

日期：2026-11-01。结论：**PASS（当前 Windows 用户的 Host Gateway 查询/写入与身份撤权子范围）**。

Windows 原生桌面宿主已通过 Named Pipe 复用唯一 `TaskHost` 和同一 Gateway：登记 Agent 可握手、创建命名任务、查询详情/事件并取消；未登记、禁用或撤权 Agent 返回稳定 `-32003`；任务状态、事件和归属可由原生 Task Space 回读。详细机器证据见 `../ag-s2-local-task-registration/verification-windows-20261101.md`、`../ag-s6-local-agent-registration/verification-windows-20261101.md` 和 `apps/desktop/evidence/ds-s2-windows-20261101/native-task-space-result.json`。

未完成边界仍是正式签名安装包和跨 Windows 用户 Named Pipe ACL/生产认证验证；当前调试构建不能替代这些门禁。
