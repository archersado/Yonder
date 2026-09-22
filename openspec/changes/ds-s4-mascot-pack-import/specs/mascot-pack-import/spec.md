# 桌宠资源包导入增量规格

## ADDED Requirements

### Requirement: 仅导入受限声明式资源包

系统 MUST 只接受含完整标准状态集的声明式 PNG/WebP 资源包；资源包 MUST NOT 包含可执行内容或越出暂存目录的路径。

#### Scenario: 有效资源包

- Given 用户选择符合 manifest 规范的 ZIP
- When 所有条目和图像通过校验
- Then 系统原子激活该包并使桌宠按现有任务事实渲染对应状态

#### Scenario: 无效资源包

- Given ZIP 含路径穿越、未知文件、损坏帧、缺失状态或超限内容
- When 用户尝试导入
- Then 系统拒绝导入、清理暂存内容并保留当前可用资源包
