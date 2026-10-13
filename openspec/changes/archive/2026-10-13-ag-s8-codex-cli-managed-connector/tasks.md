# 任务

- [x] 实现 `yonder codex-hook` 输入校验与当前 session 显式绑定
- [x] 以公开共享 daemon 与 WebSocket-over-proxy 替代手填 App Server Socket，并恢复同一 thread
- [x] 修正 proxy 原始字节流接线，覆盖 WebSocket 握手与真实 active-writer 会话回归
- [x] 实现 Yonder 断线有界退避重连与跨重连输入去重
- [x] 提供可审阅安装的 Codex Hook 配置和使用/诊断文档
- [x] 覆盖单元、合约与 macOS 集成测试
- [x] 声明并实现 Codex 同会话圈选附件接收、校验、一次性消费与内存图片投递
- [x] 覆盖附件成功、乱序、超限、哈希错误、跨会话、超时、断连、拒绝和不重放合约
- [x] 修正圈选提交错误映射，确保附件能力缺失或投递失败可诊断
- [x] 建立并运行独立 macOS Verification Goal
- [x] 审阅并 Archive；Windows 与 ChatGPT Desktop 保留 Deferred/Unsupported
