# 任务

- [x] 更新 AD-EX-02 为 macOS-only Accepted 子路线
- [x] 修订 EX-S2 三份设计与验收映射
- [x] 定义 Application 有界候选决策 Port 与策略
- [x] 增加 macOS Keychain 读取与官方 SDK Worker
- [x] 接入 macOS 组合根并保持 Windows 路径不编译
- [x] 增加设置页 Keychain 只读说明与远端 HTTPS 前置校验
- [x] 实现设置页 API Key 密码框及 macOS 原生 Keychain 安全写入/状态查询
- [x] 增加离线单元测试与验证 Goal
- [x] 在用户提供系统 Keychain 测试凭据后，完成不含密钥/正文的 macOS 凭据读取与远端调用证据

## 后续边界（不属于本 Change）

- 计划片段执行用例、任务状态、事件与 Outbox 接入须在 EX-S1、TM-S7 和执行层门禁满足后另建 Change。
- Windows 接线、费用证据与双平台结论须先更新 AD-EX-02，再进入后续 Change。
