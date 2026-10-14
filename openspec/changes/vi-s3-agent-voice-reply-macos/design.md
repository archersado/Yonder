# 设计

`VoiceReplySession` 位于 Application，持有 enabled、generation、reply_id、输入缓冲、下一片段序号和终态。它只把稳定的 `SpeakRequest { generation, sequence, text }` 交给 Port；正文不进入任务状态或持久化。片段转换去除有限 Markdown 表现符，代码块和 URL 生成短提示，单段与总量有界。

Desktop 组合根持有 `VoiceReplyRuntime`，把 AG-S9 已通过 Hub 校验的通知同时投影给圈选卡和语音会话。macOS Objective-C Adapter 在主线程持有单个 `AVSpeechSynthesizer`，只选择已安装中文声音；队列顺序由系统保证，Rust generation 再次防止停止后的歧义回调污染新轮次。

设置值使用 macOS 用户默认设置保存非敏感布尔值。关闭设置、关闭回复卡、连接中断及开始 VI-S1 收音都会先使 generation 失效，再停止原生播放。回调只携带 generation、sequence 和封闭事件枚举；普通日志和 UI 事件不携带朗读正文或声音名称。

UI 在现有回复卡显示“正在准备语音 / 正在朗读 / 已停止朗读 / 本地语音不可用，已保留文字回复”和停止按钮。文字回复生命周期与语音生命周期分离；播放结束不能改变 Agent turn 终态。
