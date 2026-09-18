# 独立 Verification Goal：真实微博浏览任务

日期：2026-09-17  
结论：PASS（macOS真实用户任务）

用户要求Yonder查看当日微博热搜。Agent经安装包内`yonder mcp`登记“查看今日微博热搜”，Yonder创建并持有`ego:49`引用；Agent只在该关联的ego-lite Task Space中打开微博热搜，随后经Yonder执行Observe、步骤推进和finish。

最终任务为`completed@14`；三个attempt均为`stopped`且`action_succeeded=1/observe_valid=1`，Browser引用为`finished=true`，14条事件与14条Outbox记录对应，ego-lite空间已关闭。证据见[`result.json`](../../../apps/desktop/evidence/browser-user-task-20260917/result.json)。

本Goal证明真实任务生命周期与引用映射，不声称Yonder复制或解释网页动作；网页操作仍由ego-lite承担。Windows Runtime按既有产品边界不可用并继续暂缓。
