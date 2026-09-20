# 独立 Verification Goal：macOS 无截图文字提交

日期：2026-09-20  
状态：等待非实现者复核

## 实现者证据

- 正式`com.yonder.desktop` bundle从圈选层直接点击进入350点高“仅提问”确认卡；accepted、rejected、unknown各只发送1条`agent.input`，`source=selection`、无`attachment_id`且附件帧为0。
- 三条路径使用只声明`user_input`的AgentSession，证明无截图提交不要求`user_input_attachment`。
- 临时bundle `com.yonder.desktop.cx2.textpermission`重置ScreenCapture授权后完成有效框选，明确显示权限缺失与“本次不包含截图”，随后同样以0附件帧提交并accepted；未自动请求权限。
- 四条路径均未在应用数据目录发现固定测试正文，证据不含问题正文、截图、base64、摘要值或本机路径。
- Application与Desktop定向合约、Swift/Python/JavaScript语法检查、严格OpenSpec和架构门禁通过。

结构化证据位于`apps/desktop/evidence/cx-s2-text-only-submit-macos-20260920/*/result.json`。

## 边界

本Goal仅覆盖macOS单显示器、本地UDS AgentSession及权限缺失后的文字入口。语音组合、多显示器、Windows和云端产品WSS仍是后续门禁；本Change不得在独立复核前Archive。
