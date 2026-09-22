# EX-S4 产品需求

## 问题、范围与来源

原始需求：产品简报「目标用户与工作场景」的文档与文件任务、「首批权限」的 command:execute、可见且可控；架构主干「Command、File 与 Document」要求结构化命令、OOXML 保真、默认另存、文件锁和原子写入。后续用户变更（2026-09-21）要求快脑覆盖 Office 与 Command 层。架构约束：Office 通过既有 Document Port/OOXML Adapter 或另行显式 CUA 步骤，不在本 Story 新建 Office Driver；FI/DO/CM 独立门禁先通过。

## 验收映射

| ID | 对应来源 | 验收 |
|---|---|---|
| EX4-01 | 用户快脑变更 | 快脑可在慢脑预先授权且参数完整的 Document/Command 候选中选下一步，缺少内容、路径、程序或参数时交回慢脑。 |
| EX4-02 | 架构 Document 边界 | OOXML 修改仍用 expected_hash、锁检查、临时文件/结构验证/原子提交，默认另存；覆盖须显式要求。不能让 Jev 自由生成改写正文。 |
| EX4-03 | 架构 Command 边界 | 只执行显式 program+args+cwd+env；Shell、提权、安装、删除、支付、发送仍须现有确认，模型置信度不能代替确认。 |
| EX4-04 | 架构状态/恢复 | 执行结果不明为 `unknown` 且不自动重试；每步 Observe 和任务事件/Outbox 同事务，取消/接管优先。 |
| EX4-05 | 用户 token/质量目标 | 文档与命令固定样本对照慢脑调用/token、时延、成功率及误副作用；Windows/macOS 分别验证并经独立 Goal。 |

## 待审建议

先验证「从完整结构化候选中选择」是否真实减少慢脑调用；若慢脑仍需逐项提供全部候选，保留数据并可能淘汰该子范围，不为了覆盖四类 Driver 而引入无收益循环。
