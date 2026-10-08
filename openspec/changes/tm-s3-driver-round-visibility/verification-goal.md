# Verification Goal：TM-S3 Driver 轮次可见性

## 目标

在 macOS 正式 Yonder GUI 与正式 Gateway/CUA Driver 链路中，验证顶部控制条逐轮显示派发和 Observe 结论，且不会泄露敏感参数或误报任务完成。

## 通过条件

- Hub/Port 单元测试覆盖执行中、成功、明确失败、unknown、容量上限、replan 保留与清理。
- Webview 回归覆盖轮次顺序、状态可访问文本、无正文/参数/Payload 和日志不可用降级。
- 正式 macOS GUI 样本至少产生两轮 Driver 记录，原生 AX/截图可见轮次与结论；任务成功后控制条正常收起。
- Windows 原生证据按用户决定暂缓。

## 结果

待实施后填写。
