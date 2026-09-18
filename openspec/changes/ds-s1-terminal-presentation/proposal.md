# 提案：DS-S1 真实终态动画接线

Story：DS-S1。来源：产品简报桌宠状态反馈、架构九态定义及 DS-S1 TERM-PRESENT-01～04。

问题：任务完成后宿主只重新派生当前快照，因此直接回到 `idle`，已有 `success/failed` 素材没有真实事件入口。

变更：统一 Gateway 写响应识别本次真实终态提交，向桌宠发布带事实标识和恢复状态的一次性展示脉冲；读取与失败响应不触发。

Architecture Impact：conforming。无协议、SQLite、依赖方向或状态所有者变化；不新增失败协议。Windows 按用户要求暂缓。
