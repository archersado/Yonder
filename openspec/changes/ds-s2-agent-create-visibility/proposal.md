# Proposal：DS-S2 Agent 新建任务即时透出

## Why

真实 Local Socket Agent 成功创建任务后，Yonder 只刷新短暂 listening 状态而不打开任务空间，用户看不到刚创建的任务，违背统一任务空间“真实任务可见”的目标。

## What Changes

- 成功的新建任务触发当前 pet 旁任务空间自动展开与列表刷新。
- 复用现有 `accepted_create → listening_pending` 派生信号和既有 task-space 打开事件。
- 幂等重试、拒绝、失败及显示失败不影响创建语义；不新增协议、SQLite 或状态所有者。

## Impact

影响 `apps/desktop` 的 Local Socket 呈现协调与任务空间定位复用；遵循 DS-S2、AD-DS-02 的展示边界。macOS 验证；Windows 暂缓。
