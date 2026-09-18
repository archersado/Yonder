# TM-S6 独立 Verification Goal

Story: TM-S6
OpenSpec: tm-s6-running-state-query
日期：2026-09-12
依据：AD-TM-02-RUNNING-STATE-QUERY.md
状态：核心检查通过；平台与 PR 验证未完成，不 Archive。

## 目标与证据

实施完成后单独核对 TM-S6 AC1–4，不修改实现。环境为本地 macOS，合成 SQLCipher v2 数据库及测试子密钥，不使用用户数据库或系统凭据。

命令：`/Users/archersado/.cargo/bin/cargo test --workspace --offline --locked`，退出 0，共 16 项通过（Adapter 7、Application 4、Domain 1、Protocol 4）。新增 `global_running_state_ignores_pages_and_owners_and_fails_unknown` 验证：

- AC1：空库无运行任务；105 条普通任务之后的其他 Agent 运行任务仍返回 Running，首页 100 条均无 running。
- AC2：双连接读到已提交完成状态，重开连接保留 running 事实，显式恢复后返回 NoRunningTask。
- AC3：测试库临时改表名使真实查询失败，返回 Unknown；恢复后读取新提交 Running，不使用旧值。
- AC4：三次查询前后 sequence 总和、events 与 Outbox 数量不变；代码审阅确认调用 running(1)，查询只走现有 SELECT。

`python3 -m unittest discover -s scripts -p 'test_*.py'`：15 项通过。架构依赖及 21 个 Story 规划门禁通过。生成协议一致性测试通过；既有 ts-rs deny_unknown_fields 解析警告仍存在，未更改协议模型。

## 限制与后续门禁

仅 macOS 核心库验证，没有 Windows 运行或原生 UI 证据。无实际宿主消费者、无隐藏/新任务准入同步保证、无执行器停止证明；NoRunningTask 不是空闲许可。TM-S1 AC11 仍未整体关闭。尚未提交 PR，当前混合工作区需在交付时隔离本 Story 差异，完成平台检查与审阅后才能 Archive/Done。

续作 AC5–8 及最新 18 项核心回归见 [执行占用汇总验证](verification-activity.md)，本记录保留首次数据库查询验证范围。

AC9–11 最新结果见 [收起预约验证](verification-rest-reservation.md)，共 20 项核心回归通过，仍未完成原生接线。
