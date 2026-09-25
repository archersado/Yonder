# FI-S1 临时文件授权引用核心

Story：FI-S1。Architecture Impact：architecture-change。关联 Accepted AD-FI-01、AD-FI-02、TM-S7 与 DO-S2。

## 动机

File Runtime 已能安全处理精确路径，但 Agent 不能把路径、授权根或确认布尔值当成授权。先建立可信本机入口与未来 Gateway 之间的内存授权核心，防止后续 DO/FI 接线形成路径旁路。

## 范围

- Application 维护最多 256 个、最长 15 分钟的任务绑定文件授权。
- 既有文件授权保存规范位置、身份和 SHA-256，不保存正文。
- 新建目标由 File Port 预检规范父目录、授权根、目标不存在和父目录身份。
- 读取授权可复用；新建、替换和回收站授权首次解析即消费。
- 覆盖 LocalUser-only、跨任务/Agent/用途、过期、撤销、容量与竞态回归。

不新增原生选择器、Gateway/CLI/MCP、协议、SQLite、Outbox、任务事件或 Windows 实现；不访问用户真实文件。
