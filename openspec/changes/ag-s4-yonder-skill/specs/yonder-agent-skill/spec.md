# Yonder Agent Skill Delta Specification

## ADDED Requirements

### Requirement: Skill必须从同一Yonder任务编排已发布能力

系统 MUST 提供可发现的Yonder Skill；一个用户目标只创建一个Yonder任务，并从当前MCP工具发现BUA、CUA、Document与Command能力。能力缺失时 MUST 失败关闭，不得调用旁路执行栈冒充完成。

#### Scenario: 复合目标保持单任务

- **WHEN** 用户目标依次需要浏览器、文档和命令能力
- **THEN** Skill复用同一`task_id`和最新`sequence`，按需加载能力模块，不为每种能力另建任务

#### Scenario: 目标能力未发布

- **WHEN** 当前Yonder未发现目标能力工具
- **THEN** Skill明确报告不可用，且不直接使用shell、其他浏览器、文件写入或Driver替代

### Requirement: CUA慢脑必须提交完整受限计划片段

系统 MUST 指导慢脑一次提交覆盖当前有界子目标的多槽位片段；每个槽位表达一个顺序语义步骤，同槽位候选只能是该步骤的替代实现。元素或原生语义候选 MUST 优先，视觉候选只在元素缺失、不唯一或不可操作时兜底。

#### Scenario: 企业微信消息准备

- **WHEN** 目标需要启动应用、解析会话、准备草稿并在确认后发送
- **THEN** 慢脑一次生成多个顺序槽位，由Yonder/Jev在片段内逐步Observe和连续推进，而不是每动作交回慢脑

#### Scenario: 元素不可用

- **WHEN** 当前步骤的元素定位缺失、不唯一或不可操作
- **THEN** Yonder可选择同槽位的视觉兜底候选；若候选耗尽才交回慢脑，不默认每步截图

#### Scenario: 副作用结果待核实

- **WHEN** 发送、删除或其他副作用返回`unknown`、超时或断连
- **THEN** Skill先读取新鲜任务与事件并停止自动重试，等待重新Observe或用户核实

### Requirement: BUA必须复用Yonder登记的ego-lite空间

Skill MUST 只操作Yonder创建并持久化引用的唯一Task Space，保留同空间恢复、Page复用、用户交接、Observe与完成清理语义。

#### Scenario: 浏览器需要用户登录

- **WHEN** 页面要求用户登录或处理浏览器权限
- **THEN** Skill经Yonder hand-off并停止Agent操作，交回后恢复同一空间，不新建未关联空间

### Requirement: Skill包必须可追溯且不包含运行时状态

Skill MUST 声明自身版本、最低Yonder协议、验证平台和ego-browser上游版本；包 MUST NOT 包含凭据、用户数据、运行时代码、协议模型副本或任务状态。

#### Scenario: 安装与卸载

- **WHEN** Skill目录被安装或移除
- **THEN** Agent发现状态相应变化，Yonder桌面数据、Runtime和已有任务不受影响
