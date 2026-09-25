# AD-TM-18 控制请求与停止确认的历史投影

状态：Accepted；日期：2026-09-24；Story：TM-S5。来源：产品简报「MVP 主干链路」第 9 步、「Task Space 与权限模型」，TM5-AC02/07/08；承接 AD-TM-08、AD-TM-17。

## 决策

`task.events` 协议 1.22 增加可选 `control_event`，仅按现有 `task_controls.accepted_sequence` 与 `stopped_sequence` 对应已提交事件投影 `control_id`、`attempt_id`、`kind` 与当时阶段。请求与停止分属不同序号时，前者固定为 `pending`、后者固定为 `stopped`；直接停止的同序号只投影 `stopped`。不得从当前控制阶段回写早期请求事件，也不得把普通状态变化推断为控制事实。

Adapter 只读关联既有表；Application 在已授权查询中只对协商 1.22 的会话输出字段；Rust 协议继续唯一生成 TypeScript/JSON Schema。Task Space 使用纯文本区分“停止请求已登记”和“边界停止已确认”，不把请求称为已移交。沿用连续序号和编码后字节预算，不加表、迁移、写入者或通道。

## 边界与验证

本增量不补写旧库没有的控制事实，不投影定位中/定位成功/失败、用户行为记录或交回；这些需各自的历史契约。Agent 声明的意图也不得被写成用户控制。验证覆盖 pending→stopped、直接 stopped、旧协议字段隔离、越权、分页/字节预算与 macOS 原生界面；Windows 按用户决定暂缓，完整 TM-S5 不因此 Done。
