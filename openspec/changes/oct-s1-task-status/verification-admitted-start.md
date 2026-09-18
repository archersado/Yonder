当前归属 Story：TM-S1；规划：`docs/specs/epic-TM/story-TM-S1/README.md`。旧编号保留历史追溯。

# 独立 Verification Goal：OCT-S1 准入后启动

日期：2026-09-11。关联 OCT-S1、AD-OCT-06 启动补充、task-status 的准入与持久化启动场景。Architecture Impact：conforming。

状态：macOS 核心验证通过；不 Archive、不标记 Story Done。

实现复用已有 admission.try_acquire 与 Application transition，未新增依赖、协议、数据库字段或执行器。成功返回持久化 Task 和占用凭证；失败只释放本次尚未派发的占用，释放异常独立返回。入口仅供可信宿主，不接受外部 Agent 直接调用。

## 实测

`cargo test --workspace --offline --locked`：14 项通过，0 失败；构建 1.43 秒，五项 Adapter 测试 0.40 秒。真实 Cargo 架构检查通过。

新增真实 SQLCipher 临时库双连接测试验证：

- start 返回后第二连接立即读到 running/sequence=2 和追加事件；两个不同资源任务均可持久化 running。
- 相同文件冲突时任务保持 created；容量满时第三个任务无法启动。
- Outbox 插入触发器注入失败，当前状态、事件与 Outbox 全部回滚；本次浏览器占用及容量释放，第一任务文件占用不受影响。
- 旧序号拒绝且释放未派发资源；显式重试成功。中断任务不能经 Start 自动恢复，失败后资源可用于其他任务。
- 持久化 interrupted 不自动释放执行资源，必须显式确认停止。其他任务快照保持不变。

## 限制

合成文件身份及模拟停止确认，没有真实文件/桌面/浏览器动作。写 running 表示已提交启动状态，不证明动作已执行；若之后崩溃仍依赖宿主恢复为 interrupted。宿主必须在实际派发前再次处理接管、取消和截止时间，不能把返回凭证当作长期执行授权。

尚未接入任务空间、真实执行器、单实例宿主或 Windows 原生验证。本轮未更改用户数据库或密钥配置；密钥工作和环绕菜单实现继续暂停。
