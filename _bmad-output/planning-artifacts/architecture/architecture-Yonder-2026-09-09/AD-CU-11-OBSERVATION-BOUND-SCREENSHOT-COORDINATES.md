# AD-CU-11 Observation 绑定的截图坐标

- 状态：Accepted（macOS-only，Windows 暂缓）
- Story：CU-S6
- OpenSpec：`cu-s6-ephemeral-element-handles`
- 日期：2026-10-08

## 问题

协议 1.41 允许视觉候选提交裸 `x/y`，但 Observation 只返回截图路径和 MIME。慢脑无法知道截图原始像素尺寸，也无法证明坐标来自哪次截图；图片在对话界面中缩放后，渲染坐标会被直接当作 Sky 坐标。Retina 缩放、窗口裁剪或窗口几何变化会导致点击偏移，正式 QQ 音乐样本已复现“截图目标正确但连续点击未命中”。

## 决定

1. Gateway 与慢脑中的视觉坐标使用 Sky 当前应用截图的原始像素坐标，不使用桌面全局坐标、UI 渲染坐标或调用方猜测的 Retina 倍率。Sky 动作继续携带已绑定应用，由 Driver 负责窗口归属；外部 Agent 不提交窗口原点、PID、窗口 ID 或缩放倍率。
2. 含截图的 Observation 必须同时返回原始 `screenshot_width_px` 与 `screenshot_height_px`，并复用同次 `observation_ref` 作为截图坐标引用。尺寸来自实际持久化图片字节，不从显示器、窗口 bounds 或渲染尺寸推断。
3. 所有坐标动作必须携带产生该截图的最新 `observation_ref`。Worker 只接受同任务、同应用、最近一次 Observation 的引用，并要求 `x/y` 位于该截图原始像素边界内。
4. 动作前 Worker 重新 Observe；当前截图尺寸与引用记录的尺寸不一致时，视为窗口缩放、裁剪或代次变化，不派发点击，返回新鲜 Observation 交回慢脑。不得把旧坐标按新窗口拉伸后继续使用。
5. 截图尺寸、引用与应用绑定只存在于当前 Worker/Gateway 调用链，不进入 SQLite、事件、Outbox、日志或顶部浮窗。截图内容与完整 Driver Payload 的既有禁记规则不变。
6. 图片展示层可任意缩放，但必须使用响应中的原始宽高把渲染点换算为原始像素点；若调用方不知道渲染图片的完整尺寸，则不得提交坐标候选。
7. 动作回执仍不代表成功。点击只派发一次，随后强制 Observe；引用过期、尺寸变化、越界或后置事实不足均交回，不自动重试。
8. macOS Sky 0.7.4 的公开 `get_app_state/click` 契约不返回也不接受 `screenshotId`；Yonder 不得伪造该字段。Worker 除原图尺寸外，还把该次 Observation 的有界 transcript 指纹保存在内存。动作前重读的 transcript 已变化时，即使截图尺寸相同也不得派发旧坐标。视觉像素变化但 transcript 未变化不单独使引用失效，避免光标、动画等非语义变化永久阻塞坐标兜底。
9. QQ 音乐的一次坐标成功只证明该窗口样本的截图坐标可用，不能外推到企业微信自绘控件、其他窗口几何或显示器配置。每个正式产品样本都必须由动作后 Observe 明确证明目标 UI 状态，不得用“坐标请求已返回”或“任意截图变化”代替。
10. 2026-10-08 企业微信正式对照任务 `task_df83da0c773b542c78e2cf8744c5bf2f` 证明此前“Sky 动作使用逻辑点”的结论无效：把精确原图坐标 `(58,1929)` 除以 2 后，动作后 Observation 与原界面完全相同；既有 QQ 音乐样本则已证明原图坐标可直接命中。此前 560×872 菜单中 `(115,52)` 与 `(180,100)` 是两个不同命中点，不能据此推导 2x 坐标单位差异。
11. macOS Adapter 不读取、保存或应用显示器 backing scale；通过全部引用、尺寸、边界和 transcript 新鲜校验后，把同次 Observation 的原始截图像素原样传给 Sky。Agent、UI、协议与 Adapter 均不得猜测 Retina 倍率。
12. 多显示器、窗口移动或裁剪仍以动作前截图尺寸与 transcript 指纹变化使旧引用失效；不得对旧坐标做比例拉伸。若精确原图坐标仍无法命中，只能以后置 Observation 交回并修正视觉定位或元素候选，不能改变坐标单位契约。

## 取代关系

本决定收紧 AD-CU-09/协议 1.41 的裸坐标规则，并补充 AD-CU-10 的 `observation_ref` 适用范围：引用不仅保护 AX 元素索引，也保护截图坐标。第 10～12 条撤销短暂引入的 Adapter backing-scale 换算，恢复并强化“原始截图像素直接传给 Sky”的契约。AX/元素优先、Sky 单栈、运行时内存态和 unknown 不重试继续有效。

## 验收

- Rust 协议、Schema 与 TypeScript 从单一来源生成，严格拒绝未绑定引用、半套截图尺寸及越界坐标。
- Worker 从 PNG/JPEG/WebP 原始字节读取尺寸；不信任调用方尺寸，也不读取或猜测 Retina 倍率。
- Adapter 测试覆盖原始像素不变传递、同尺寸 transcript 变化、窗口尺寸变化、过期/跨任务引用、越界坐标和动作后 Observe。
- macOS 正式企业微信样本用精确原始截图像素命中企业切换入口，并由后置 Observe 核验目标企业名称；Windows 对等验证继续暂缓。
