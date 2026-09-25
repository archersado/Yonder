# CM-S1 架构设计

## 边界与依赖

独立 Command Adapter，经 Application Port 与后续统一任务生命周期；不借 CUA 执行命令。Accepted AD-CM-01 当前只授权 macOS Runtime，Windows 返回 unavailable。风险确认与任务事实接线未定案前不接 Gateway。

## 状态与契约

MVP 按 Accepted AD-ST-01 使用未加密 SQLite 保持任务当前事实源，SQLCipher 延期至 ST-S2；UI 仅持展示快照。传输类型从 Rust 派生。改变协议/持久化/边界前先补 ADR，不为本 Story 另建状态系统。

## 失败与验证

Application 定义不含技术依赖的 `CommandRequest/CommandOutcome/CommandPort`。macOS Adapter 使用 Rust 标准库与原生进程组，不安装新依赖；请求边界拒绝相对或不可规范化的 program/cwd、NUL、超额参数与环境，并 `env_clear`。stdout/stderr 各有界读取，停止后必须确认整个进程组消失。

Runtime 是同步的受监管调用：调用方提供只读取消信号，Adapter 轮询取消、超时和输出超限。它不持有任务状态、不创建线程池、不保存正文。未来 Gateway 必须先经 TM-S7 统一启动、准入、风险确认和 attempt 登记，结果再同事务写任务事实；本增量不建立旁路。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 架构影响

本增量新增 Application Command Port 和 macOS Adapter，依赖方向保持 adapters→application；不变更协议、SQLite、任务状态所有者或技术栈。Windows 编译路径稳定返回 unavailable。
