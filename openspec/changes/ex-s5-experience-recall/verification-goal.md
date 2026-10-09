# EX-S5 独立 Verification Goal

Story: EX-S5
Result: PASS
状态：macOS 自动化通过；真实产品样本与 Windows 原生证据待补

## 目标

证明安全成功样本可形成有界经验；后续 handback 会先召回并把提示交给唯一归属慢脑；无命中、存储异常、unknown 和敏感动作均安全降级且不阻塞任务。

## 自动化

- Application：字段校验、敏感动作拒绝、排序与条数边界。
- Adapter：schema 迁移、相同键聚合、精确召回、损坏/异常降级。
- Desktop：handback → recall → AG-S5 replan 输入顺序、空结果兼容、跨 Agent 隔离。

## macOS 产品证据

2026-10-09 在 macOS 运行：

- `cargo test -p yonder-application --lib`：82/82 通过，证明坐标被剥离、敏感动作不晋升、当前槽位形成精确查询。
- `cargo test -p yonder-adapters execution_experience --lib`：通过，证明 schema 25、相同键聚合、最多三条与独立投影线程的写后召回。
- `cargo test -p yonder-desktop --lib`：30/30 通过，证明经验段先于 `task.get` 新鲜事实要求进入归属慢脑输入，且浮窗维持 replan 状态。
- `python3 scripts/check_architecture.py`：通过。
- Adapter 全量 86 项中与 schema 25 相关的原有迁移失败已修复；其余 85 项通过。既有 `command::timeout_and_cancellation_stop_parent_and_descendant` 在全量运行中仍有 PID fixture 空输出的环境性波动，单测独立重跑通过，与本 Change 无代码交集。
- 首次真实产品计划提交发现协议拒绝 `launch-application + bundle_id`，已把校验收敛为单一、最长 255 字节、含反向域分隔且仅含字母数字/点/连字符的 bundle ID；路径、无域名和多余字段均拒绝。

结构化测试同时断言经验提示不含原坐标、旧 `observation_ref`、任务正文或消息内容。召回通过独立有界 worker，调用方最多等待 20ms；超时按空结果继续 replan。Windows 原生证据本轮暂缓；真实安装包任务样本留作 Archive 前最终证据。
