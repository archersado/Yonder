# Background Native Action Delta Spec

## ADDED Requirements

### Requirement: 后台原生动作不占用前台桌面

系统 MUST 只执行已验证且无需前台键鼠输入的原生能力，并保持前台桌面不变。

#### Scenario: 支持的原生动作

- **WHEN** 隔离fixture显式请求已验证的原生能力
- **THEN** 动作不激活目标应用、不启动trycua且不产生键鼠输入
- **AND** 动作后Observe证明预期条件或返回unknown

#### Scenario: 不支持的原生动作

- **WHEN** fixture请求未注册或需要界面的能力
- **THEN** 返回稳定的不支持或需要前台分类
- **AND** 不自动调用CUA、Command或系统脚本

#### Scenario: 外部应用App Intent metadata

- **WHEN** 已安装应用只发布`Metadata.appintents`而未提供Yonder可链接的具体类型或公共调用API
- **THEN** Yonder不得生成通用App Intent调用器
- **AND** 不得把Shortcuts CLI作为后台原生动作兜底

#### Scenario: 后台动作缺少系统权限

- **WHEN** 原生能力所需权限为未决定、拒绝或受限
- **THEN** 动作返回`permission-required`且不执行副作用
- **AND** 系统授权必须由独立显式用户流程完成，授权后不自动重试原动作

#### Scenario: 已授权EventKit动作

- **WHEN** 提醒事项权限已为full access且隔离样本创建临时提醒
- **THEN** 系统按identifier Observe保存结果并删除测试项
- **AND** 前台应用保持不变，证据不包含提醒正文、列表名或对象标识

### Requirement: 统一任务可观测事实

Spike MUST 产出可映射到任务、步骤和尝试的有界可观测事实。

#### Scenario: Task Space事件映射

- **WHEN** 后台原生动作开始并结束
- **THEN** 探针产出可映射到同一task/step/attempt的执行类别、开始、Observe和结果事实
- **AND** 事实不包含正文、截图、完整Payload或原生对象身份
