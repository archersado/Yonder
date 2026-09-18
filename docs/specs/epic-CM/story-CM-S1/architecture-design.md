# CM-S1 架构设计

## 边界与依赖

独立 Command Adapter，经 Application 与任务生命周期；不借 CUA 执行命令。技术路线由Proposed AD-CM-01约束，双平台进程树停止Spike通过前不接Gateway。

## 状态与契约

SQLCipher 保持任务当前事实源；UI 仅持展示快照。传输类型从 Rust 派生。改变协议/持久化/边界前先补 ADR，不为本 Story 另建状态系统。

## 失败与验证

候选实现只使用Rust标准库和平台原生进程组/Job Object，不安装新依赖。请求边界拒绝相对program/cwd、NUL、超额参数与环境；不继承完整Agent环境。输出预算与默认超时仍由Spike证据后定案。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 架构影响

本文件为规划迁移，运行时无变化；具体实施按关联 ADR 和 Proposal 声明影响，draft/design-review 不授权实施。
