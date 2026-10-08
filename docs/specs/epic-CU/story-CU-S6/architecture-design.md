# CU-S6 架构设计

## 边界与依赖

Sky Worker 继续是 transcript 的唯一解析位置；Adapter 将有界 transcript 与句柄映射为 Application `ComputerObservation`，Gateway 再映射到 Rust Protocol。Application 不解析 AX 文本，SQLite/Runtime 不保存 transcript 或标签，React 不成为元素状态所有者。

## 状态与契约

`ComputerObservation` 新增 `observation_ref`、`transcript` 与 `elements`。transcript 限制为 64 KiB 并过滤密码/安全输入行；句柄只保留封闭角色、Sky 数字索引和短标签。Worker 从按钮、菜单项、文本框、复选框、单选框、链接、标签页、组合框、AX selectable row 及 AX image 生成句柄。可选择行的标签只拼接该行后代中有界、过滤后的文本，不跨到同级行；无文本 image 只使用通用角色标签，不推断业务含义。总数、标签长度和响应体均有界。

Worker 在任务运行态只持有最近一次对外 Observation 的引用与可操作索引集合。引用由任务与 attempt 身份派生为不透明 ID；新 Observation 替换旧引用，任务切换或 Worker 结束清除。该缓存不是任务状态、不能恢复，也不进入持久化。

## 计划与执行

协议 1.44 将 `activate-control + click + observed_element_index` 收紧为必须同时携带 `observation_ref`。协议 1.46 新增 `image` 句柄，Gateway 对 1.45 及以下客户端过滤该角色。Application 只校验形状；Worker 校验引用属于当前任务且等于最近 Observation，再执行动作前 Observe，确认索引仍存在于可操作集合，才转换为 Sky `element_index`。

若 Worker 重建导致引用丢失，返回受限 Observation 并交回慢脑；不得为恢复引用自动点击。动作后仍执行新鲜 Observe，只有预期后置事实成立才确认成功。

坐标点击调用一旦进入 Sky 动作边界，SDK 的成功、超时、窗口失效或其他错误都不能单独决定动作结果。Worker 将调用错误收敛为“结果待核实”，继续执行唯一一次动作后 Observe，并携带新鲜 Observation 以 unknown/handback 交回；界面变化只用于慢脑核实，不能把错误回执提升为 confirmed。认证、Schema 或策略错误同样不重试动作。

## Observation 绑定的截图坐标

协议 1.48 为含截图的 Observation 增加原始像素宽高；同次 `observation_ref` 同时保护 AX 句柄和视觉坐标。坐标候选继续使用 `x/y`，但必须额外携带该引用，且 `x/y` 只表示持久化截图字节的原始像素，不表示对话渲染坐标或桌面坐标。

Sky Worker 从 PNG/JPEG/WebP 字节头读取尺寸，将 `task_id + application_id + observation_ref + width + height` 作为最新临时视觉引用。坐标动作前重新 Observe 并读取当前截图尺寸；引用不匹配、坐标越界或尺寸变化时不调用 Sky。Sky 的 `app + coordinate` 继续负责将应用截图坐标解析到绑定窗口，因此外部协议不暴露窗口原点、显示器倍率或 PID。

macOS Sky 0.7.1 不公开截图 ID，Worker 不添加未受支持字段，而是在同一临时引用中保存 transcript 指纹。动作前 Observe 的 transcript 与引用代次不一致时，即便宽高相同也拒绝旧坐标；单纯视觉像素差异不参与此门禁。元素导出同时折叠装饰性 image：若 image 位于可操作祖先内，只保留祖先句柄；disabled 节点不进入集合；没有可操作祖先的 image 不做相邻代理推断，仍以自身索引执行并等待后置事实。

Gateway 只返回尺寸与引用，不持久化几何状态。调用方若查看了缩放后的图片，必须按响应的原始宽高换算；无法证明换算来源时不得生成坐标动作。动作后 Observe 与 unknown 不重试规则保持不变。

## 安全与数据生命周期

transcript 与短标签用于慢脑理解界面和控件消歧；Worker 对疑似密码/安全字段整行丢弃，并移除非法控制字符和超限文本。Gateway 响应可被归属 Agent读取，但日志、事件、Outbox、SQLite、顶部浮窗、错误字符串和持久测试证据不得记录 transcript 或标签。无正文索引和引用可进入不可变计划参数，用于同一运行会话校验。

## 失败与验证

测试覆盖 transcript 裁剪、敏感字段排除、上限、可选择行及后代标签边界、无祖先 image 句柄、装饰性 image/disabled 过滤、引用替换/跨任务/进程重建拒绝、同尺寸 transcript 变化、动作前新鲜索引校验、坐标调用错误后的 Observe、协议严格解码和 Gateway 兼容。macOS 正式企业微信样本必须证明目标企业名称已切换或安全交回；Windows 暂缓。
