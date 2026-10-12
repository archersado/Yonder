# CU-S6 产品需求

## 问题与目标

Yonder 的 Sky Worker 已能取得应用级 AX transcript，但 Gateway 只返回元素数量和截图。正式企业微信样本中，慢脑只能猜测坐标；点击后 Driver 返回 `action-unconfirmed-unparsed-transcript`，无法切换到已有的元素索引能力。产品必须把有界 transcript 与临时可验证控件句柄提供给归属慢脑。

## 需求来源与分类

- 原始需求：产品简报“执行原则”要求 AX/语义元素优先、视觉仅作为兜底，每步动作后 Observe。
- 后续用户变更（2026-10-07）：真实企业微信测试失败后要求修正，并要求具备与 Codex Computer Use 类似的观察—动作—验证循环。
- 架构约束：AD-CU-09 保持 Sky 单栈、新鲜 transcript、动作后 Observe、unknown 不重试；AD-CU-10 允许临时可操作元素句柄并禁止完整 transcript 持久化；AD-TM-23 保持运行时轻量事件驱动。
- 后续真实样本（2026-10-07）：企业微信左下角企业切换入口只暴露为无文本 AX image；坐标兜底因截图缩放难以稳定命中，要求继续保持元素优先并暴露受新鲜引用保护的 image 句柄。
- 后续正式对照（2026-10-08）：短暂引入的 Retina backing-scale 换算使企业微信精确原图坐标稳定失效；此前两个不同点位的结果不足以证明坐标单位差异，必须恢复 Sky 原始截图像素契约。
- 后续用户变更（2026-10-08）：Driver 错误必须透出；只能返回不含 SDK 原文、正文、截图或完整 Payload 的有界错误码，并在顶部浮窗、任务面板和 Gateway 使用同一事实。
- 后续用户变更（2026-10-09）：`driver-action-unconfirmed` 不应直接结束；在新鲜 Observe 能明确证明非敏感元素激活未产生效果时，必须有界重试，而不是交回后停住。
- 后续正式回归（2026-10-12）：QQ 音乐播放按钮保持“暂停播放”但页面其他区域变化时，Driver 曾错误确认成功；按钮动作必须使用目标相关结构事实，不能消费无关 transcript 噪声。
- 待审建议：未来是否为复杂表格、树增加更多结构字段，不在本 Story 自行扩展。

## 验收条件

