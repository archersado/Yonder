# 设计

- Adapter 以任务 ID 和事件序号关联 `task_controls.accepted_sequence/stopped_sequence`，对不同序号固定生成 pending/stopped；同序号直接 stopped。读取不使用控制记录的当前 `phase` 回填旧事件。
- Application 保持先授权再查历史，执行连续性和编码预算校验。只有协议 1.22 或可信同版本查询输出可选 `TaskEvent.control_event`；1.21 及以下缺省。
- Rust 协议定义 `ControlEvent` 并派生 TypeScript/JSON Schema。Task Space 仅用文本区分请求与已停止，不宣称已定位、录制或归还。
- 故障时保留已显示历史并明确局部错误；不补写缺失控制事实或修改 Outbox。
