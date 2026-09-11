# AD-E0-01 桌面基础栈（临时结论）

- 状态：待 macOS 验证
- Story：E0-S1
- OpenSpec：`e0-validate-desktop-foundation`
- 日期：2026-09-10

## 当前决定

保留 Tauri 2 + Rust Core + `interprocess 2.x` 作为桌面基础栈候选。Local Socket 的传输代码保持统一，名称构造在 Adapter 内按平台分支：Windows `GenericNamespaced`，Unix/macOS `GenericFilePath`。

## 证据

Windows 11 上 Tauri 2.11.5 Release 可构建运行，Named Pipe 往返成功；五分钟空闲 CPU 0.109%，平均 RSS 约 69.84 MiB，窗口句柄建立 1.192 秒。Linux 编译与运行自检通过，但不属于目标平台门禁证据。

## 未决项

macOS 的 UDS、透明置顶窗口、托盘、退出、权限和五分钟资源数据尚未验证；在其完成前，本 ADR 不得转为 Accepted，E0-S1 不得完成。
