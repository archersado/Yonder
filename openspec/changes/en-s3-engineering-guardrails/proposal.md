# Proposal：EN-S3 阶段 5 前端单一来源与验证脚本归位

## Why

Desktop 页面可在没有自动门禁的情况下重新定义协议状态集合，`apps/desktop` 根目录同时堆积 72 个 `check-*` 验证脚本，产品源码、构建入口与验证工具边界不清。阶段 5 需要把这两类工程约束变成可持续检查，而不是依赖人工约定。

## What Changes

- 为架构检查增加前端协议单一来源门禁：使用协议状态的页面必须显式声明 `crates/protocol/generated/protocol.ts` 为来源，重复定义协议枚举集合时失败关闭。
- 为门禁补充接受、缺少来源声明、错误来源路径和重复协议集合的隔离测试。
- 将 `scripts/verification/desktop/check-*` 迁至 `scripts/verification/desktop/`，同步代码内路径、文档引用与验证命令，不修改脚本业务逻辑。
- 保持 wire、SQLite schema、Cargo 依赖、产品 UI 与原生行为不变。

关联 Story：EN-S3。Architecture Impact：none；这是既有阶段 5 设计的 conforming 工程治理，不新增 ADR。

## Non-Goals

不实施 TaskHost 并发模型；不改变前端状态文案或协议生成方式；不运行需要真实外部账户、授权或设备的原生业务样本；Windows 原生验证继续暂缓。
