# CM-S1 macOS Agent Command 本机批准

关联 Story：CM-S1；关联 AD：Accepted AD-CM-01、AD-CM-02、AD-TM-13。Architecture Impact：architecture-change。

## 为什么

已有 macOS Runtime 能安全执行已经获准的结构化请求，却没有可信的 Agent→本机用户批准→执行链路。直接开放 `program + args` 会使 Agent 自报确认或重放参数成为绕过路径。

## 变更

建立 Rust 协议的提议/执行方法、desktop 内存批准 Registry、本机 Task Space 批准卡，以及 Application 受控执行用例。所有 Agent 命令须先由 LocalUser 对精确摘要批准；执行只引用一次性 `command_id`，先进入 TM-S7 事务再调用既有 Runtime。

不实现 Shell、提权、安装、删除、支付、发送、批量命令、云端 WSS、Windows Runtime 或持久化批准。
