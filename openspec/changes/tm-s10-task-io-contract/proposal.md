# Proposal：TM-S10 任务输入输出与交付契约

关联 Story：TM-S10；前置架构：Accepted AD-TM-25。

## Why

任务创建没有可展示的输入/预期输出契约，完成没有交付摘要，可信 File/Document 输出也未进入产物清单，导致 Task Space 交付物长期为空。展示完整 Agent 输出既慢又违反数据最小化。

## What Changes

- 协议 1.52 增加任务 I/O 摘要和完成交付摘要，由 Rust 生成 Schema/TypeScript。
- schema 27 保存不可变创建契约和终态结果摘要。
- Runtime/Projector 与兼容路径保存结果；File/Document 成功登记 opaque 产物引用。
- Task Space 渐进详情显示摘要和真实产物，不读取完整 Agent 内容。

## Impact

Architecture Decision：Accepted AD-TM-25。修改 protocol/application/adapters/CLI/Desktop，不新增依赖，不改变六边形依赖方向。Windows 原生验证继续暂缓。
