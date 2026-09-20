# 独立 Verification Goal：macOS 圈选附件提交

日期：2026-09-20  
状态：等待非实现者复核

## 实现者证据

- 正式`com.yonder.desktop` bundle经真实确认卡完成accepted、rejected、unknown和不支持附件四条路径。
- accepted/rejected/unknown均按同一AgentSession发送begin、chunk、finish和`agent.input`四帧；最大帧小于64 KiB，接收端校验原始字节数与SHA-256一致，输入引用本次附件。
- rejected与unknown不自动重试；不支持附件的会话未收到文字或附件帧。三个失败结果均清除预览并要求重新圈选。
- 四条路径均未在应用数据目录发现测试正文或完整截图字节；证据不保存截图、正文、base64或摘要值。
- `cargo test --workspace`、Swift/Python/JavaScript语法检查与架构门禁通过。

结构化证据位于`apps/desktop/evidence/cx-s2-region-submit-macos-20260920/*/result.json`。

## 边界

本Goal仅覆盖macOS单显示器与本地UDS AgentSession。Windows按用户决定暂缓；多显示器与云端产品WSS不在本Change范围，CX-S2保持`verifying`且本Change不得在独立复核前Archive。
