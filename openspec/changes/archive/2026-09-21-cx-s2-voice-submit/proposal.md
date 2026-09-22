# Proposal：CX-S2 圈选语音直接提交

Story：CX-S2。来源为产品需求CX2-04/CX2-06/CX2-09、2026-09-18用户“语音不应确认，直接输入智能体”的变更，以及AD-VI-01/AD-CX-01于2026-09-21的组合定稿。

在macOS圈选确认卡增加麦克风入口。点击即授权本轮最终非空转写携带当前临时截图或无附件selection输入直接提交；部分转写不外发。复用现有VI原生采集、PreviewSession、`region_preview_submit`与AgentSession，不新增协议、SQLite或任务创建路径。

Architecture Impact：conforming。Windows、多显示器、首次权限拒绝、运行中撤权与设备切换仍保留既有门禁，完整CX-S2/VI-S1不转Done。
