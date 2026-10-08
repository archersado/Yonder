# Verification Goal：TM-S3 Driver 轮次可见性

## 目标

在 macOS 正式 Yonder GUI 与正式 Gateway/CUA Driver 链路中，验证顶部控制条逐轮显示派发和 Observe 结论，且不会泄露敏感参数或误报任务完成。

## 通过条件

- Hub/Port 单元测试覆盖执行中、成功、明确失败、unknown、容量上限、replan 保留与清理。
- Webview 回归覆盖轮次顺序、状态可访问文本、无正文/参数/Payload 和日志不可用降级。
- 正式 macOS GUI 样本至少产生两轮 Driver 记录，原生 AX/截图可见轮次与结论；任务成功后控制条正常收起。
- Windows 原生证据按用户决定暂缓。

## 结果

2026-10-08 自动化验证通过：

- `cargo test --offline --locked -p yonder-desktop`：lib 27 项、main 11 项全部通过；覆盖轮次执行中/成功/失败/unknown、六轮上限、replan 保留、Hub 清理、动作摘要脱敏与发布契约。
- `node apps/desktop/check-cua-control.mjs`：轮次顺序、状态可访问文本、敏感正文不进入日志及既有接管/确认交互全部通过。
- 控制条原生尺寸契约和 AX 探针已更新为 560×300，并要求存在“Driver 轮次”标题。

正式 macOS GUI 的两轮真实 Driver 样本尚待重建产品后执行，因此本 Goal 仍为进行中，不能 Archive。
