# 独立 Verification Goal：macOS 圈选附件提交

日期：2026-09-20  
结论：PASS（macOS单显示器、本地UDS AgentSession子范围）

## 实现者证据

- 正式`com.yonder.desktop` bundle经真实确认卡完成accepted、rejected、unknown和不支持附件四条路径。
- accepted/rejected/unknown均按同一AgentSession发送begin、chunk、finish和`agent.input`四帧；最大帧小于64 KiB，接收端校验原始字节数与SHA-256一致，输入引用本次附件。
- rejected与unknown不自动重试；不支持附件的会话未收到文字或附件帧。三个失败结果均清除预览并要求重新圈选。
- 四条路径均未在应用数据目录发现测试正文或完整截图字节；证据不保存截图、正文、base64或摘要值。
- `cargo test --workspace`、Swift/Python/JavaScript语法检查与架构门禁通过。

结构化证据位于`apps/desktop/evidence/cx-s2-region-submit-macos-20260920/*/result.json`。

## 非实现者复核

非实现者基于提交`4a911ec7548f2e663d600a73742cfad28ad12aae`独立重建正式bundle并复跑四条路径：accepted、rejected、unknown均为4帧，unsupported为0帧，最大帧25027字节。复跑前后任务、事件、Outbox与Attempt计数保持`134/1161/1161/230`，应用数据目录与证据未出现测试正文、完整截图、base64、摘要值或本机路径。

严格OpenSpec校验、Workspace测试、定向协议/Application/Desktop合约、生成文件一致性、16项架构测试、架构门禁及Swift/Python/JavaScript语法检查全部通过。本轮正式bundle标识为`com.yonder.desktop`，二进制SHA-256为`e8114b902c85b72f3f433d26e95658f5f3bbd7c100b7104d853d849f9ef40d72`。

## 边界

本Goal仅覆盖macOS单显示器与本地UDS AgentSession。Windows按用户决定暂缓；多显示器与云端产品WSS不在本Change范围，CX-S2保持`verifying`且本Change不得在独立复核前Archive。
