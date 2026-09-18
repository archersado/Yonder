当前归属 Story：TM-S1；规划：`docs/specs/epic-TM/story-TM-S1/README.md`。旧编号保留历史追溯。

# 独立 Verification Goal：OCT-S1 确认停止后结束

日期：2026-09-11。关联 OCT-S1、AD-OCT-06 结束补充、task-status 的确认停止后提交终态并释放场景。Architecture Impact：conforming。

状态：macOS 核心验证通过，Story 未完成，不 Archive。

## 实测

`cargo test --workspace --offline --locked`：15 项通过，0 失败；构建 1.57 秒。新增 SQLCipher 双连接测试验证：旧序号返回原凭证；Outbox 故障回滚终态、事件和 Outbox，资源保持占用；移除故障后使用返回凭证提交状态成功，没有重放动作。完成状态与序号在第二连接可见，事件及 Outbox 各增加一条，之后才允许同资源再次准入。

另一任务的状态和桌面占用不受影响；明确 Failed 结果同样持久化后释放。完成操作的 task_id 由凭证内部绑定，调用方不能提供另一个任务 ID。未知结果没有 Outcome 枚举分支，不在本用例推断失败。

## 限制

测试以可信调用方模拟执行已停止，没有真实 Driver 停止检测。未运行 Windows、桌面或浏览器动作。任务状态提交与内存资源释放不是同一事务，顺序保守；释放失败分支返回已经提交的 Task 和释放错误，该锁故障分支本批没有注入验证。数据库失败时返回原凭证但不自动重试；调用方须观察当前状态再决策。

没有接入 UI、真实执行器、取消/暂停/恢复控制或宿主；不把核心用例通过视为 Epic/Story Done。密钥暂停，环绕菜单只保留规格。
