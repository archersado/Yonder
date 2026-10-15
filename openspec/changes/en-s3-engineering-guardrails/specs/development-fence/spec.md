# 研发围栏增量

## ADDED Requirements

### Requirement: 前端协议状态必须可追溯到生成来源

架构门禁 MUST 检查 Desktop 页面使用的协议状态来源；页面不得把手写枚举集合作为第二份 wire 协议模型。

#### Scenario: 页面映射协议状态到展示文案

- **WHEN** Desktop JavaScript 页面使用任务状态、产物可用性或文件授权用途等协议值
- **THEN** 页面显式声明 `crates/protocol/generated/protocol.ts` 为唯一协议来源
- **AND** 架构门禁验证该生成文件存在

#### Scenario: 页面定义第二套协议状态来源

- **WHEN** 页面缺少来源声明、声明路径错误或定义可独立演进的重复协议枚举集合
- **THEN** 架构门禁失败并指出违规页面

### Requirement: Desktop 验证脚本必须与产品源码分区

仓库 MUST 将 Desktop `check-*` 验证脚本统一保存在 `scripts/verification/desktop/`，不得继续散落在 `apps/desktop` 产品根目录。

#### Scenario: 新增或引用 Desktop 验证脚本

- **WHEN** 开发者新增验证脚本或在文档中引用既有脚本
- **THEN** 使用统一验证目录下的仓库相对路径
- **AND** 迁移不得改变脚本原有验证逻辑
