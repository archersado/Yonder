当前归属 Story：TM-S1；规划：`docs/specs/epic-TM/story-TM-S1/README.md`。旧编号保留历史追溯。

# Verification Goal：OCT-S1

状态：未通过，Story 仍在实施。

2026-09-11 任务归属/授权增量见 `verification-task-ownership.md`：可信 AuthContext、归属过滤、统一错误和 v1 文件保护，macOS Workspace 9 项测试通过。实际认证、Gateway、旧库迁移和桌面 E2E 仍待完成。

2026-09-11 进程内 task.list 增量见 `verification-task-list.md`：真实 SQLCipher 跨连接分页与状态筛选、协议生成和 macOS Workspace 8 项测试通过。尚无任务归属/认证、并行执行或桌面总览，不据此通过整项 Story。

## 多任务增量验证目标（待执行）

2026-09-11 关联 AD-OCT-03、Story AC 6–8 和 task-status 的多任务场景。task.list 进程内增量已有独立验证，授权、执行器和 UI 仍未交付。

- 两个不同资源后台任务同时 running，独立更新序号；取消一个不影响另一个（取消操作在 OCT-S2 验证）。
- 两个 CUA 任务只有一个持有前台租约，同文件写入串行；等待原因真实且可见。
- 超过一页的授权非终态任务均可访问，新增/终态变化刷新正确；伪造身份不能查询他人任务。
- 未选中/未加载页的 running 任务阻止小龙隐藏，最后一个结束后重新计时三分钟。
- 多任务重启恢复全部 running 为 interrupted，不重放动作。
- Windows/macOS 原生任务总览与交互分别提供证据。所有依赖门禁通过前不 Archive。

## CI 增量验证入口

2026-09-11 独立记录：`verification-ci.md`。本地 macOS 真实依赖检查、6 个 Rust 测试、协议生成检查及 11 个 Python 门禁测试通过。Windows/macOS GitHub 工作流尚未实际运行；本记录先前的“CI 尚未接线”描述对应只读协议批次，现已补充配置，但不得据此认定 CI 已通过或 Story Done。

当前证据：`cargo test --workspace --offline` 通过 Domain 生命周期测试与 Application 故障注入测试。测试验证旧序号拒绝、提交失败错误传播、终态保护及排他游标分页。

SQLCipher Adapter 已加入真实临时加密数据库测试：两个连接读取同一版本后顺序竞争提交，旧序号返回 Conflict；Outbox 触发器故障使三表整体回滚；重新打开持久化状态正确，错误密钥和未知 schema_version 被拒绝。不是多线程压力测试。

## 2026-09-11 启动恢复验证

- Linux/WSL：`cargo test --workspace --offline --locked`，4 个测试通过，0 失败；Adapter 测试耗时 0.66 秒。
- Windows 原生 MSVC：`cargo test --workspace --release --offline --locked --target-dir C:\Users\gongjian\AppData\Local\Yonder\encrypted-storage-spike\target`，构建 2.91 秒，4 个测试通过，0 失败；Adapter 测试耗时 1.28 秒。复用已有工具链及缓存，无安装操作。
- 新恢复测试：关闭并重新打开加密数据库不会自行中断任务；显式恢复每批有界，拒绝 0/101；仅 running 转 interrupted，保留其他全部状态；第二个任务 Outbox 故障后该任务回滚、第一个已提交任务保留；再次恢复处理剩余任务；重开后重复恢复返回 0，不增加事件。每个恢复任务最终只有序号 3 的一条中断事件及对应 Outbox。
- 错误密钥测试中的 SQLCipher HMAC 错误日志为预期拒绝，不是测试失败；测试只处理合成数据。
- 以上是原生库层测试，不是生产宿主重启或 Agent/桌面 E2E。未实现单实例启动接线，不能据此宣称应用已经具有重启恢复能力。macOS 未验证。

完成门禁仍要求生产 Credential Manager 接线、协议校验、Gateway/CLI 实连、桌面同源状态展示及 Windows 原生 E2E。以上完成后独立复核，才能 Archive。

## 2026-09-11 只读协议验证

- 关联 AD-OCT-02；Rust 生成 request/response Schema 和 TypeScript。请求 Schema 使用反序列化语义，响应 Schema 使用序列化语义（错误响应 id 必须输出，包括 null）。
- Linux/WSL 最终 `cargo test --workspace --offline --locked`：6 个测试全部通过。真实 SQLCipher 查询验证 task.get 返回持久化 running/序号 2，task.events 排他分页返回序号 2，过期请求返回 -32001 且保留请求 id。
- Windows 原生最终 `cargo test -p yonder-protocol -p yonder-application --release --offline --locked`（附临时 vendor 配置和现有 target-dir）：3 个测试全部通过，构建 5.99 秒，协议测试 0.03 秒。未在本批重新运行 Windows Adapter 新增查询集成测试，不将其算作 Windows E2E。
- 覆盖缺失元数据、未知字段、非法 ID、过期/超安全整数 deadline、分页边界、超长请求、非规范序号、批量请求拒绝；9007199254740993 字符串往返无精度损失。
- `generate --check` 和自动生成一致性测试通过；Node 内置 TypeScript 语法剥离检查通过（不是 tsc 类型检查）。依赖方向人工核对符合围栏，CI 尚未接线。
- ts-rs 对枚举 deny_unknown_fields 输出已知忽略警告，未屏蔽；Rust 拒绝未知字段测试通过。错误密钥测试的 SQLCipher HMAC 错误仍为预期。
- 状态：只读契约增量验证通过，Story Goal 仍未通过；无实际 IPC、Gateway 身份绑定、hello 协商或桌面任务展示。生成物不是已对外发布的协议版本。
