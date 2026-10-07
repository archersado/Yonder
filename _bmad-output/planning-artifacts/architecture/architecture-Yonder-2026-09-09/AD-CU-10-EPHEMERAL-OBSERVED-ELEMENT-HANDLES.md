# AD-CU-10 临时观察元素句柄

- 状态：Accepted（macOS-only，Windows 暂缓）
- Story：CU-S6
- OpenSpec：`cu-s6-ephemeral-element-handles`
- 日期：2026-10-07

## 问题

AD-CU-09 要求元素优先，并已允许 `observed_element_index`，但同时只允许 Gateway 返回元素数量和截图。正式企业微信样本因此只能猜坐标；Sky 已取得 260 个 AX 元素，却无法把可操作索引合法交给慢脑，最终形成 `action-unconfirmed-unparsed-transcript`。

## 决定

1. 完整 AX transcript 仍只存在于 Sky Worker，继续禁止持久化、日志、事件、Outbox和 UI 展示。
2. Worker 可从新鲜 transcript 派生最多 128 个临时可操作句柄；每项只含 `1..=65535` 索引、封闭角色和最多 128 字符的短标签。静态正文、安全输入、坐标、PID、窗口身份及其他 AX 属性全部丢弃。
3. 对外 Observation 同时返回不透明 `observation_ref`。引用只绑定当前任务和最近一次 Worker Observation；新 Observation、任务切换、会话结束或 Worker 重建使其失效。
4. 使用 `observed_element_index` 的计划候选必须携带同次 `observation_ref`。Worker 动作前重新 Observe，确认引用仍有效、索引仍存在且可操作后才转换为 Sky 内部 `element_index`。
5. 元素句柄不进入 SQLite、Runtime 状态、事件、Outbox、顶部浮窗或日志；Gateway 仅在协议 1.44 及以上返回。旧客户端保持既有响应。
6. 动作后 Observe 规则不变。Sky 调用成功但 transcript/视觉事实不足时仍为 unknown/handback，绝不自动重试。

## 取代关系

本决定只修订 AD-CU-09 第5项“对外只返回元素数量”的限制；AD-CU-09 的 Sky 单栈、完整 transcript 不持久化、应用绑定、每步新鲜 Observe 和 unknown 不重试继续有效。

## 验收

- Rust 协议、Schema 与 TypeScript 仅由单一来源生成，严格拒绝缺失或伪造引用的索引动作。
- Worker 测试证明只导出可操作短句柄，过滤静态正文/安全字段并执行容量限制。
- Worker 测试证明引用替换、跨任务、重建和消失索引均拒绝且不派发动作。
- Gateway 只把本次临时 Observation 返回归属 Agent，不写任务历史。
- macOS 正式企业微信样本证明慢脑可使用句柄推进企业切换入口；Windows 对等验证继续暂缓。
