# Proposal：EN-S2 发布签名与分发

Story：EN-S2
规划：`docs/specs/epic-EN/story-EN-S2/README.md`。

依据产品简报「MVP 主干链路」「待决策事项」与架构主干「非功能与发布」，定义 Windows/macOS 发布签名、版本固定、升级备份与双通道分发。Architecture Impact：conforming；不改变运行时代码、协议、持久化或依赖方向。

## 为什么

发布必须保证 desktop、CLI、MCP、IPC 协议、迁移和 Driver manifest 在同一版本内交付，并在升级失败时保留数据。

## 变更

建立最小发布脚本、版本冻结、签名与公证、升级前备份和结构化发布证据；不引入自更新服务或云端发布平台。

## 影响

- Story：EN-S2
- 架构影响：conforming
- 相关决策：沿用架构主干发布约束
- 涉及模块：仅发布脚本与验证工具
- 协议影响：无
- 迁移影响：只验证既有迁移，不新增迁移
