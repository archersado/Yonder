# EX-S3 产品需求

## 问题、范围与来源

原始需求：产品简报「两条执行路径」及补充材料「CUA 与 BUA 的 Task Space」「执行原则」要求 AX 优先、浏览器复用 ego-lite、观察—操作—验证、CUA 单前台租约。后续用户变更（2026-09-21）把 Jev 决策放在 CUA/BUA Driver 之上。架构约束：架构主干「执行能力」及既有 CU/BU ADR；Yonder 不复制 ego-lite Task Space。参考 [jev-ultrafast](https://github.com/browser-use/jev-ultrafast) 的动态编号目标/新鲜度复核为待验证设计建议，不能直接搬其浏览器 Runtime。

## 验收映射

| ID | 对应来源 | 验收 |
|---|---|---|
| EX3-01 | 用户快脑变更 | CUA 与 BUA 各从最新可信 Observe 形成当前可用的操作和兼容目标；Jev 只返回受支持组合，不返回坐标、选择器、JS 或任意 SDK 参数。 |
| EX3-02 | 原始观察—操作—验证 | 派发前目标新鲜度/身份复核，动作后 Observe；目标变更、遮挡、失焦、页面跳转时拒绝旧决策并交回，不盲点。 |
| EX3-03 | 既有 Task Space/租约 | CUA 单前台租约和用户输入暂停有效；BUA 保留 ego-lite Task Space 归属与 external_task_ref，不在 Yonder 造第二浏览器空间。 |
| EX3-04 | 用户质量目标 | 同任务与慢脑逐步决策基线对照成功率、误动作、token、时延，`DONE` 独立验证。 |
| EX3-05 | 双平台围栏 | 分别提交 macOS/Windows Driver 与 UI/权限相关结构化或视觉证据；Windows BUA 不可用时明确 capability_unavailable，不伪报覆盖。 |

## 待审建议

首批只选 EX-S1 证明可枚举且可复核的动作子集；自由文本由慢脑计划提供。浏览器使用 ego-lite 公开集成面，若不支持稳定目标引用则停止 BUA 子范围，不用第二浏览器引擎兜底。
