# CU-S5 v2 固定样本目录

版本：`agent-desktop-spike-samples/v2`。机器定义见 [sample-definitions-v2.json](sample-definitions-v2.json)。完整继承 [CU-S4 v1 样本目录](../cu-s4-agent-desktop-driver-spike/sample-catalog.md) 的 15 个必做样本、动作、predicate、oracle、时限和零自动重试；其 SHA-256 与 v1 definitions 哈希均写入 v2 JSON。v2 新增门禁样本，不删除或放宽 v1 样本。Windows/macOS 使用同一行的通过语义，平台原生实现分别取证。

| 样本 | Windows | macOS | 判定 |
|---|---|---|---|
| V2-SOURCE-01 | 固定 Windows/core 接缝与源码哈希 | 固定 macOS/core 接缝与源码哈希 | v1 证据来源不变；v2 新来源独立绑定 |
| V2-IDENTITY-01 | 同窗 UIA 按钮 A→同属性 B | 同窗 AX 元素 A→同属性 B | 旧保留对象失效，零重绑定、零副作用 |
| V2-IDENTITY-02 | 整窗重建、尝试完整 HWND 复用 | 整窗重建并替换 AX 对象 | 旧对象拒绝；未造出完整 HWND 复用记未验证 |
| V2-DELIVERY-01 | UIA/SendInput 每个支持动作终端 | AX/CGEvent 每个支持动作终端 | 最终交付仍绑定旧对象；不能证明的动作拒绝 |
| V2-INPUT-01 | 10 轮真实鼠标/键盘 | 10 轮真实鼠标/键盘 | 零漏分，observer 故障拒绝 |
| V2-INPUT-02 | 10 轮当前 Worker 与 10 轮其他程序注入，含伪造标记 | 等价 CGEvent 注入与伪造字段 | 零误分；未知冻结 |
| V2-INPUT-03 | 观察器超时/重启、桌面/会话切换、Worker 崩溃 | Event Tap 禁用/重启、会话变化、Worker 崩溃 | 全部冻结，零动作帧 flush |

阶段 0/1 样本不执行桌面动作。真实 `IDENTITY-01` 等 v1 动作样本只在 AD-CU-08 后续修订批准后运行。`environment_unavailable` 不等于 pass；到 2026-11-11 任一必做门禁未通过即不提出产品 Apply。
