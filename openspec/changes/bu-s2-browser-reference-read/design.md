# 设计

复用`GetParams`、`task.read`和现有`BrowserReference`响应类型。Application先按AuthContext读取任务，再从TaskStore读取引用；不存在时返回`BrowserReferenceResult`的`reference=null`，避免用NotFound混淆任务不存在与尚无BUA事实。Gateway只向协议1.15开放该查询，正式本机TaskHost使用同版本可信查询。

Task Space选择任务时与步骤、时间线并行读取。成功显示引用、所有权、托管页数、活动/已结束与更新时间序号；缺失显示“未关联”，失败显示“关联状态读取失败”。不因引用读取失败清空其他详情。

Agent控制的活动引用提供本机打开命令。UI只提交`task_id + expected_sequence`；TaskHost从SQLite重新读取任务和引用，通过现有`execute_agent_action(..., "hand-off")`路径取得Browser资源、生成执行身份、调用ego-lite `handOff()`并在Observe后提交引用和事件。Agent恢复继续使用Gateway `take-over`，两条路径共享同一Adapter。
