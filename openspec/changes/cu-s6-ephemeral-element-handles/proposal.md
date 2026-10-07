# Proposal：CU-S6 临时观察元素句柄

Story: CU-S6

关联 [CU-S6](../../../docs/specs/epic-CU/story-CU-S6/README.md) 与 Accepted AD-CU-10。Architecture Impact：architecture-change（临时 Observation 协议与 Worker 运行态引用）。

## Why

Sky 已取得可操作 AX 元素，但 Gateway 只返回数量和截图，导致慢脑无法合法生成 `observed_element_index`，视觉点击失败后持续交回。

## What Changes

- 协议 1.44 为 `ComputerObservation` 增加临时元素句柄与 `observation_ref`。
- 元素索引候选必须绑定产生它的 Observation 引用。
- Sky Worker 裁剪可操作元素、缓存最近引用，并在动作前重新 Observe 校验。
- Gateway 只返回临时句柄，不持久化、不投影到 UI。
- 增加协议、Application、Adapter/Gateway 合约与 macOS 正式验证。

## Non-goals

不暴露完整 AX transcript或静态正文，不建立持久元素索引，不增加第二 Driver，不以回执代替动作后 Observe，不自动重试 unknown。
