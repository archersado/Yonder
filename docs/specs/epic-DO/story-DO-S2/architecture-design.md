# DO-S2 架构设计

## 边界与依赖

Application定义Document Port、语义请求及文件组合用例；Rust OOXML Adapter位于`crates/adapters`，依赖Application，不访问任务SQLite。文件路径规范化、文件身份、源快照保护、同文件写租约与宿主锁检测由FI-S1 File Port提供，Document Adapter不得自行建立第二套身份规则。Desktop组合根与Gateway仍不接线。

## 状态与契约

读取结果包含格式、语义节点、有界文本、规范路径、文件身份和SHA-256。写请求包含授权根、源绝对路径、输出绝对路径、`expected_hash`及语义操作；默认另存使用 `create-new`，覆盖只接受 LocalUser。输出事实只在原子提交成功后返回；任务状态、事件和Outbox仍由TM拥有。

File Port 的 guarded write 在源身份租约与共享宿主锁下写目标暂存文件，并通过组合校验器在提交前复核源哈希和OOXML格式。覆盖源文件复用 File Port 的独占目标锁及 `expected identity + expected hash`，不再额外取得同一身份租约造成自锁。

## 失败与验证

区分非法路径、格式不支持、hash冲突、目标存在、文件锁、语义目标不存在/不唯一、结构校验失败和I/O失败。提交前失败不得留下正式输出；提交结果不明时标为unknown且不自动重试。

### 验证

复用DO-S1统一合成与Microsoft Transitional样本；新增File Port身份/锁夹具、原文件并发变化、目标存在、临时文件清理与可信覆盖拒绝。真实Gateway任务另建 Change。Windows/macOS分别提供结构化日志或Office/WPS打开证据；Windows当前暂缓但门禁保留。

## 架构影响

新增Application Document Port与Rust Adapter，遵守adapters→application依赖；若新增Gateway协议，须先建立DO-S2 Architecture Decision并从Rust类型生成Schema/TS。
