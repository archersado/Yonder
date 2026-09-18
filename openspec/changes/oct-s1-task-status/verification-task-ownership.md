当前归属 Story：TM-S1；规划：`docs/specs/epic-TM/story-TM-S1/README.md`。旧编号保留历史追溯。

# 独立 Verification Goal：OCT-S1 任务归属与查询授权

日期：2026-09-11。关联 OCT-S1、AD-OCT-04、task-status 的任务归属与可信身份场景。Architecture Impact：architecture-change。

状态：macOS 核心验证通过，OCT-S1 仍未通过，不 Archive。

## 实现

Application 创建任务及只读分派显式接收 AuthContext。身份类型不实现 JSON 反序列化；真实组合根负责先认证。Agent 只能读取自己的任务，LocalUser 可读取本用户库全部任务，JSON 中 agent_id 必须匹配绑定身份。task.get 与 task.events 共用 readable 校验；task.list 在 SQL 中先按归属过滤，再筛选和分页。

新建库为 schema_version=2，tasks.owner_agent_id 必需并由创建上下文写入；状态迁移和重启恢复保持归属。v1/未知版本拒绝打开，未实施自动迁移。快照新增 owner_agent_id，Schema/TypeScript 从 Rust 重新生成。

## 实测

- `cargo test --workspace --offline --locked`：9 项通过，0 失败；构建 1.00 秒，Adapter 四项测试 0.30 秒。生成漂移检查包含在协议测试中通过，ts-rs 既有 deny_unknown_fields 警告保留。
- 真实 SQLCipher 合成库：两个 Agent 各创建两条任务；Agent 列表不会混入他人任务，即使他人的 ID 位于首项和末项，分页游标仍只基于本人的结果。
- Agent 的 get/events：自己的任务成功，他人任务和不存在任务均返回 -32004。三个只读方法在绑定身份与请求 agent_id 不符时均返回 -32003。
- LocalUser 全量列表覆盖四个任务，分派使用同一请求文本时，返回范围由传入的可信上下文决定；身份不是从 JSON 推导。
- 状态迁移及 running→interrupted 恢复后归属保持；重新打开数据库仍保持。
- 将合成测试库转为没有 owner_agent_id 的真实 v1 布局，打开被拒绝，前后数据库文件字节完全一致。测试未打开用户实际数据库。
- 现有 105 任务分页、加密/错误密钥、旧序号拒绝、Outbox 原子回滚及恢复回归均通过。
- 真实 Cargo 依赖检查与 `git diff --check` 通过，无新增依赖。

## 限制

AuthContext 是可信内部输入，不是认证实现。尚无 OS peer 身份、Agent 凭据、云端身份校验或 Gateway 实连；禁止由未认证传输根据请求自行构造 AuthContext，禁止开放任意 LocalUser 升权入口。

这是破坏旧库直接打开兼容性的开发增量。v1 不会被自动升级；部署前必须另建迁移 Change，备份并确定旧任务归属。本轮只在临时合成库验证，运行中的桌宠及用户文件未改变。

Windows 本轮未运行；任务空间、并行执行器、真实认证和原生 E2E 均未完成，不将核心授权测试等同产品安全验收。
