# EX-S3 架构设计

## 边界与依赖

CU Adapter 和 ego-lite Bridge 各输出当前 Observe 可支持的操作与可信目标引用；Application 对候选做授权、任务范围和兼容性过滤，再交快脑 Port 选择。候选只包含目标编号、操作类别及必要的最小可见描述，不允许 Jev 输出坐标、选择器、JS 或 SDK 原始调用。文本输入仅引用慢脑计划中已定的内容；缺参交回慢脑。派发前 Driver 重新解析目标并验证当前窗口/页面、对象身份、焦点/遮挡及权限；变化时丢弃决策，重新 Observe 或交回，不尝试模糊同名匹配。

EX-S3 只消费 EX-S2 已冻结计划片段授权后的 Observe 候选，并在每次派发后重新生成候选；跨 Observe 复用目标编号、选择器或旧元素引用一律禁止。

## 状态与契约

CUA 继续使用唯一前台租约与 trycua SDK 监管 Worker；用户输入优先暂停。BUA 继续以 ego-lite 的 Task Space、external_task_ref 和其受支持操作为准。若公开 Bridge 无法提供稳定候选/新鲜度保障，BUA 子范围停止，不绕过 ego-lite 自建浏览器连接。每步后置 Observe 与 TM 任务事件链一致；`DONE` 需由独立状态断言验证。

### 候选契约

- Driver 输出 `observation_id`、`candidate_id`、`action`、`target_ref`、可选 `text_ref`、`enabled` 与 `disabled_reason`；这些字段只在当前 Observe 内有效。
- Application 只把通过任务授权、能力范围和资源门禁的候选交给 Jev；Jev 返回 `candidate_id` 或交回原因，不生成新字段。
- Driver 在派发前必须重新解析 `target_ref`；目标缺失、身份变化、页面跳转或遮挡时返回 `stale_target`，不尝试模糊匹配。
- 动作完成后必须重新 Observe；下一轮候选重新生成，不继承上一轮编号、目标或文本引用。

当前实施基线保持 AD-EX-01 的“每步 Jev 决策”。Proposed AD-EX-04 的有界动作配方不进入本 Story 的运行时代码、协议或数据库设计；只有该 ADR 被 Accepted 且 EX-S2 计划片段契约修订后，才允许另行创建对照 OpenSpec。

## 失败与验证

验证包括受支持/不支持控件、目标替换、页面跳转、遮挡、网络断开、用户干预、后置 Observe 失败。macOS/Windows 分别提供原生证据；Windows BUA 的既有限制不可标为通过。
