# Proposal：DS-S2 Agent 新建任务即时透出

## Why

真实 Local Socket Agent 成功创建任务后，Yonder 只刷新短暂 listening 状态而不打开任务空间，用户看不到刚创建的任务，违背统一任务空间“真实任务可见”的目标。

## What Changes

- 成功的新建任务触发当前 pet 旁任务空间自动展开与列表刷新。
- 复用现有 `accepted_create → listening_pending` 派生信号和既有 task-space 打开事件。
- 任务空间通过协议 1.32 `newest_first` 请求按 SQLite 创建时间倒序分页，确保新建任务位于第一页顶部。
- 幂等重试、拒绝、失败及显示失败不影响创建语义；schema 21 只增加创建时间排序事实，不新增状态所有者。
- 已有 CUA 控制条不得压住新任务卡片；任务空间与顶部控制条可同时显示，语音/圈选输入仍受保护。

## Impact

影响 `crates/protocol`、`crates/application`、`crates/adapters` 与 `apps/desktop`；遵循 DS-S2、AD-DS-02 与 Accepted AD-DS-05。macOS 验证；Windows 暂缓。
