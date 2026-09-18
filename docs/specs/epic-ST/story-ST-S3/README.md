# ST-S3 MVP 未加密任务存储

Story: ST-S3
Epic: ST
Status: verifying
OpenSpec: st-s3-mvp-task-storage

按Accepted AD-ST-01实施独立存储子范围。三份设计已完成工程审阅，不涉及UI、认证、桌面执行器或密钥；复用TaskStore与现有事务。工作区含其他Story未提交变更，保留当前现场，本次不创建或合并PR，后续须隔离变更。

Change：openspec/changes/st-s3-mvp-task-storage/；实现后创建独立Verification Goal，不以本Story通过替代DS-S2。

显式未加密入口已实施，10项Adapter回归及架构检查通过。[独立Verification Goal](../../../../openspec/changes/st-s3-mvp-task-storage/verification-goal.md)记录AC1–4本机PASS。Windows与PR未完成，不Archive，不据此宣称桌面任务总览已接通。
