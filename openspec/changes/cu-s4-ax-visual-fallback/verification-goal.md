# 独立 Verification Goal：CU-S4 AX失败后的同窗口视觉降级

状态：实施中

复核AX优先、空树/未确认动作同窗口截图、`UnknownObserved`视觉事实返回、通用坐标文本foreground注入、失败步骤投影、无动作重放与临时证据清理。

## 当前自动化证据

- `check-cua-action-failure-visual-fallback.py`隔离证明：空AX树和refused动作均产生同窗口截图，包含SDK只返回内嵌图片而未写文件的情况；通用坐标文本被注入foreground及精确target；原动作只派发一次。
- `check-cua-generic-plan-actions.py`隔离证明：通用聚焦、输入与激活动作沿同一可信窗口连续派发，不依赖消息意图引用，且输入仍受协议边界约束。
- 正式打包Yonder任务`task_9c3fb2493a822115b43c04dd3c911716`经MCP/Gateway一次接受5槽位慢脑片段；原生AX与窗口截图复核顶部控制条显示“慢脑已提交5个受限步骤”、当前附近四步、剩余一步、快脑决策与正在执行步骤，窗口顶部居中且任务面板状态为运行中。任务在聚焦步骤后带视觉Observation交回慢脑，读取快照/事件后已取消清理。
- 既有`check-cua-visual-fallback.py`与`check-cua-visual-focus.py`回归通过，元素多义、受保护引用、一次性焦点与敏感字段剥离不变。
- Application单元测试证明`UnknownObserved`视觉Observation不会被TM-S9运行时丢弃。
- 2026-10-08修复通用搜索输入的动作确认：唯一搜索文本框只派发一次Sky `set_value`，并且只有同一应用的新鲜Observe从该控件读回目标AX值才标记`confirmed`；回执成功但值未变化仍为`unknown`，不重放输入。`check-sky-cua-worker.py`同时覆盖成功与未确认两侧，`check-sky-message-worker.py`证明消息引用路径未回归，Adapter CUA 12项测试通过。
- 由`dev`构建的macOS预览包通过`check-cua-package.py`：唯一Driver仍为外部固定Sky 0.7.1，Worker资源与源码一致，Jev SDK为0.6.0；通用计划动作和动作失败视觉交回隔离回归通过。重启GUI后旧MCP transport关闭，待客户端重新连接后继续正式QQ音乐样本，不以隔离证据替代产品验证。
- 2026-10-09正式任务`task_c35e925a59fe3866ee41b7e2fb6374e0`稳定复现：QQ音乐启动与`⌘F`聚焦均已Observe，随后元素`set_value`返回`driver-target-stale`。Worker现允许同任务、同应用、紧邻且已确认的`⌘F`焦点凭据单次改用`type_text`；凭据在派发前消费，仍以新鲜AX值确认，失败不回退、不重试。隔离夹具覆盖该路径及凭据消费后的`set_value`失败边界。

正式macOS Yonder QQ音乐样本、完整Rust回归和OpenSpec严格校验完成前，本Goal不记PASS、不Archive。Windows按用户决定暂缓。
