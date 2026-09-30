# Design：内存意图引用与一次性发送

`CuaIntentRegistry`由Desktop组合根拥有，最多32项、单项最长15分钟。提议只接受归属Agent和活动任务，限制目标256字节、正文4096字节；生成独立`intent_ref`与`confirmation_ref`。Gateway响应只返回引用、状态和到期时间。

计划候选使用`target_ref=intent_ref`。Application按动作语义解析：输入目标查询取得接收人，填写草稿取得正文；聚焦/激活可把接收人作为Worker内存匹配提示。敏感值只存在于当前调用栈与Worker stdio帧，Worker不输出、不记录。

发送槽位调用Registry的`arm`。首次到达返回`awaiting-confirmation`并保持当前槽位；本机顶部浮窗读取预览并批准/拒绝。再次执行时Application在派发前`consume`，无论Driver结论如何引用均不可重放。确认不改变任务sequence，不伪造执行事实。

Worker根据语义从同一可信窗口的新鲜元素中选择唯一目标。`click`语义可带窗口坐标；协议1.37另允许`enter-target-query`与`draft-message-ref`的`type_text`带窗口坐标，以符合trycua在Electron/Catalyst无元素树上的单次像素聚焦+输入契约。Worker为动作注入显式窗口target并瞬时展开引用文本；Driver返回`unverifiable`时保留失败结论，沿1.34把同次窗口Observation交回慢脑核验，不自动进入下一槽位。零匹配或多匹配在无视觉候选时不执行副作用。TaskHost未就绪时Desktop在Socket创建前失败退出。
