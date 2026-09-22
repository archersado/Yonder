# development-fence Specification

## Purpose
定义模块化 Epic/Story、OpenSpec 与独立 Verification 的规划及交付围栏，防止无设计实施、跨模块混更或以历史迁移冒充完成。

## Requirements

### Requirement: 模块 Epic 与 Story 设计围栏

规划与 PR 门禁 SHALL 以下述场景执行。

关联 EN-S1。

#### Scenario: 规划完整性
每个 epic-模块/README.md 定义唯一 Epic；story-ID/ 内有 README.md 和非空的产品需求、架构设计、视觉交互设计文档，包含各自必需章节。缺失、空章节、ID 或归属不符时检查失败。

#### Scenario: 实施前置
PR 必须关联唯一新目录 Story、OpenSpec 和独立 Verification。Story 处于 draft/design-review/deferred 时拒绝；ready/implementing/verifying/done 仍须双向关联及文档完整，不能用旧平铺文档绕过。

#### Scenario: 迁移与保留
历史实现不被迁移自动标记完成；证据保留，新旧 ID 映射可查。无 UI Story 仍说明调用交互和错误反馈。用户暂停项保持 deferred。
