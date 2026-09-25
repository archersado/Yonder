# FI-S1 macOS 原生文件授权入口

Story：FI-S1。Architecture Impact：architecture-change。关联 Accepted AD-FI-01、AD-FI-02、AD-FI-03。

## 动机

临时授权核心已经能绑定文件事实，却没有可信产品入口，Agent 也无法取得用户已经授权的引用。本 Change 将选择、短期授权、查询和撤销接入既有 TaskHost 与 Agent Gateway，保持文件位置只留在本机受控层。

## 范围

- Task Space 内为当前任务提供读取、新建、替换和回收站授权入口；选择器路径不进入 WebView。
- 覆盖和回收站选择后使用原生二次确认；本机用户可列出和撤销授权。
- Gateway 1.27 增加 `file.grant.read` / `task.file.grants` 安全摘要，CLI/MCP 提供只读工具。
- Agent 撤权时清空其内存授权。
- 覆盖协议、Application、桌面宿主、前端和 macOS 原生证据。

不开放文件读取、写入、覆盖、回收站或 Document 的 Agent 执行方法；不修改 SQLite、Outbox、任务事件或 Windows Runtime。
