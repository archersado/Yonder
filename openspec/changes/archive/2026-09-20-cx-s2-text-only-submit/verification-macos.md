# 独立 Verification Goal：macOS 无截图文字提交

日期：2026-09-20  
结论：PASS（macOS单显示器、本地UDS无截图文字提交子范围）
关联：CX-S2、`cx-s2-text-only-submit`

## 实现者证据

- 正式`com.yonder.desktop` bundle从圈选层直接点击进入350点高“仅提问”确认卡；accepted、rejected、unknown各只发送1条`agent.input`，`source=selection`、无`attachment_id`且附件帧为0。
- 三条路径使用只声明`user_input`的AgentSession，证明无截图提交不要求`user_input_attachment`。
- 临时bundle `com.yonder.desktop.cx2.textpermission`重置ScreenCapture授权后完成有效框选，明确显示权限缺失与“本次不包含截图”，随后同样以0附件帧提交并accepted；未自动请求权限。
- 四条路径均未在应用数据目录发现固定测试正文，证据不含问题正文、截图、base64、摘要值或本机路径。
- Application与Desktop定向合约、Swift/Python/JavaScript语法检查、严格OpenSpec和架构门禁通过。

结构化证据位于`apps/desktop/evidence/cx-s2-text-only-submit-macos-20260920/*/result.json`。

## 非实现者复核

非实现者基于HEAD `33335828f18e73bd140fa6d4dfa5b24638fe0e8e`独立重建并复跑：直接点击及不足最小选区均进入350点高“仅提问”卡；accepted、rejected、unknown每次恰好1条`agent.input`、0附件帧、`source=selection`且无`attachment_id`。unknown额外观察得到`first_frames=1`、`extra_frames_after_unknown=0`。

临时bundle重置ScreenCapture权限后明确降级为无截图文字并accepted，未调用权限请求API。复跑前后tasks/events/outbox/attempts保持`134/1161/1161/230`；应用数据、运行日志及证据未出现固定正文、截图、base64或哈希。Workspace 57项测试、严格OpenSpec、架构与真实PR关联门禁、Swift/Python/JavaScript语法检查全部通过。正式bundle SHA-256为`69a2cd5fd4a53c74a6616d5d5758ed0e1e25ca3563db856859a68e728e754409`，临时权限bundle为`66a20d5ea2d9b26f4b1ee3cf03ddc96265a30f4ae8767845bc4a60758653619d`。

## 边界

本Goal仅覆盖macOS单显示器、本地UDS AgentSession及权限缺失后的文字入口。语音组合、多显示器、Windows和云端产品WSS仍是后续门禁；本Change不得在独立复核前Archive。
