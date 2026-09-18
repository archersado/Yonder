# AD-DS-02 真实请求的瞬时桌宠展示

状态：Accepted（2026-09-16，DS-S1 `listening` 与宿主执行占用展示子范围）。

## 背景

产品简报「桌宠」定义 `listening` 标准动画；用户要求动画准确表达 Yonda 状态。当前正式桌宠只有 idle、executing、waiting_for_user、paused 四种真实来源，其余状态不得由前端定时器伪造。

## 决定

Agent Gateway 成功接受 `task.create` 后，Application 在同一调用结果中返回一个不持久化的“已接受创建请求”展示信号。桌面组合根保留一个待消费标记，从 UI 首次读取该标记时开始显示 1.6 秒 `listening`，避免 WebView 尚未就绪时提前过期；SQLite 任务状态仍是事实源，信号不写 tasks/events/outbox，不改变任务状态或序号。

展示优先级为：真实执行占用 `executing` 高于瞬时 `listening`，首次读取时如已有执行占用立即丢弃待消费标记，不延迟播放；存储/准入未知继续报错并显示 unknown。拒绝、解析失败、过期、权限失败及存储失败均不产生信号。幂等成功重投仍表示 Gateway 已接受请求，可以重放短展示，但不会创建第二任务。

桌面 UI 只消费 Rust 派生字符串；收到 `listening` 时可唤醒边缘小龙并播放完整透明姿态、确认标记、呼吸和眨眼。脉冲结束后重新查询当前 SQLite 快照，不由 UI 自行选择后续任务状态。

2026-09-16用户实机反馈并修订：同步 CUA/BUA 请求持有 `TaskHost` 锁时，UI轮询会等待动作结束，从而错过真实 `running` 窗口；桌宠状态必须改为事件驱动，不得轮询数据库。产品UDS在Rust协议成功解码出`computer.execute`/`browser.execute`后、取得宿主锁前，向pet窗口发布`executing`原生事件；请求返回后在同一锁内从SQLite/Admission派生下一展示状态并发布。其他任务变更入口提交后同样发布；pet只在素材就绪时读取一次初始快照，随后消费宿主事件。`listening`到期由宿主一次性定时事件重新派生，不启动周期查询。

## 边界

本决定不新增 wire 字段、数据库迁移、任务事件类型、常驻队列或前端状态所有者。窗口事件只是同进程展示通知，不替代SQLite任务事实和唯一Admission，也不得用于完成、重试或资源释放；解析失败不发布执行事件，事件丢失由窗口初始化时的一次快照恢复。`success`、`failed` 必须等待真实执行终态提交入口与可排序展示事件；`thinking` 等待明确外部等待事实；`recording` 归 RC-S1 手动录制。不得由展示事件推导这些状态。
