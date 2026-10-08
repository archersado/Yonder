# Design：临时观察元素句柄

Sky Worker 每次动作后返回有界、过滤安全输入行的新鲜 transcript，从中选择可操作角色与 AX selectable row 生成句柄，并为该集合生成绑定任务与 attempt 的不透明 `observation_ref`。selectable row 的标签只拼接其后代文本，遇到同级或更浅节点即停止。Adapter与Gateway只做类型映射，不解析原始 AX 文本。

下一计划片段若使用元素索引，必须同时携带该引用。Worker 先确认引用是同任务最近集合，再重新Observe并确认索引仍是可操作元素；随后仅把数字索引转换为Sky内部参数。任一校验失败均不派发点击，返回临时Observation交回慢脑。

transcript与句柄标签只出现在调用响应，生命周期不跨 Worker 重建。下一计划参数只携带无正文索引与引用；SQLite、事件、Outbox、Runtime快照、顶部浮窗与日志都不保存 transcript 或标签。

坐标调用进入 Sky 后，无论返回成功或错误都进入同一个动作后 Observe；错误把动作效果保持为 unknown/handback，不跳过 Observe，也不自动重试。新鲜事实只供慢脑核实，不能把错误回执提升为成功。

含截图的 Observation 从实际图片字节提取原始像素宽高，并把同次 `observation_ref` 作为视觉坐标引用。调用方提交的 `x/y` 必须绑定该引用；Worker 校验同任务、同应用、最新引用、原图边界及动作前截图尺寸一致后，把原图像素不变传给 Sky。窗口或截图几何变化时返回新鲜 Observation 交回，不对旧坐标做比例拉伸，也不读取或推断 Retina 倍率。

macOS Sky 0.7.1 没有公开 `screenshotId`，实现不得发送未受支持字段；最新引用在 Worker 内存中额外绑定 transcript 指纹，同尺寸但语义界面已变化时拒绝旧坐标。句柄导出器折叠可操作祖先下的装饰性 image，并排除 disabled 节点；独立 image 不借相邻节点伪造代理索引，执行后必须由目标相关的新鲜事实核验。
