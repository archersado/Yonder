# macOS 已知 Agent/Replay 注入拒绝 Verification Goal

日期：2026-09-22  
结论：子范围 PASS。

探针在显式租约内注入一条带Yonder标记的零位移滚动事件。结果为`controlled_session_inputs=0`、`known_injected_rejected=1`、`external_unknown=0`、`gaps=0`；没有采集事件、按键、坐标、截图、AX文本或完整事件Payload。该样本只证明带标记的已知程序注入被拒绝，不证明物理用户来源，也不授权产品持久化或Replay。

结构化证据：[result.json](../../../spikes/recording-capture/evidence/known-injection-macos-20260922/result.json)。
