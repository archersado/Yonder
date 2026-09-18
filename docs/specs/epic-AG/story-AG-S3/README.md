# AG-S3 Agent步骤声明接入

Story: AG-S3
Epic: AG
Status: verifying
OpenSpec: ag-s3-step-declaration

设计：[产品需求](product-requirements.md)、[架构设计](architecture-design.md)、[视觉交互设计](visual-interaction-design.md)。来源既有产品/架构，首批按Accepted AD-AG-04只支持created步骤声明及读取，不解释标签、不启动执行器。完整动作/观察/接管契约保持后续设计门禁。

## 当前状态与前置条件

三份设计与AD-AG-04已完成首批审阅；OpenSpec、协议1.4、Application Port 与 SQLite6 已实施。私有stdio Agent 创建、声明、兼容读取和取消后幂等重试通过，独立验证见关联 Change。生产认证、真实动作/Observe与Windows仍保留门禁。

关联 Change：`openspec/changes/ag-s3-step-declaration/`。
