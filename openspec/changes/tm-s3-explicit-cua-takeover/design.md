# Design：TM-S3 显式 CUA 接管与控制卡

`CuaControlHub` 是独立于 `TaskHost` 锁的进程内互斥值。Gateway 依据已校验的 CUA execution hint 调用 begin；同一 task_id 的后续请求刷新步骤投影但保留任务级活动状态。可信 `cua-control` 窗口按当前 task_id 调用 request 后，立即在 blocking worker 排队获取共享 `TaskHost` 锁；若动作正在执行，该锁把停止自然串行到动作及 Observe 之后，若处于步骤间则立即执行。窗口命令在锁内唯一消费请求，以最新 sequence 调用 `TaskHost::user_takeover_current`；Gateway/Application 只在每次新派发前读取请求并停止推进，不消费或重复提交。普通动作返回不调用 finish；成功终态响应或确认的会话终止显式清理，避免把 RPC 边界误当任务边界。

UI 窗口宽 560、高 174，锚定 pet 当前显示器，在 work area 横向居中并距顶部 16 逻辑像素，与圈选工具条共用同样的 monitor/work-area 定位和视觉参数。它不是全屏窗口，不吞控制条外输入。JS 只消费宿主事件和调用无 sequence 的显式命令；本地 pending 状态跨投影刷新保留，避免按钮在停止完成前被重新启用。

新执行不读取 macOS HID generation。历史枚举和数据兼容保留；原生 C 文件中的监测实现可删除，避免常驻 Event Tap。

`CuaControlPresentation`增加慢脑与快脑的有界摘要。计划读取只产生槽位标签/数量；Application在选择前后通过`ComputerUsePort`默认空实现的投影钩子更新候选数量、单候选直接授权、多候选Jev选择或HandBack，具体Adapter和UI不参与决策。动作语义来自封闭`CuaActionKind`映射，不投出参数、置信度、模型原文或思维链。JS注册事件后立即主动读取当前Hub投影，事件与轮询只做刷新，消除隐藏WebView尚未加载时丢失首次事件的空白卡片。

步骤状态采用显式提交边界：Driver 的进程返回、工具无错误或 Observe 截图可读都不单独等于成功。Worker 必须保留 trycua `effect`；只有 `confirmed` 可进入 `Observed(true)`，`partial`、`unverifiable`、`suspected_noop` 统一进入 unknown/交回，`refused` 为已知失败。Application 仅在片段 CAS 推进成功后投影 completed；下一步开始不得反推前一步成功。控制条据此显示绿色 ✓、当前状态或黄色待核实 !，并为每项提供等价可访问文本。
