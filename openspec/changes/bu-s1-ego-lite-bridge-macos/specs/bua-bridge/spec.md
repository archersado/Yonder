# BUA Bridge Delta Spec

## ADDED Requirements

### Requirement: macOS 复用 ego-lite Task Space

Yonder MUST 通过 ego-lite 正式 Task Space API 执行 create/reuse、observe、handOff、takeOver 与 finish，并返回稳定外部引用、所有权和托管页数量。

#### Scenario: 生命周期连续

- **WHEN** Bridge 创建空间后依次观察、交接、接管和完成
- **THEN** 所有操作引用同一空间，所有权变化可观察，完成后不遗留 Agent 托管页

### Requirement: 失败不伪报成功

Bridge MUST 在依赖缺失、超时、进程失败或响应不可信时返回明确 unknown，MUST NOT 自动重试副作用或回退 CUA。
