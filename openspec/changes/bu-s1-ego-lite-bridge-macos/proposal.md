# Proposal：macOS ego-lite BUA Bridge

Story：BU-S1。

macOS 已满足 AD-E0-03 重启条件。直接复用已安装 ego-lite Task Space，提供 create/reuse、observe、handOff、takeOver、finish Bridge；不复制 Browser Task Space，不回退 CUA，不改变 Agent 协议或 SQLite。
