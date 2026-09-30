# Design

`cua_worker.mjs`先以`include_screenshot=false`读取已绑定PID/window的AX树。元素候选足够且动作`confirmed`时沿用轻量路径；动作效果为`refused|partial|unverifiable|suspected-noop`时，后置Observe打开同窗口截图。confirmed动作的后置AX元素为零且尚无截图时，再执行一次只读窗口Observe；不重放动作。

通用坐标`type_text`与现有受保护坐标文本统一视为窗口视觉动作，由Worker根据SDK Schema注入`delivery_mode=foreground`、受监管session和已缓存精确target。Agent提供的PID、window、session、target、snapshot和element token仍由协议拒绝或Worker覆盖。

Application把`UnknownObserved.observation`与Known Observation使用同一提取函数返回Gateway。unknown结论、任务序号和重规划边界不变。运行时仅在`action_succeeded=true`时投影成功；已Observe失败仍可推进到安全边界，但投影为未核实。

截图继续由Adapter限定为Yonder私有目录、允许Mime与4MiB；Worker会话结束或unknown清理。协议结构不新增字段，避免为Driver内部原因建立第二模型；Agent从动作结论、unknown reason与临时截图形成新计划。

协议1.40为通用桌面片段增加`focus-control/input-text/activate-control`动作种类，并在Rust唯一模型中限制工具与参数。归属慢脑经既有Gateway一次提交完整片段；Application在首步派发前形成全部槽位的宿主投影，顶部浮窗持续更新当前/完成/待执行状态。`computer.step`只保留给未协商片段能力的旧客户端。
