# Verification Goal：桌宠执行状态事件驱动

状态：PASS（macOS，2026-09-16）。Windows按用户决定暂缓。

目标：同步CUA/BUA调用持有`TaskHost`锁期间，pet不轮询SQLite，仍由宿主事件立即进入`executing`；调用结束后再由SQLite与Admission派生事件切换。

证据：

- `apps/desktop/evidence/event-driven-executing-20260916/executing.png`：真实`computer.execute`尚未返回时截取原生桌面，小龙已显示无暂停标记的执行挥爪姿态；结构化摘要见同目录`result.json`。
- ego-browser事件夹具完整回归通过：`executingAnimation`、连续局部挥爪、等待/暂停、隐藏唤醒、减少动态效果及菜单交互均为true。
- `pet.js`只在素材就绪后调用一次`pet_task_state`恢复初始快照；产品代码不存在`setTimeout(checkTasks, ...)`或其他任务状态数据库轮询。后续状态只消费`yonda-presentation`窗口事件。
- `cargo test -p yonder-application -p yonder-desktop --locked`共14项通过；新增协议解码门禁测试确保只有完整合法的`computer.execute`/`browser.execute`请求触发执行开始通知。架构围栏、JavaScript语法及`git diff --check`通过。

调用结束、错误与控制提交后仍在宿主锁内调用`TaskHost::presentation()`，事件只携带白名单展示状态和任务存在布尔值，不成为任务事实源、不控制重试或资源释放。`listening`到期使用一次性宿主计时，不恢复周期轮询。
