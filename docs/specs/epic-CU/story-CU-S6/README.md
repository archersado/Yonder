# CU-S6 临时观察元素句柄

Story: CU-S6
Epic: CU
Status: implementing
OpenSpec: cu-s6-ephemeral-element-handles

[OpenSpec Change](../../../../openspec/changes/cu-s6-ephemeral-element-handles/)

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 目标

把 Sky 已取得但目前只在 Worker 内可见的 AX transcript 收敛为临时、无正文、可验证的控件句柄，使慢脑能在视觉坐标失败后提交绑定同一次 Observation 的 `observed_element_index`，并继续由动作后 Observe 判断是否成功。

Windows 原生验证继续按主人决定暂缓；macOS 完成协议、Worker、Gateway 合约和正式企业微信样本后进入 verifying。
