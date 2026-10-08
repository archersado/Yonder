# Proposal：CU-S6 临时观察元素句柄

Story: CU-S6

关联 [CU-S6](../../../docs/specs/epic-CU/story-CU-S6/README.md)、Accepted AD-CU-10 与 Accepted AD-CU-11。Architecture Impact：architecture-change（临时 Observation 协议、Worker 运行态引用与截图坐标契约）。

## Why

Sky 已取得可操作 AX 元素，但 Gateway 只返回数量和截图，导致慢脑无法合法生成 `observed_element_index`，视觉点击失败后持续交回。

## What Changes

- 协议 1.44 为 `ComputerObservation` 增加有界 transcript、临时元素句柄与 `observation_ref`。
- 元素索引候选必须绑定产生它的 Observation 引用。
- Sky Worker 裁剪可操作元素、缓存最近引用，并在动作前重新 Observe 校验。
- Sky Worker 将 AX selectable row 导出为可操作句柄，并在坐标调用报错后继续唯一一次动作后 Observe。
- 含截图的 Observation 返回原始像素宽高；所有坐标动作绑定同次引用并由 Worker 校验尺寸与边界。
- 同尺寸但 transcript 已变化时拒绝旧坐标；装饰性 image 与 disabled 节点不进入独立动作候选。
- macOS Adapter 将通过门禁的原图像素不变传给 Sky，不读取、保存或应用显示器 backing scale。
- Gateway 只在当前响应返回有界 transcript 与临时句柄，不持久化、不投影到 UI。
- 增加协议、Application、Adapter/Gateway 合约与 macOS 正式验证。

## Non-goals

不持久化 AX transcript、元素标签或截图几何，不建立持久元素索引，不增加第二 Driver，不以回执代替动作后 Observe，不自动重试 unknown，也不向 Agent 暴露窗口原点、PID 或显示器倍率。
