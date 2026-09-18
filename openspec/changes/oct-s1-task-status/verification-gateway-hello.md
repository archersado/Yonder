当前归属 Story：TM-S1；规划：`docs/specs/epic-TM/story-TM-S1/README.md`。旧编号保留历史追溯。

# 独立 Verification Goal：OCT-S1 Gateway 握手

日期：2026-09-11。关联 OCT-S1、AD-OCT-05、task-status 的 Gateway 握手场景。Architecture Impact：architecture-change。

状态：macOS 核心验证通过；OCT-S1 未通过，不 Archive。

## 实现与验证目标

GatewaySession 固定可信 AuthContext 和平台，每连接独立协商候选协议 1.0。未握手不得查询；主版本不兼容清除握手状态；能力清单仅返回 task.read。Rust 生成请求/响应 Schema 和 TypeScript，无新增依赖。

## 实测结果

- `cargo test --workspace --offline --locked`：11 项通过，0 失败；构建 0.41 秒，四项 SQLCipher Adapter 测试 0.31 秒。第一次验证发现测试请求缺少字符串结束引号，修正合成样本后重跑全量通过。
- 无存储测试替身的所有方法均 panic：三个查询在握手前返回 -32002；hello 成功、重复 hello、过期、身份不符和主版本不兼容均不访问存储。
- 客户端 1.99 协商至 1.0，重复 hello 响应相同；macOS/Windows 平台字段按构造参数返回。独立新会话仍须握手；一个会话协商失败不影响另一个。
- 已握手会话收到主版本 2 后返回 -32010，之后三个查询重新被门禁拒绝；身份冒用返回 -32003；无效 JSON 返回 -32700 且 id=null。
- 真实临时 SQLCipher 库通过 GatewaySession 执行 list/get/events：列表只返回本 Agent，自己的任务可查，他人和不存在任务均返回 -32004。
- 版本缺失、负数、溢出及未知字段被 Rust 解码拒绝；派生协议漂移测试通过。保留 ts-rs 对 deny_unknown_fields 的既有警告，Rust 字段校验仍生效。
- `PATH=/Users/archersado/.cargo/bin:$PATH python3 scripts/check_architecture.py` 通过。

## 限制

这是进程内会话逻辑，尚无真实认证、连接或宿主接线。平台字段测试不是 Windows 原生执行证据。本轮未更改桌宠、读取用户数据库或启动 Worker；未完成 Windows/macOS 原生 E2E。生产入口必须先认证并使用 GatewaySession，不能根据 JSON 自行构造 AuthContext。可信内部 query::handle 不承担连接握手，其 hello 返回 -32002。

桌面与 IPC 产品实施仍受 E0-01 门禁约束；完整步骤/观察/意图协议、Credential Store、任务空间及资源并发控制继续待办。
