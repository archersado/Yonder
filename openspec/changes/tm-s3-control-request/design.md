# 设计

协议1.6新增task.control。Application校验身份和归属，Store把pending control、任务sequence、同状态事件和Outbox原子提交。control_id绑定当前attempt。重复同控制幂等，不同控制冲突。任务卡片接管/执行中取消调用该方法并显示正在停止；不伪报接管成功。
