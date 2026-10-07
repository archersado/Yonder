# 桌面任务

CUA 只走 Yonder 的组合步骤或计划片段入口。不要直接调用 trycua、Sky、系统自动化脚本、私有 Worker 或底层 `computer_execute`。

## 先计划再执行

1. 读取最新任务快照和事件，识别当前应用、目标状态、用户约束及已经确认的副作用。
2. 若发现 `task_plan_submit` 与 `task_plan_execute`，为当前有界子目标一次生成完整多步片段：启动或恢复应用、定位工作对象、准备内容、等待确认、执行副作用、验证结果分别作为必要槽位。不要把整个片段退化为一个动作。
3. 每个槽位可以有多个完成同一语义步骤的候选。例如元素定位为首选，元素缺失或不唯一时才提供视觉候选；不要把“点击、输入、发送”塞进同一槽位作为互斥候选。
4. 计划提交成功后，使用返回的新序号调用 `task_plan_execute`。Yonder/Jev 在片段内选择、执行并逐步 Observe；慢脑不要在每步之间抢回规划权。
5. Runtime 不支持计划片段时才使用 `computer_step`，并明确这是兼容路径。协议1.40支持通用桌面任务时，必须一次提交完整有界片段，并使用`focus-control`、`input-text`、`activate-control`等Schema实际公开的动作种类；不得把片段拆成多个单步。每次调用仍由 Yonder完成步骤声明、动作、Observe和推进。

## Observe 与重规划

- 动作成功回执不是结果证明；以后置 Observe 和任务事件为准。
- 协议1.44返回`transcript`、`elements`与`observation_ref`时，先根据 transcript 选择唯一可操作元素；通用`activate-control + click`必须同时提交该元素的`observed_element_index`和同次`observation_ref`。引用过期、元素消失或Worker重建时重新Observe，不复用旧索引。
- 元素缺失时先使用同片段中的视觉兜底。只有候选耗尽、事实改变或预期 Observe 不满足才交回慢脑。
- 交回后先 `task_get` 与 `task_events`，区分 `observed`、`unknown`、等待确认和用户接管，再提交递增版本的新片段。
- `unknown`、断连或超时意味着副作用结果待核实。不得重放发送、删除、支付等动作；先重新 Observe 或等待用户核实。
- 用户普通鼠标或键盘输入本身不等于接管。只有 Yonder 显式接管事实才停止；停止后不要继续派发动作。

## 发送类任务

协议 1.39 及以上按以下顺序生成，不把收件人与正文写进计划：

1. 任务创建后调用 `task_cua_intent_propose(task_id, target, message)`。只保存返回的 `intent_ref` 与 `confirmation_ref`；不要在后续参数、日志或说明中重复明文。
2. 一个计划片段至少分别表达 `focus-target-search`（元素/坐标`click`，或企业微信封闭`hotkey`参数`keys=["cmd","f"]`）、`enter-target-query`（`type_text`）、`activate-target`（`click`）、`focus-message-composer`（`click`）、`draft-message-ref`（`type_text`）和 `send-message`（`click` 或 `press_key`）。这些候选的 `target_ref` 都使用同一 `intent_ref`，只有最终发送候选携带 `confirmation_ref`；不得生成其他快捷键组合。
3. 元素候选的 `arguments` 使用空对象；只有 Yonder 返回临时视觉 Observation、慢脑据此确认同一窗口坐标后，才可给 `click`、`enter-target-query`或`draft-message-ref`提交仅含 `x`、`y` 的新候选。后两者由Yonder原子注入精确窗口foreground输入，正文仍只在派发瞬间从引用展开；Agent不得提交`delivery_mode`或`text`。只有Driver明确返回`confirmed`且同次Observe有效才算成功；`unverifiable`必须交回且不得自动重试。
4. `task_plan_execute` 返回 `awaiting-confirmation` 时，顶部浮窗会展示本机发送预览。此时不完成任务、不另造确认，也不重建计划。用户批准后以返回的未变化任务序号再次调用同一计划版本；拒绝、过期或引用缺失则停止并报告。

确认只绑定最终发送，不阻止前面的启动、搜索和草稿准备。最终以 `delivery-confirmed` Observe 或明确的待核实状态为准；发送已派发后的 `unknown` 绝不重试。
