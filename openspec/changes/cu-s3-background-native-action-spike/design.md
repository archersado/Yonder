# 设计

在macOS隔离fixture中记录动作前后最前台应用身份，以系统原生能力执行一个无正文动作并读取明确结果；另用现有trycua链路执行AX样本作对照。结果只含布尔、耗时和安全枚举。

Spike验证未来产品事件所需的最小事实：task/step/attempt关联、执行类别、开始、Observe、结果分类。它不修改Rust协议、SQLite、Task Space UI或桌宠，不创建通用NativeActionPort。原生不支持时必须停止，不能调用trycua、键鼠、Command或AppleScript兜底。

期限为2026-09-20。Windows按用户决定暂缓，因此macOS通过只代表子范围证据，AD-CU-06继续保持Proposed。

App Intents补充探针只读检查本机公开Swift interface和计算器metadata。若SDK没有按外部metadata标识调用的公共入口，则淘汰通用App Intent桥接；不得改用Shortcuts CLI冒充原生调用。

EventKit样本把`authorize`与`background`编译进同一临时App并顺序执行。前者只请求/Observe权限，不执行副作用；后者仅在启动时已经full access才保存临时提醒，按自身identifier判断存在，立即删除并用新store确认不存在。后台模式不得请求权限。证据只记录授权枚举、Task Space最小顺序事实与布尔结果。
