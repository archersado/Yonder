# document-adapter Specification

## Purpose
定义 Document Port 的语义边界与安全提交约束，使调用方通过文档、表格、幻灯片和文本语义修改 OOXML，而不暴露 XML、绕过 `expected_hash` 或原子另存校验。

## Requirements

### Requirement: 文档语义边界

Document Port SHALL 只暴露文档、工作表、幻灯片和文本语义，不暴露 OOXML/XML。

#### Scenario: Agent 修改文档内容

- **WHEN** Agent 提交局部内容修改
- **THEN** Adapter 将语义操作映射到目标 OOXML part
- **AND** 调用方不需要知道 part 路径或 XML 命名空间

### Requirement: 安全提交

Adapter SHALL 验证 `expected_hash`，默认另存并在结构校验后提交。

#### Scenario: 原文件已变化

- **GIVEN** 当前文件 hash 与 `expected_hash` 不同
- **WHEN** Adapter 收到写请求
- **THEN** 返回 `document_conflict`
- **AND** 不生成或替换输出文件

### Requirement: OOXML 保真

Adapter SHALL 原样复制未修改的 ZIP entry。

#### Scenario: 修改单一内容 part

- **WHEN** Adapter 修改 DOCX、XLSX 或 PPTX 的目标 part
- **THEN** 其他 entry 的解压后字节保持一致
- **AND** 输出包的内容类型与关系目标有效
