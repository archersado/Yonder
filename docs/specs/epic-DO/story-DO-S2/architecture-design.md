# DO-S2 架构设计

## 边界与依赖

Application定义Document Port和语义请求；Rust OOXML Adapter位于`crates/adapters`，依赖Application，不访问任务SQLite。首段只接收和返回内存字节；Desktop组合根与Gateway不接线。文件路径规范化、文件身份、同文件写租约与宿主锁检测由FI-S1提供，Adapter不得自行建立第二套身份规则。

## 状态与契约

读取结果包含格式、语义节点、有界文本和SHA-256。写请求包含输入文件身份、输出绝对路径、`expected_hash`及语义操作；默认要求输入输出不同。输出事实只在原子提交成功后返回；任务状态、事件和Outbox仍由TM拥有。

## 失败与验证

区分非法路径、格式不支持、hash冲突、目标存在、文件锁、语义目标不存在/不唯一、结构校验失败和I/O失败。提交前失败不得留下正式输出；提交结果不明时标为unknown且不自动重试。

### 验证

复用DO-S1统一合成与Microsoft Transitional样本；新增File Port身份/锁夹具、原文件并发变化、目标存在、临时文件清理和真实Gateway任务。Windows/macOS分别提供结构化日志或Office/WPS打开证据；Windows当前暂缓但门禁保留。

## 架构影响

新增Application Document Port与Rust Adapter，遵守adapters→application依赖；若新增Gateway协议，须先建立DO-S2 Architecture Decision并从Rust类型生成Schema/TS。
