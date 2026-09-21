# 设计

`VoiceRuntime`在可信Tauri命令开始前固定一次性目标：小龙入口为`direct`，圈选窗口入口为`region`。原生Adapter仍只上报请求权限、部分文本、最终文本和失败；运行时仅在`direct`目标调用既有`deliver_voice`，在`region`目标把有界事件投影给圈选窗口。

圈选窗口收到部分文本时只更新问题框；收到最终非空文本时立即调用现有`region_preview_submit`。该用例仍由Application PreviewSession决定是否带当前内存截图，仍使用selection来源、同一AgentSession和既有accepted/rejected/unknown反馈。最终事件消费后将目标恢复idle，不能同时产生普通voice输入。

关闭、取消、重新圈选、超时和窗口销毁调用仅对region目标生效的取消；直接语音卡保持既有行为。一个进程仍只有一个原生采集会话，不增加Worker、持久化或全局监听。
