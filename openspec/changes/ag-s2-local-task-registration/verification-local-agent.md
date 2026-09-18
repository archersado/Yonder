# AG-S2 本地 Agent 登记独立 Verification Goal

日期：2026-09-14。环境：macOS，本机离线锁定依赖构建。关联[AG-S2](../../../docs/specs/epic-AG/story-AG-S2/README.md)、本Change delta spec及Accepted AD-AG-01/02、AD-ST-01。验证目标独立于实施任务；本阶段验证只读取结果，不修改实现。

状态：首批登记核心及私有stdio联调PASS。完整Story不Done/Archive，生产认证、正式桌面IPC接入和Windows尚未通过。

| 验收映射 | 本次证据 | 结论与限制 |
|---|---|---|
| AC1 | 实际SQLite会话拒绝未握手、过期、伪造身份、1.0、LocalUser、非法字段/能力；拒绝项不产生创建记录 | 门禁核心PASS；生产凭据认证未接 |
| AC2 | 独立Python测试Agent通过父子进程私有stdio发hello/create；正式TaskHost绑定local-test-agent，数据库生成任务ID | 测试连接PASS；不是当前小龙进程连接 |
| AC3 | 同Agent同key同说明返回同ID，修改说明-32009；重启重开库后仍返回当前running序号2，未重置 | PASS |
| AC4 | 实际文件schema2→3保留旧任务；任务/事件/Outbox/映射同事务；Outbox触发失败四表计数不变 | PASS |
| AC5 | 两Agent同key取得不同ID，各仅读所属，跨Agent get-32004；本地stdio同Agent登记两个任务并查询 | 归属与并列登记PASS；原生面板显示两个新任务未测 |
| AC6 | stdio新登记任务均created，未伪造running或执行description；既有宿主展示状态测试通过 | 核心PASS；实际执行器联动未测 |
| AC7 | LocalUser创建拒绝，直接query拒绝task.create；仅新增examples测试宿主，无产品人工创建入口 | 创建禁区PASS；已有任务接管另属TM/RC设计 |

运行命令：

```bash
cargo run --offline --locked -p yonder-protocol --example generate
cargo test --offline --locked -p yonder-protocol -p yonder-application -p yonder-adapters -p yonder-desktop --lib
cargo test --offline --locked -p yonder-domain --lib
cargo build --offline --locked -p yonder-desktop --example local-agent-gateway --bin yonder-desktop
python3 apps/desktop/check-local-agent.py apps/desktop/evidence/local-agent-registration-20260914/result.json
PATH=/Users/archersado/.cargo/bin:$PATH python3 scripts/check_architecture.py
git diff --check
```

结果：Protocol4 + Application6 + Adapter11 + Desktop3 + Domain1 = 25项通过；正式桌面二进制及测试宿主构建通过。生成协议检查包含在Protocol回归内。架构与文档关联通过不等于完整Story通过。既有ts-rs未知serde属性警告保留，Rust与Schema输入保护未关闭。

[本地Agent结构化证据](../../../apps/desktop/evidence/local-agent-registration-20260914/result.json)：实际任务2个、幂等重放与冲突拒绝、四表各2条；product_data_modified=false、live_desktop_connection=false。测试库自动清理，不保留任务说明、请求正文或凭据日志。

后续需定稿AG-S1生产身份绑定与正式宿主UDS/Named Pipe接线，再将真实任务用于原生面板/状态验证；不得把预绑定测试身份当生产认证。Windows按用户指示暂缓，保留双平台门禁。未实施执行器、Recording或向外发送用户记录。
