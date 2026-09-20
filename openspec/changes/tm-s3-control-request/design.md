# 设计

协议1.6新增task.control。Application校验身份和归属；`prepared`尚未派发副作用，可登记控制以冻结后续派发；`observed`或已停止的安全边界也可登记控制。仅`unknown`直接返回结果待核实，不能制造永远无法确认的pending control。Store把pending control、任务sequence、同状态事件和Outbox原子提交。control_id绑定当前attempt。重复同控制幂等，不同控制冲突。任务卡片接管/执行中取消调用该方法并显示正在停止；不伪报接管成功。
