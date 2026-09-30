# Design：内存意图引用与一次性发送

`CuaIntentRegistry`由Desktop组合根拥有，最多32项、单项最长15分钟。提议只接受归属Agent和活动任务，限制目标256字节、正文4096字节；生成独立`intent_ref`与`confirmation_ref`。Gateway响应只返回引用、状态和到期时间。

计划候选使用`target_ref=intent_ref`。Application按动作语义解析：输入目标查询取得接收人，填写草稿取得正文；聚焦/激活可把接收人作为Worker内存匹配提示。敏感值只存在于当前调用栈与Worker stdio帧，Worker不输出、不记录。

发送槽位调用Registry的`arm`。首次到达返回`awaiting-confirmation`并保持当前槽位；本机顶部浮窗读取预览并批准/拒绝。再次执行时Application在派发前`consume`，无论Driver结论如何引用均不可重放。确认不改变任务sequence，不伪造执行事实。

Worker根据语义从同一可信窗口的新鲜元素中选择唯一目标。只有`click`语义可带窗口坐标；Worker为窗口动作注入SDK支持的精确target。坐标聚焦被Driver接受且同次Observe有效、但效果为`unverifiable`时，Worker保留同任务同PID同窗口同语义的一次性焦点凭据，同时沿1.34交回截图且不推进槽位。慢脑核验后以新片段提交无坐标引用文本；Worker在派发前消费凭据并瞬时展开文本，任何失败都不恢复。零匹配或多匹配在无视觉候选时不执行副作用。TaskHost未就绪时Desktop在Socket创建前失败退出。

协议1.38另允许`focus-target-search`使用唯一的`hotkey({keys:["cmd","f"]})`候选。该参数在Rust协议与Application各校验一次，Worker仅注入精确窗口target和会话，不做自由键盘映射。确认或不可核实动作都强制同窗口Observe；不可核实只建立一次性搜索焦点并交回。
