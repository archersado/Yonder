# Design：TM-S3 显式 CUA 接管与控制卡

`CuaControlHub` 是独立于 `TaskHost` 锁的进程内互斥值。Gateway 依据已校验的 CUA execution hint 调用 begin；可信 `cua-control` 窗口只按当前 task_id 调用 request；Gateway 在同步动作返回后调用 finish，若存在请求则用最新 sequence 执行 `TaskHost::user_takeover_current`。无论成功失败都清理活动展示，连接断开也不遗留控制卡。

UI 窗口宽 440、高 132，锚定 pet 当前显示器并在 work area 中心定位，与圈选交互共用相同 monitor/work-area helper。它不是全屏窗口，不吞卡片外输入。JS 只消费宿主事件和调用无 sequence 的显式命令。

新执行不读取 macOS HID generation。历史枚举和数据兼容保留；原生 C 文件中的监测实现可删除，避免常驻 Event Tap。
