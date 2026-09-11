# 设计

拖动补充（2026-09-11）：在现有 E0-S1 Spike 范围内使用 Tauri 自带 `data-tauri-drag-region`，仅为本地 `pet` 窗口声明 `core:window:allow-start-dragging`。不添加自定义命令、前端拖动算法或位置持久化；架构影响 conforming，无协议、状态所有者和技术路线变化。

使用官方 Tauri 2 最小模板；Rust Core 同进程运行 Local Socket echo 服务，CLI 仅发送一条带版本的 JSON 消息。先在当前 Linux 开发环境证明可编译性，再在 Windows/macOS 实机执行相同验证。Spike 代码不得提前拆成正式六 crate Workspace。

`interprocess` 统一 Listener/Stream API，但名称构造允许最小平台分支：Windows 使用 `GenericNamespaced` 映射 Named Pipe，Unix/macOS 使用 `GenericFilePath` 映射文件型 Local Socket。不得将该差异扩散到协议或 Application 层。

通过门槛：两目标平台均满足 Story 验收条件，且空闲 CPU 平均低于 1%、空闲内存目标低于 150 MB、托盘至可用低于 3 秒。若平台能力不同，只允许 Adapter 内差异，不得改变 Application 语义。
