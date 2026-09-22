# EN-S1 架构设计

## 边界与依赖

依据 AD-DEV-01；迁移历史关联，Python 标准库检查目录/必需文档/状态/双向引用，沿用 CI。

## 状态与契约

SQLite 当前状态表保持任务当前事实源；SQLCipher、FTS 及附件加密按 AD-ST-01 延期至 MVP 后。UI 仅持展示快照，传输类型从 Rust 派生。改变协议、持久化或边界前先补 ADR，不为本 Story 另建状态系统。

本 Story 只改规划元数据：README 的 Story/Epic/Status/OpenSpec 必须唯一，Story ID 与模块及目录一致。状态使用 AD-DEV-01 固定枚举；无 Change 用 `-`，implementing/verifying/done 不允许缺 Change。新 PR 必须关联新目录 Story、相同 Change 和引用 Story 的 Markdown Verification。仍使用 Python 标准库，无运行时变化。

## 失败与验证

本 Story 为用户明确授权的流程调整；运行门禁自测后进入验证，不改运行时。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 架构影响

本文件为规划迁移，运行时无变化；具体实施按关联 ADR 和 Proposal 声明影响，draft/design-review 不授权实施。
