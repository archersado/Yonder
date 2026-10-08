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

macOS 协议、Worker、Gateway 临时 transcript 子范围已通过独立 Verification Goal。2026-10-07 企业切换回归发现可选择行未导出句柄、坐标点击错误会跳过动作后 Observe，已返回实施阶段；Windows 原生验证继续按主人决定暂缓。

2026-10-08 QQ 音乐正式回归进一步证明裸 `x/y` 无法区分原始截图像素与对话缩放坐标；本 Story 依 AD-CU-11 增加 Observation 绑定的截图尺寸与坐标引用，禁止继续猜测 Retina 比例或窗口偏移。
