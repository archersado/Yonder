# DO-S2 macOS OOXML 文件 Runtime

Story：DO-S2。Architecture Impact：conforming。关联 Accepted AD-DO-01、AD-FI-01、DO-S1、FI-S1。

## 动机

DO-S2 已能在内存中读取和转换 DOCX/XLSX/PPTX，FI-S1 已提供 macOS 规范文件身份、锁和原子提交。本增量组合两个 Port，关闭默认另存与可信本机覆盖的真实文件边界。

## 范围

- 从 File Port 有界读取源文件，再由 Document Port 返回语义快照。
- 默认另存绑定源文件身份与 `expected_hash`，源变化、目标存在或锁冲突不产生输出。
- 暂存输出由 Document Port 重新 inspect 并确认格式一致后原子提交。
- 覆盖原文件仅允许可信 LocalUser，用读取时身份与哈希防陈旧。
- DOCX/XLSX/PPTX 使用隔离临时目录统一验证。

不新增 Gateway/CLI/MCP、协议、SQLite、任务事件、覆盖确认 UI 或 Windows 路线；不修改用户真实文档。
