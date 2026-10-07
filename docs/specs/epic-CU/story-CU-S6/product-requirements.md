# CU-S6 产品需求

## 问题与目标

Yonder 的 Sky Worker 已能取得应用级 AX transcript，但 Gateway 只返回元素数量和截图。正式企业微信样本中，慢脑只能猜测坐标；点击后 Driver 返回 `action-unconfirmed-unparsed-transcript`，无法切换到已有的元素索引能力。产品必须提供最小、临时且可验证的控件句柄，而不是暴露完整 AX 树。

## 需求来源与分类

- 原始需求：产品简报“执行原则”要求 AX/语义元素优先、视觉仅作为兜底，每步动作后 Observe。
- 后续用户变更（2026-10-07）：真实企业微信测试失败后要求修正，并要求具备与 Codex Computer Use 类似的观察—动作—验证循环。
- 架构约束：AD-CU-09 保持 Sky 单栈、新鲜 transcript、动作后 Observe、unknown 不重试；AD-CU-10 允许临时可操作元素句柄并禁止完整 transcript 持久化；AD-TM-23 保持运行时轻量事件驱动。
- 待审建议：未来是否为复杂表格、树或无标签图标增加更多结构字段，不在本 Story 自行扩展。

## 验收条件

- OEH-01：`ComputerObservation` 可返回最多 128 个可操作控件句柄，每项只含 `index`、封闭 `role` 和至多 128 字符的可选短标签。
- OEH-02：静态正文、消息内容、密码/安全输入、完整 AX 行、坐标、PID、窗口 ID 和任意属性不得进入句柄。
- OEH-03：Observation 返回不透明 `observation_ref`；使用元素索引的候选必须同时提交该引用。
- OEH-04：Worker 只接受同任务、最近一次已返回 Observation 的引用；进程重建、任务切换、新 Observation 或引用不匹配均安全交回。
- OEH-05：动作前重新 Observe，并确认索引仍存在且仍是可操作元素；只在确认后把索引转换为 Sky 内部 `element_index`。
- OEH-06：Driver 回执不能代表成功；动作后 transcript/截图未证明变化时仍返回 unknown/handback，绝不重放点击。
- OEH-07：句柄与引用只存在于本次 Worker/Gateway 响应和计划参数，不写 SQLite、事件、Outbox、日志、顶部浮窗或任务说明。
- OEH-08：Rust Protocol 是 JSON Schema 与 TypeScript 的唯一来源，能力版本显式升级；旧客户端仍只得到原有观察字段。
- OEH-09：macOS 正式企业微信样本可从新鲜句柄选择企业切换入口并推进；不能解析时返回明确交回事实。Windows 暂缓且不得外推。

## 验收映射

| 来源 | 验收 |
|---|---|
| 产品简报 | OEH-01、OEH-05、OEH-06 |
| 2026-10-07 用户变更 | OEH-03～OEH-06、OEH-09 |
| AD-CU-09、AD-CU-10 | OEH-01～OEH-08 |
| AD-TM-23 | OEH-07 |

## 范围与非目标

本 Story 只覆盖 CUA 临时 Observation 句柄、引用绑定、协议/Gateway/Worker 合约和 macOS 正式验证。不建立持久 AX 索引，不把完整 transcript 交给 Agent，不把截图 OCR 伪装为 AX，不新增第二 Driver，不自动重试 unknown 动作，也不完成暂缓的 Windows 验证。
