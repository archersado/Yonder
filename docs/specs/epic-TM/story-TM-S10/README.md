# TM-S10 任务输入输出与交付契约

Story: TM-S10  
Epic: TM  
Status: verifying
OpenSpec: tm-s10-task-io-contract

变更目录：`openspec/changes/tm-s10-task-io-contract/`

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态

2026-10-16 用户明确要求解决“交付物为空”，并检查任务分配时的输入输出契约。审阅确认当前 `task.create` 无可展示 I/O 契约、`task.complete` 无交付摘要，能力执行回执也未登记为产物。Accepted AD-TM-25 授权协议 1.52、schema 27、Runtime 投影和 File/Document 可信产物登记；Windows 原生证据继续暂缓。

同日 macOS 增量实施与独立 Verification Goal 通过：任务创建/完成契约可恢复，File/Document 成功输出自动进入不可变清单，Task Space 只展示 240 字以内摘要并保持渐进加载。Windows 原生证据按主人决定继续暂缓，Story 保持 `verifying`。
