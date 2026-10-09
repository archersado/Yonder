# EX-S5 已验证执行经验召回

状态：implementing  
关联 Epic：[EX](../README.md)  
OpenSpec：`openspec/changes/ex-s5-experience-recall`

## 目标

当执行片段交回慢脑时，Yonder 先按当前失败动作的稳定特征召回少量、经过成功后置条件验证的执行经验，再通过既有 Agent 输入通道把经验提示与 handback 一起交给归属慢脑。慢脑仍须从 Gateway 读取 `task.get` 与 `task.events` 的新鲜事实并提交新计划；召回结果不是事实、计划或执行授权。

## 交付

- Application 定义有界经验记录、召回查询及校验规则。
- SQLite 保存不含正文、截图、坐标、元素索引和临时句柄的建议性经验投影。
- Desktop 在 handback 后、唤醒慢脑前执行有界召回；失败或无结果不阻塞 replan。
- 协议与 Gateway 不新增第二通道，Driver 和 Jev 不读取经验库。
- 覆盖 Application、SQLite 与 Desktop 组合链路测试；建立独立 Verification Goal。

## 非目标

不做向量库、Embedding、知识图谱、自动技能生成、跨设备同步或旧动作重放；不从 `unknown`、未核验动作回执或敏感副作用中学习成功经验。
