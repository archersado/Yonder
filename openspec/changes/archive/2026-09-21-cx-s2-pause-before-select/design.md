# 设计

`Admission`增加只读的唯一资源持有者查询；它只返回进程内可信准入的task_id，不读取请求参数。`TaskHost::pause_desktop_for_user`在宿主互斥区内读取该任务、调用现有`request_control(LocalUser, Pause)`；stopped attempt直接暂停，observed attempt紧接现有`stop_at_boundary`确认，成功后释放任务Permit。

Desktop圈选入口改为异步命令。小龙与托盘都先在阻塞线程调用同一宿主用例，再回到主线程显示选择层。无租约视为无需暂停；unknown/prepared、事务失败或租约/任务不一致返回`desktop-stop-unconfirmed`，Application圈选会话保持idle且覆盖层不出现。暂停不调用WorkFocusPort、不设置desktop_taken_over、不启动Recording。

验证以受控本地Agent创建真实CUA步骤：普通stopped边界和observed边界均由入口变为paused并打开选择层；unknown保持running/占用且不打开；无租约回归、托盘/小龙一致、任务事件/Outbox原子性及零CUA新动作一并取证。
