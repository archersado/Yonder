# 设计

Application 的 `CommandPort` 接收已经过可信上层授权的 `CommandRequest`，输出 `CommandExecution`。请求只有 program、args、cwd、env、timeout；没有 shell 字符串或确认布尔值。取消通过只读 `CommandCancellation` 查询，不让 Adapter 写任务状态。

macOS Adapter 在 spawn 前完成全部静态校验与 canonicalize，使用 `CommandExt::process_group(0)` 建立独立进程组，stdin=null、stdout/stderr=piped、env_clear。两个有界读取线程只保留各 64 KiB 并用原子标记通知主循环；主循环轮询 child、取消、超时与超限。

停止顺序为进程组 TERM→250 ms→KILL→最多 1 s确认。正常父进程退出后仍发现组内后代时，同样清理并返回 unknown，不能把后台后代遗留视为成功。读取线程通过有界结果通道回收；无法及时收口时不阻塞无限等待。

非 macOS Adapter 实现稳定返回 `UnsupportedPlatform`，不尝试以单进程 kill 或 Shell 替代。
