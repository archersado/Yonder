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

同日企业微信复核证明此前“企业切换通过”缺少目标企业名称事实，已撤回该结论。当前补充同尺寸 transcript 代次校验、装饰性 image/disabled 候选过滤，并要求正式回归直接观察到目标企业名称后才能通过。

后续正式对照撤销了“动作使用逻辑点”的推断：精确原图坐标经 2x 除法后稳定失效，既有成功点与失败点并非同一视觉位置。Story 继续处于实施阶段，并恢复从 Gateway 到 Sky 的原图像素不变契约。

重建后的企业微信正式负样本进一步发现：Worker 已有 `failure_stage` 在 Adapter 边界被折叠为通用 `observe-failed`，用户与慢脑无法判断坐标动作是超时、被拒绝还是仅未观察到效果。依 AD-CU-12，Story 增加有界 Driver 错误码，经既有 Attempt 事件同步透出 Gateway、任务面板和顶部 Driver 轮次。
