# AD-E0-04 OOXML Adapter

- 状态：Accepted
- Story：E0-S4
- OpenSpec：`e0-compare-ooxml-adapters`
- 日期：2026-09-10

## 决定

首版 Document Adapter 使用 Rust 进程内实现，固定最小技术基线为 `zip 6.0.0`、`quick-xml 0.42.0` 和 `sha2 0.10.9`。Node Worker 方案淘汰，不进入产品依赖树。

## 证据

Rust 与 Node 均通过合成及 Microsoft 官方 Transitional DOCX/XLSX/PPTX 的局部文本替换、`expected_hash` 冲突拒绝、原文件不变、未修改 part 保真和 ZIP 重开。复杂样本三文件 Rust 为 51ms，Node 为 1647ms；合成样本进程峰值 RSS 分别约 3.1MiB 与 71.4MiB。Rust Release 探针约 964KiB，Node 依赖约 1.3MiB 且仍需 Node Runtime。Rust 三种复杂输出均通过 WPS 实际打开和用户目视确认。

## 架构围栏

- Document Port 不暴露 XML 或 ZIP part 路径。
- 默认另存；覆盖必须显式请求并验证 `expected_hash`。
- 输出先写同卷临时文件，结构校验后原子提交。
- 未修改 part 的解压后字节必须保持一致。
- 不处理宏、旧二进制格式、复杂图表或版式重构；这些能力降级 CUA。
- 不为当前局部操作建立完整 OOXML 对象模型。
