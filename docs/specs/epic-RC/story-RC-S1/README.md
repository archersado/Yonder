# RC-S1 手动录制与不可变时间线

Story: RC-S1

2026-09-17显式接管意图入口PASS：仅打包Task Space的专用`user_takeover`命令可声明本地用户意图，通用JSON、Agent、CLI和普通输入不能建立该意图。Recording与用户控制租约仍未启用，完整Story保持未完成。
Epic: RC
Status: design-review
OpenSpec: rc-s1-recording-capture-spike

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

产品边界与首个原生采集 Spike 已按 Proposed AD-RC-01 定义；产品 Recording 协议与持久化仍未授权。派生、审阅和回放已拆至RC-S2，默认关闭不变。

## OpenSpec 与验证

[macOS原生采集Spike](../../../../openspec/changes/rc-s1-recording-capture-spike/proposal.md)已建立。先验证来源、隐私排除、有界队列和停止边界；通过前不接产品任务库。Windows按用户决定暂缓，完整Story不进入Done/Archive。

2026-09-17来源Spike结论FAIL：CGEvent字段会把CUA注入误判为用户；IOHID关联虽能拒绝注入，却无法在三轮物理正样本中稳定证明用户来源。两条路线均未接产品，Recording继续默认关闭；需重做来源契约后再继续隐私与持久化。

2026-09-18：依据 AD-RC-01 修订为“受控会话输入”契约。显式接管租约内事件只可作为交回前 Observe 证据，不宣称物理用户来源、不自动回放；先完成租约、隐私与停止 Spike，产品时间线仍未授权。

2026-09-20：受控会话、租约外拒绝、系统安全输入与停止边界的 macOS 无正文子范围已验证。用户排除应用样本未完成：临时探针无法在不抢占前台的情况下稳定读取排除目标身份；按用户决定暂缓，RC-S1 保持 `design-review`，不得接入产品录制或回放。
