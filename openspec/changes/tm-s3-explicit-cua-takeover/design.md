# Design：TM-S3 显式 CUA 接管与控制卡

`CuaControlHub` 是独立于 `TaskHost` 锁的进程内互斥值。Gateway 依据已校验的 CUA execution hint 调用 begin；同一 task_id 的后续请求刷新步骤投影但保留任务级活动状态。可信 `cua-control` 窗口只按当前 task_id 调用 request；Gateway 在同步动作返回后只在存在接管请求时消费并清理 Hub，再用最新 sequence 执行 `TaskHost::user_takeover_current`。普通动作返回不调用 finish；成功终态响应或确认的会话终止显式清理，避免把 RPC 边界误当任务边界。

UI 窗口宽 460、高 68，锚定 pet 当前显示器，在 work area 横向居中并距顶部 16 逻辑像素，与圈选工具条共用同样的 monitor/work-area 定位和视觉参数。它不是全屏窗口，不吞控制条外输入。JS 只消费宿主事件和调用无 sequence 的显式命令。

新执行不读取 macOS HID generation。历史枚举和数据兼容保留；原生 C 文件中的监测实现可删除，避免常驻 Event Tap。
