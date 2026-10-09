# AD-EX-06 已验证执行经验召回

状态：Accepted  
日期：2026-10-09  
关联：EX-S5、EX-S2、AG-S5、AD-EX-01、AD-EX-05、AD-TM-23、AD-TM-24

## 决策问题

真实 CUA 回归已经形成可复用经验，例如某类媒体结果行使用坐标双击后，只有新鲜 Observe 出现播放态才算成功。目前这些经验只存在于单次会话或人工判断，后续 handback 的慢脑无法稳定利用。用户要求 replan 前先召回成功经验，再由慢脑规划。

## 决定

Yonder 增加 Application 所有的“已验证执行经验”建议投影。经验只保存稳定、非敏感的策略元数据及验证条件；它不是任务事实、计划、权限或 Driver 指令。只有动作后 Observe 成功且归属慢脑通过同一 Gateway 提交最终 `achieved` 的任务样本可以晋升。`unknown`、未核实结果、仅有 Driver 成功回执以及敏感副作用不得成为成功经验。

handback 后、AG-S5 唤醒归属慢脑前，Desktop 通过 Application Port 做一次有界精确召回，最多返回三条提示。提示经既有 Agent 输入通道交给同一归属慢脑；慢脑仍必须读取 `task.get` 与 `task.events` 的新鲜事实，并经同一 Gateway 提交新的受限计划片段。召回不直接调用 Jev、Driver 或慢脑，不建立第二通道。

经验不得保存任务正文、用户输入、联系人、消息正文、截图、transcript、坐标、PID/window ID、元素索引、临时句柄、Observation 引用、完整 Agent Payload 或模型 Payload。MVP 只用封闭字段和 SQLite 索引，不引入向量库、Embedding 或知识图谱。相同键聚合计数以限制增长。

召回是非阻塞增强：无命中、存储不可用或超过小预算时返回空提示，原 handback/replan 继续。经验 SQLite 是事后建议投影，不改变 ExecutionRuntime 的活动状态所有权，也不加入任务状态、事件和 Outbox 的原子批次。

## 后果

Application 新增经验 Port 与纯值对象；SQLite Adapter 增加独立表和索引；Desktop 组合根在锁内读取有界提示、锁外投递 Agent 输入。Driver、Jev 与 React 不读取经验库。Windows 可复用同一 Port，但本 Change 先以 macOS 链路和平台无关测试验证。
