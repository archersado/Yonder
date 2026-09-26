# FI-S1 macOS Agent 受控文件执行

Story：FI-S1。Architecture Impact：architecture-change。关联 Accepted AD-FI-01、AD-FI-02、AD-FI-03、AD-FI-04 与 AD-TM-13。

## 动机

用户已经可以从 macOS 原生入口为任务签发短期文件授权，但归属 Agent 只能读取安全摘要，不能在同一任务和 Gateway 内执行受控操作。本 Change 在不泄露路径/授权根、不建立第二执行栈的条件下，完成小型文件读写与回收站执行闭环。

## 范围

- 从 Rust 唯一协议源派生 1.28 `file.execute` / `task.file.execute`，请求只可引用授权和提交有界内容。
- Application 在 TM-S7 启动事务后解析授权并通过现有 File Port 完成读、创建、替换或回收站。
- Gateway 与 desktop TaskHost 仅向当前认证归属 Agent 开放，保留无路径/无正文日志边界。
- 读取和写入原始正文最大 48 KiB；超限失败关闭。
- 覆盖协议、授权消费、任务启动、TOCTOU、撤权及 macOS Gateway 结构化证据。

不实现大文件分片/附件传输、目录枚举、永久删除、XML/Office 语义编辑、Shell、批量操作、Windows Runtime 或新的任务持久化模型。
