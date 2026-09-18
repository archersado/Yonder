当前归属 Story：TM-S1；规划：`docs/specs/epic-TM/story-TM-S1/README.md`。旧编号保留历史追溯。

# 独立 Verification Goal：OCT-S1 进程内多任务列表

日期：2026-09-11。关联 OCT-S1 AC 6、AD-OCT-02 task.list 增量、AD-OCT-03 和 task-status 的进程内列表分页场景。

状态：macOS 核心增量验证通过；OCT-S1 Goal 仍未通过，不 Archive。

## 交付范围

Rust 定义 task.list 请求/响应并生成 Schema/TypeScript；Application 校验分页并计算下一游标；SQLCipher 使用绑定参数按不可变 task_id BINARY 排序读取最多 101 行。默认返回全部非终态，include_finished=true 返回全部状态。未改变 schema_version、依赖、任务状态所有者或传输。

## 验证结果

- `cargo test --workspace --offline --locked`：8 项通过，0 失败；最终构建 0.43 秒，Adapter 三项测试 0.29 秒。保留 ts-rs 已知 deny_unknown_fields 警告，Rust 输入校验未关闭。
- 105 个逆序创建的真实加密数据库任务，经两个独立连接验证：前页 100 项、末页 2 项，无重复/遗漏；终态默认排除，暂停/等待用户/中断/运行仍包含。恰好满页且无后续项时游标为 null；空库和尾后查询返回空列表。
- 查询不产生事件或 Outbox；跨连接提交的新任务在首页可见，提交的终态从非终态筛选移除；重开数据库后查询相同事实源。
- task.list 经 JSON 解码、Application 分派与真实 Adapter 返回分页结果和原请求 ID；序号仍以十进制字符串返回。
- 协议测试覆盖缺省游标/筛选、过期请求、空/非法游标、越界分页、错误布尔类型和未知参数。现有全部协议、状态迁移、事务及恢复回归通过。
- 生成漂移测试通过。TypeScript 明确生成可选 after_task_id 和 include_finished，与 Rust 可省略的输入规则一致。
- `PATH=/Users/archersado/.cargo/bin:$PATH python3 scripts/check_architecture.py` 通过；`git diff --check` 通过。

## 边界与后续

没有任务所有者字段或 AuthContext 绑定，因此这是可信本机调用方限定存储实例范围的进程内能力，不能冒充多 Agent 授权。真实 Gateway 接线前必须完成归属和授权约束。

游标分页是逐请求已提交快照，不承诺跨页事务快照；游标前的新任务及筛选状态变化需刷新首页发现。返回行数有界不代表任意历史库大小下查询成本恒定；当前复用主键索引，后续按实测再增加查询索引。

本批未实现执行器、资源准入、并行任务执行、桌面总览或 CLI 实连，也未改动正在运行的桌宠。Windows 本批未运行，不作为双平台或 UI 验收。后续先完成任务归属/认证设计与依赖 E0 门禁，再完成宿主、Gateway 和 Task Space 接线。
