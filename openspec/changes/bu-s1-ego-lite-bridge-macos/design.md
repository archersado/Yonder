# 设计

Application 定义最小 BrowserUsePort、动作、外部引用和结果；调用前读取当前 Prepared attempt，并在 pending control 时拒绝。Adapter 启动可信绝对路径的 `ego-browser nodejs`，把固定 Worker 与经 JSON 双重转义的数据字面量组成 SDK 程序，写入权限0600的短生命周期文件作为标准输入并在启动后立即unlink，不经过 Shell；响应限 64 KiB 且核对操作和空间引用。副作用超时、进程失败和响应不可信均返回 unknown，不自动重试。

Bridge 结果可复用既有 attempt result 提交；`Resource::Browser` 继续保证 MVP 单并发。外部引用持久化与 Agent wire 方法等待 TM-S1/AG 的独立设计，不在本 Change 越过门禁。
