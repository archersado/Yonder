# Verification Goal：TM-S1 创建去重寿命与事件预算

状态：PASS（macOS/Core 子范围，2026-09-25）。Windows 按用户决定延期；完整 TM-S1 仍处于 verifying，不 Archive/Done。

## 范围

- Story：TM-S1，DEDUP-01～03、BUDGET-01，以及 TM1-AC01/03/07。
- OpenSpec：`tm-s1-dedup-lifetime`。
- 架构：AD-TM-01、AD-TM-06、AD-TM-22。
- 环境：macOS，Rust Workspace 离线锁定依赖；SQLite 使用隔离临时目录，不访问正式任务库。

## 验证矩阵

| 场景 | 证据 | 结果 |
|---|---|---|
| 首次创建 | Gateway 成功返回任务，内部创建信号为 true | PASS |
| 取消、重启、配额耗尽后同内容重投 | 返回同一 cancelled 任务、sequence=2，创建信号为 false | PASS |
| 同键不同名称 | 幂等冲突，创建信号为 false | PASS |
| 配额耗尽后新键创建 | 返回 `-32014`，不新增任务/事件/Outbox/创建映射 | PASS |
| 正式 TaskHost 展示 | 首次创建只播放一次 listening；同请求重投后保持 idle | PASS |
| 事件编码预算 | JSON 转义后超 8 KiB 拒绝；256 KiB 返回连续前缀；首项无法容纳明确失败 | PASS |
| 回归与门禁 | Workspace、Python、Architecture、生成物、67 个活动 OpenSpec 严格校验 | PASS |

结构化证据见 [`apps/desktop/evidence/tm-s1-dedup-lifetime-macos-20260925/result.json`](../../../apps/desktop/evidence/tm-s1-dedup-lifetime-macos-20260925/result.json)。该增量没有修改 React、原生窗口、Driver 或系统权限，不要求视觉截图；TaskHost 集成测试验证了实际宿主消费信号后的展示状态。

## 命令与结果

- `cargo test --offline --locked --workspace`：111 项 Rust 测试通过（Adapters 49、Application 29、CLI 5、Desktop lib 7、Desktop bin 11、Domain 1、Protocol 9）。
- `python3 -m unittest discover -s scripts -p 'test_*.py'`：33 项通过。
- `python3 scripts/check_architecture.py`：通过。
- `cargo run --offline --locked -p yonder-protocol --example generate -- --check`：通过。
- 活动 OpenSpec `--strict`：67 个通过。

## 范围限制

本 Goal 不证明 Windows 原生表现、AG-S1 云端身份、TM-S4 Resume、其他执行操作的业务幂等身份或完整 TM-S1 跨模块 AC12～16 已完成。创建重投不会生成新的协议事件，因此也不以 UI 通知替代任务事实。
