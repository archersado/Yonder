# Proposal：Browser引用只读详情

Story：BU-S2，关联DS-S2 AC4与Accepted AD-BU-03。协议1.15增加`task.browser.get`，让可信Task Space读取AD-BU-01已提交的Browser引用并展示安全摘要；用户点击Agent控制的活动引用时，可信桌面命令通过既有受监督Browser链执行hand-off并进入对应ego-lite空间。

Architecture Impact：architecture-change（Rust协议新增只读方法）；不改持久化、依赖方向或执行资源。
