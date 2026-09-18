# 提案：Agent 提交已观察失败终态

Story：TM-S2。来源：TM-S2 FAIL-01～04、产品简报 failed 状态、Accepted AD-TM-11。

问题：正式 Gateway 没有失败终态入口，确定失败的动作只能让任务停留 running，桌宠 failed 素材也没有真实来源。

变更：协议 1.18 增加 `task.fail`；仅把最新已 Observe 为动作失败并推进到 stopped 的 CUA 任务原子提交为 failed，随后释放资源并发布一次性桌宠反馈。

Architecture Impact：architecture-change（协议 minor 1.18）；SQLite schema 与依赖方向不变。unknown 不得失败终结，Windows按用户要求暂缓。
