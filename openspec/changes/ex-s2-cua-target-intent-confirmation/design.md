# Design：内存意图引用与一次性发送

`CuaIntentRegistry`由Desktop组合根拥有，最多32项、单项最长15分钟。提议只接受归属Agent和活动任务，限制目标256字节、正文4096字节；生成独立`intent_ref`与`confirmation_ref`。Gateway响应只返回引用、状态和到期时间。

计划候选使用`target_ref=intent_ref`。Application按动作语义解析：输入目标查询取得接收人，填写草稿取得正文；聚焦/激活可把接收人作为Worker内存匹配提示。敏感值只存在于当前调用栈与Worker stdio帧，Worker不输出、不记录。

发送槽位调用Registry的`arm`。首次到达返回`awaiting-confirmation`并保持当前槽位；本机顶部浮窗读取预览并批准/拒绝。再次执行时Application在派发前`consume`，无论Driver结论如何引用均不可重放。确认不改变任务sequence，不伪造执行事实。

2026-10-07 既有交互规格缺陷修复：顶部浮窗依据本机预览的到期时间展示“发送确认已过期”，读取失败展示“发送确认暂不可用”，不再吞掉错误后继续显示等待确认。缺少预览且没有到期证据时只声明不可用，不猜测过期。批准/拒绝反馈保持到执行投影离开等待确认；无有效预览时禁用发送并清除目标/正文 DOM。仅保存任务绑定的短暂展示反馈和到期时间，不新增任务状态、协议、Registry 或自动授权；用户操作期间抑制轮询覆盖，异步旧任务预览不得污染新任务。

2026-10-18 正式企业微信样本补充单调渲染约束：每次`cua_control_presentation`返回后，控制条必须在同一渲染栈重新应用当前任务的本地确认结果；轮询使用单飞门禁，禁止多个在途请求交错恢复旧的“等待用户确认”文案。计划片段在等待确认期间过期时，执行入口不得只返回冲突并保留活动旧片段，而须在消费确认和派发发送前形成`handback`，让归属慢脑提交新片段。

2026-10-18 用户进一步明确批准后必须自动执行。Desktop批准命令因此同时触发Application可信内部续跑：从当前Runtime解析原任务归属、活动片段版本、游标与序号，以不可由请求反序列化的`BoundPlan`权限继续现有片段。该权限不能提交计划、replan或完成任务；只在本机一次性确认仍有效且活动游标未改变时允许跨越原Gateway调用deadline，随后仍执行Driver、Observe和`unknown`交回。外部Agent对过期片段的重复执行继续交回，不获得相同豁免。

Worker根据语义从同一可信窗口的新鲜元素中选择唯一目标。只有`click`语义可带窗口坐标；Worker为窗口动作注入SDK支持的精确target。只有Driver返回`confirmed`且同次Observe有效的聚焦才保留同任务同PID同窗口同语义的一次性焦点凭据；`unverifiable`沿1.34交回截图且不得继续输入引用文本。Worker在派发文本前消费凭据并瞬时展开，任何失败都不恢复。零匹配或多匹配在无视觉候选时不执行副作用。TaskHost未就绪时Desktop在Socket创建前失败退出。

协议1.38另允许`focus-target-search`使用唯一的`hotkey({keys:["cmd","f"]})`候选。该参数在Rust协议与Application各校验一次，Worker仅注入精确窗口target和会话，不做自由键盘映射。动作后强制同窗口Observe；只有明确确认才建立一次性搜索焦点，`unverifiable`直接交回。
