# EX-S5 独立 Verification Goal

状态：待运行

## 目标

证明安全成功样本可形成有界经验；后续 handback 会先召回并把提示交给唯一归属慢脑；无命中、存储异常、unknown 和敏感动作均安全降级且不阻塞任务。

## 自动化

- Application：字段校验、敏感动作拒绝、排序与条数边界。
- Adapter：schema 迁移、相同键聚合、精确召回、损坏/异常降级。
- Desktop：handback → recall → AG-S5 replan 输入顺序、空结果兼容、跨 Agent 隔离。

## macOS 产品证据

使用隔离 fixture 形成一条已核验 CUA 激活经验，再创建同类失败任务触发 handback；记录结构化日志证明提示在慢脑规划前到达且不含正文/坐标/临时句柄。Windows 原生证据本轮暂缓。
