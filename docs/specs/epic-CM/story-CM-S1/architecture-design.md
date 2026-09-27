# CM-S1 架构设计

## 边界与依赖

独立 Command Adapter，经 Application Port 与后续统一任务生命周期；不借 CUA 执行命令。Accepted AD-CM-01 当前只授权 macOS Runtime，Windows 返回 unavailable。Accepted AD-CM-02 定案 macOS 首批 Agent Command 的本机批准：TaskHost 内存 Registry 持有提议和批准，Gateway 只传不透明引用，Application 是唯一启动协调者。

## 状态与契约

MVP 按 Accepted AD-ST-01 使用未加密 SQLite 保持任务当前事实源，SQLCipher 延期至 ST-S2；UI 仅持展示快照。传输类型从 Rust 派生。改变协议/持久化/边界前先补 ADR，不为本 Story 另建状态系统。

## 失败与验证

Application 定义不含技术依赖的 `CommandRequest/CommandOutcome/CommandPort`，并定义 Command Approval Port：提议只在 TaskHost 内存登记，批准只允许 LocalUser UI 写入，执行原子消费绑定 task/owner/digest/sequence 的一次性引用。macOS Adapter 使用 Rust 标准库与原生进程组，不安装新依赖；请求边界拒绝相对或不可规范化的 program/cwd、NUL、超额参数与环境，并 `env_clear`。stdout/stderr 各有界读取，停止后必须确认整个进程组消失。

Runtime 是同步的受监管调用：调用方提供只读取消信号，Adapter 轮询取消、超时和输出超限。它不持有任务状态、不创建线程池、不保存正文。Gateway 的提议不触碰 Runtime；执行用例先经 TM-S7 统一启动、准入和 attempt 登记，再消费批准并派发，结果再同事务写任务事实；不建立旁路。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 架构影响

本增量新增 Application Command Approval Port、Rust 协议方法和 desktop 内存组合根；不新增 SQLite、第二执行栈或 Adapter 间依赖，依赖方向保持 desktop→adapters/application、adapters→application/protocol。Windows 编译路径稳定返回 unavailable。
