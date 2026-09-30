# EX-S2 CUA目标意图与发送确认 Verification Goal

状态：自动化通过，待正式macOS产品样本。Windows按用户决定暂缓，不Archive。

验证必须覆盖：协议不回显敏感值、内存容量/期限/任务归属、动作语义映射、元素与视觉候选、顶部确认、批准一次消费、拒绝/过期/unknown不重试，以及正式Yonder企业微信多步骤样本。

## 2026-09-30 自动化结果

- Rust协议、Application、Adapter、CLI及Desktop共180项测试通过。
- 隔离Worker样本完成5个元素语义动作，确认私有字段在调用SDK前剥离；元素歧义样本确认不执行副作用并只返回临时视觉证据。
- OpenSpec严格校验与Node语法检查通过。
- 正式签名macOS Yonder的企业微信样本仍待下一步运行，未据自动化结果提前Archive。