- OEH-01：`ComputerObservation` 可返回最多 128 个可操作控件句柄，每项只含 `index`、封闭 `role` 和至多 128 字符的可选短标签；AX 标记为 selectable 的行必须作为 `selectable-row` 导出，标签可由该行有界后代文本合成；AX image 必须作为 `image` 导出，无文本节点只使用通用角色标签。
- OEH-02：Observation 可返回最多 64 KiB 的当前 AX transcript；密码/安全输入行必须过滤，坐标、PID、窗口 ID 和非 transcript 原生属性不得注入。
- OEH-03：Observation 返回不透明 `observation_ref`；使用元素索引的候选必须同时提交该引用。
- OEH-04：Worker 只接受同任务、最近一次已返回 Observation 的引用；进程重建、任务切换、新 Observation 或引用不匹配均安全交回。
- OEH-05：动作前重新 Observe，并确认索引仍存在且仍是可操作元素；只在确认后把索引转换为 Sky 内部 `element_index`。
- OEH-06：Driver 回执不能代表成功；动作后 transcript/截图未证明变化时仍返回 unknown/handback，绝不重放点击。
- OEH-07：transcript 与句柄标签只存在于本次 Worker/Gateway 响应，不写 SQLite、事件、Outbox、日志、顶部浮窗或任务说明；无正文的索引与引用可随不可变计划参数保存，但 Worker 重建或新 Observation 后不可复用。
- OEH-08：Rust Protocol 是 JSON Schema 与 TypeScript 的唯一来源，能力版本显式升级；协议 1.45 及以下过滤 `image`，旧客户端仍只得到其已协商的观察字段。
- OEH-09：macOS 正式企业微信样本可从新鲜句柄选择企业切换入口并推进；不能解析时返回明确交回事实。Windows 暂缓且不得外推。
- OEH-10：坐标点击已经派发后，即使 Sky 返回超时、窗口代次变化或其他动作错误，Worker 仍必须执行一次动作后 Observe；后置事实不足时返回 unknown/handback，不得丢失新鲜 Observation 或自动重放点击。
- OEH-11：含截图的 Observation 必须返回图片原始像素宽高；视觉坐标必须携带同次 `observation_ref`，并以原始截图像素表达。不得把对话界面的缩放渲染坐标、桌面全局坐标或猜测的 Retina 倍率直接交给 Driver。
- OEH-12：Worker 必须以实际图片字节校验尺寸，并在动作前确认同任务、同应用、最新引用、坐标边界及当前截图尺寸一致；窗口缩放、截图裁剪、引用过期或越界时不得派发动作，只返回新鲜 Observation 交回慢脑。
- OEH-13：同尺寸窗口内 transcript 已变化时，旧截图坐标同样失效；Worker 仅在内存保存该次 transcript 指纹，不持久化正文或指纹，也不得伪造当前 macOS Sky 未公开的 `screenshotId`。
- OEH-14：可操作节点的装饰性 image 后代不得作为第二个独立候选；disabled 节点不得作为可操作候选。无可操作祖先的 image 仍可作为受引用保护的尝试句柄，但动作回执不代表该 AX 节点确实可点击。
- OEH-15：正式样本必须通过动作后 Observation 中与目标直接相关的结构事实核验；QQ 音乐坐标成功不得外推为企业微信企业切换成功，截图任意变化也不得作为企业切换完成证据。
- OEH-16：Gateway、计划、慢脑与 Sky Adapter 均只使用同次 Observation 的原始截图像素；Adapter 不得读取、保存或应用显示器 backing scale。
- OEH-17：原图坐标未命中时必须以后置 Observation 安全交回；不得把失败解释为倍率问题后自动换算或重放，后续慢脑只能基于新鲜截图修正点位或改用元素句柄。
- OEH-18：Driver unknown 必须区分动作超时、动作拒绝、动作未确认、目标引用失效、Observe 失败与传输失败；错误码必须进入既有 Attempt 事件并同步显示在顶部 Driver 轮次、任务面板及协议 1.49 Gateway 响应。
- OEH-19：错误透出不得包含 SDK/MCP 原文、动作参数、应用名、AX transcript、截图、PID、窗口 ID 或完整 Payload；协议 1.48 及以下必须降级为既有通用 unknown reason。
- OEH-20：元素动作只允许派发一次；若首次动作后 Observe 尚未出现预期结构变化，Worker 可在不重放动作的前提下进行不超过 900ms 的有界只读 Observe 收敛。期限内仍无直接结构证据时必须返回 unknown/handback，不得以等待时间或 Driver 回执替代成功事实。
- OEH-21：若元素激活调用正常返回、900ms 收敛后 Observe 有效，且目标可按原 Observation 的角色与标签在最新树中唯一解析并明确保持原状态，Worker 必须使用新索引追加至多一次恢复动作；第二次仍无效果则交回慢脑。
- OEH-22：受限恢复不得复用旧索引，也不得用于坐标、发送、文本输入、删除、支付、安装、提权或需要确认/一次性引用的动作；超时、拒绝、断连、Observe 失败及结果无法核实仍不得自动重试。
- OEH-23：Observation 句柄按钮只能由同一角色/标签目标消失或改变来确认动作结构效果；目标仍保持原状态时，其他 transcript 变化不得提升为成功，也不得触发自动恢复。

## 验收映射

| 来源 | 验收 |
|---|---|
| 产品简报 | OEH-01、OEH-05、OEH-06 |
| 2026-10-07 用户变更与后续真实样本 | OEH-01、OEH-03～OEH-06、OEH-08～OEH-10 |
| 2026-10-08 QQ 音乐坐标偏移回归 | OEH-11、OEH-12 |
| 2026-10-08 企业微信企业切换负样本与 Retina 逻辑点诊断 | OEH-13～OEH-17 |
| 2026-10-08 Driver 错误透出用户变更 | OEH-18、OEH-19 |
| 2026-10-09 Sky 播放控件延迟状态样本 | OEH-20 |
| 2026-10-09 `driver-action-unconfirmed` 恢复变更 | OEH-21、OEH-22 |
| 2026-10-12 QQ 音乐播放按钮正式回归 | OEH-23 |
| AD-CU-09、AD-CU-10、AD-CU-11、AD-CU-12 | OEH-01～OEH-08、OEH-10～OEH-19、OEH-23 |
| AD-TM-23 | OEH-07 |

## 范围与非目标

本 Story 只覆盖 CUA 临时 Observation transcript、句柄、引用绑定、协议/Gateway/Worker 合约和 macOS 正式验证。不建立持久 AX 索引或 transcript，不把截图 OCR 伪装为 AX，不新增第二 Driver，也不完成暂缓的 Windows 验证。不可核实 unknown 仍不重试；仅对可由新鲜 Observe 证实无效果的非敏感元素激活执行一次受限恢复。
