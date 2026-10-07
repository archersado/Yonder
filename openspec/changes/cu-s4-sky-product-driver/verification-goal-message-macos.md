# 独立 Verification Goal：Sky 消息语义修复

状态：验证中；Windows 按既有决定暂缓，不 Archive/Done。

## 依据与复现

沿用 CU-S4 FGD-21/23、AD-CU-09 与既有受保护消息意图契约。正式任务 `task_bb8f5f9f855944f8d711d1eaed346560` 的启动步骤成功，搜索步骤在序号 10 返回 `unknown/worker-failed`；宿主有界阶段为 `target-semantic-element`。实际 AX 使用无标签 `文本栏 (settable)`，旧匹配器无法定位；任务在序号 12 取消，发送没有派发。

## 验证范围

- 隔离 MCP fixture 覆盖真实角色别名、唯一无标签单行输入框、选择行与会话标题核对、独立消息编辑框。
- 已存在完全相同草稿不重复输入；新草稿精确替换且以后置值核实。
- 发送回执不足以证明完成；必须输入框清空且精确消息记录数量增加。发送 no-op 必须交回，不能重试。
- 发送动作派发前重新从同任务意图引用取得目标与正文；会话漂移或草稿被修改时拒绝发送，不消费任意当前输入作为已确认正文。
- 私有语义/正文引用字段不透传到 Sky 工具参数；只在既有运行时范围展开正文。
- 正式 GUI → CLI MCP → Gateway → 计划片段验证，发送前保留本机一次性确认，完成前按 AD-TM-24 提交目标核验。

## 自动化证据

`python3 apps/desktop/check-sky-message-worker.py` 通过：唯一无标签输入框可定位，相同草稿不重复，发送 no-op 拒绝。

既有 `check-sky-cua-worker.py`、Node 语法检查、OpenSpec strict 与 diff 检查通过；Application 60 与 Adapter 78 项测试全部通过。正式 macOS debug GUI/CLI 构建和打包通过。

## 正式产品阶段结果

修复提交 `03cddca` 已 ff-only 合并 dev，重新启动正式 GUI。任务 `task_2b04936bc487ce52b1cc0757c5957d80` 经官方 CLI MCP 提交完整五槽位片段 `wecom-existing-conversation` v1。当前会话已匹配目标，故直接核对现有会话而不额外搜索。

一次 `task_plan_execute` 连续完成恢复应用、核对目标会话、聚焦输入框、核对受保护草稿，在序号 22 返回 `awaiting-confirmation`。未派发发送；等待用户从顶部产品浮窗确认，不以 Agent 自造确认代替。真实送达与目标核验尚待该确认后验证，当前不宣称发送完成或 Archive。
