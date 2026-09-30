# Design：内存意图引用与一次性发送

`CuaIntentRegistry`由Desktop组合根拥有，最多32项、单项最长15分钟。提议只接受归属Agent和活动任务，限制目标256字节、正文4096字节；生成独立`intent_ref`与`confirmation_ref`。Gateway响应只返回引用、状态和到期时间。

计划候选使用`target_ref=intent_ref`。Application按动作语义解析：输入目标查询取得接收人，填写草稿取得正文；聚焦/激活可把接收人作为Worker内存匹配提示。敏感值只存在于当前调用栈与Worker stdio帧，Worker不输出、不记录。

发送槽位调用Registry的`arm`。首次到达返回`awaiting-confirmation`并保持当前槽位；本机顶部浮窗读取预览并批准/拒绝。再次执行时Application在派发前`consume`，无论Driver结论如何引用均不可重放。确认不改变任务sequence，不伪造执行事实。

Worker根据语义从同一可信窗口的新鲜元素中选择唯一目标；计划坐标候选带`x/y`时直接使用窗口坐标。零匹配或多匹配不执行副作用，沿1.34返回临时窗口Observation。
