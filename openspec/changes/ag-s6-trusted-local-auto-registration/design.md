# 设计

`local_agent_stdio::serve` 已经固定绑定 `local-test-agent`，不接受请求选择身份。它在创建 GatewaySession 前调用既有 `TaskHost::register_agent`，以当前时间写入 SQLite。该调用只发生于私有开发 stdio 入口；普通 UDS、WSS 与 GatewaySession 不新增“看到 agent_id 就登记”的分支。已禁用/撤权的身份不自动复活：自动登记仅创建缺失记录，状态不是 enabled 时保持原状并让 Gateway 返回现有权限拒绝。

验证使用正式打包 macOS 宿主的私有 stdio：首次启动自动登记并可握手/创建/提交片段；禁用后重启 stdio 仍拒绝；SQLite 重启保留状态。不得直接写库或绕过 Registry。
