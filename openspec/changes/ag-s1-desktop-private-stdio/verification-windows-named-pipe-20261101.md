# AG-S1 Windows Named Pipe 当前用户链路验证

日期：2026-11-01。结论：**PASS（调试构建、当前 Windows 用户、Named Pipe 与细粒度 Agent 注册/撤权子范围）**。

同日正式桌面宿主的 `com.yonder.desktop.agent` Named Pipe 已完成以下真实验证：协议 1.19 握手、命名任务创建/查询/取消、Agent 身份归属、未登记/禁用/撤权 `-32003` 拒绝、禁用后的写请求拒绝、活动 `user_input` 会话撤权断开以及重启持久化。证据分别见 `../ag-s2-local-task-registration/verification-windows-20261101.md` 和 `../ag-s6-local-agent-registration/verification-windows-20261101.md`。

本报告不宣称生产本地认证完成：当前可执行文件不是正式签名安装包，尚未用两个 Windows 用户验证 Named Pipe ACL、安装目录边界和跨用户拒绝。因此父任务与完整 Story 继续保持未完成。
