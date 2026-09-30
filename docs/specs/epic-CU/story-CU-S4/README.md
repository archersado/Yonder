# CU-S4 trycua 单次前台投递升级 Spike

Story: CU-S4
Epic: CU
Status: verifying
OpenSpec: cu-s4-trycua-foreground-delivery-spike

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

2026-09-30正式Yonder企业微信样本证明：固定`@trycua/cua-driver@0.25.0`可以启动并Observe精确窗口，但企业微信后台快捷键、像素点击和引用文本输入均不能取得确认后置事实；把`unverifiable`聚焦当作焦点凭据会把后续敏感文本暴露给错误输入框，已禁止。

本Story仅授权限时macOS升级Spike，对比0.25.0与固定候选0.30.4。Spike通过和AD-CU-07接受前，不修改正式依赖、不改变Gateway协议、不恢复不可核实文本续跑。Windows继续按用户决定暂缓。

## OpenSpec 与验证

[OpenSpec Change](../../../../openspec/changes/cu-s4-trycua-foreground-delivery-spike/)只产生隔离证据和ADR结论。期限为2026-10-02；届时必须接受、缩小或淘汰候选路线。

2026-09-30 macOS独立Goal PASS：0.25.0与0.30.4统一样本均通过，升级无新增收益，AD-CU-07接受“保留0.25.0并接入既有受限前台坐标文本能力”。产品Apply与正式Yonder企业微信复验仍待独立Change；Windows暂缓，因此Story不Archive。
