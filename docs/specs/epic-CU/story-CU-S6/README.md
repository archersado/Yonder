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

把 Sky 已取得的 AX transcript 作为有界、临时 Observation 返回慢脑，同时派生可验证控件句柄，使慢脑能理解当前界面并在视觉坐标失败后提交绑定同一次 Observation 的 `observed_element_index`；动作是否成功仍由后置 Observe 判断。

Windows 原生验证继续按主人决定暂缓；macOS 完成协议、Worker、Gateway 合约和正式企业微信样本后进入 verifying。
