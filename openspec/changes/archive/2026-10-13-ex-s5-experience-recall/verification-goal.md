# EX-S5 独立 Verification Goal

Story: EX-S5
Result: PASS
状态：macOS 自动化、正式产品晋升/召回/UI 与唯一归属慢脑 accepted 投递均通过；Windows 原生证据暂缓

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
- 首次真实产品计划提交暴露出初始槽位也必须声明封闭前置事实；同时补齐此前缺失的 `launch-application` 参数 allowlist：只接受单一、最长 255 字节、含反向域分隔且仅含字母数字/点/连字符的 bundle ID，路径、无域名和多余字段均拒绝。

结构化测试同时断言经验提示不含原坐标、旧 `observation_ref`、任务正文或消息内容。召回通过独立有界 worker，调用方最多等待 20ms；超时按空结果继续 replan。

## 2026-10-13 正式产品样本

从 `dev` 的 `9e6aecb` 重建并临时签名正式 debug `Yonda.app` 后，经安装包内 MCP/Gateway 运行两阶段样本：

- 晋升任务 `task_cce31eda392a7263e5d2fd3895ac5667` 以单一安全 `launch-application(com.apple.TextEdit)` 计划形成真实 Driver Observe；归属慢脑用序号 10 的新鲜 Observation 提交 `achieved`，任务在序号 14 完成，异步经验投影不阻塞任务终结。
- 召回任务 `task_56c3b24293d3326045d96c8005c83ff5` 以相同稳定特征形成序号 5 的真实 Observe，随后归属慢脑提交 `not-achieved@8`。顶部控制窗持续显示“已召回执行经验，等待慢脑连接”，并同时保留已完成步骤和 Driver 已观察轮次，证明经验已从正式产品存储召回且投递状态不再覆盖召回结果。
- 首轮产品样本发现 `mark_replan_delivery` 会覆盖 EX5-06 的召回状态；`9e6aecb` 增加进程内召回标记并将其与 accepted/missing/ambiguous/failed 投递状态组合展示。Desktop 35/35、Application 经验测试 3/3、Adapter 经验测试 1/1、架构检查和 OpenSpec strict 均通过。
- 结构化证据位于 `apps/desktop/evidence/ex-s5-experience-recall-product-20261013/result.json`；不保存正文、截图、transcript、坐标、Observation 引用、完整 Agent Payload 或凭据。

本轮正式产品已经通过 EX-S5 的经验晋升、召回、轻量降级与顶部可观察性范围。

在由 Codex CLI 0.162.1 从启动起持有、且 AG-S8 输入通道已注册的新会话中，终局任务 `task_c945c1ff998e04c643a524a5ec52971f` 再次执行相同无副作用样本：`launch-application` 在序号 5 返回 `action_succeeded=true / observe_valid=true`，归属慢脑提交 `not-achieved@8` 后，Yonder 将两条 `native-launch` 历史经验随 replan 输入投递到当前唯一 `codex-cli` 会话，当前 turn 实际收到该输入。慢脑随后按要求通过 Gateway 读取 `task.get` 与 `task.events`，确认不存在 `unknown` 或待核实副作用，并在序号 9 取消验证任务清理现场。

该 accepted 样本不保存经验提示正文、任务正文、截图、transcript、坐标、Observation 引用、完整 Agent Payload 或凭据；结构化证据只记录数量、序号、连接版本和布尔结果。至此 EX5-01—06 的 macOS Verification Goal 全部通过，可以 Archive；Windows 原生证据继续按主人决定暂缓。
