# AG-S2 生产本地登记独立 Verification Goal

日期：2026-09-21。环境：macOS，隔离HOME下的原生Yonda进程。关联[AG-S2](../../../docs/specs/epic-AG/story-AG-S2/README.md)、Accepted AD-AG-05与本Change。验证只覆盖macOS生产UDS、MCP适配和登记/查询语义；不覆盖Windows、执行、Recording、云端接入和完整Story。

## 结果

状态：macOS生产本地登记PASS。完整Story仍不Done/Archive，Windows Named Pipe和原生桌宠端到端仍保留门禁。

离线核心回归通过：protocol 6、application 13、adapters 26、CLI 1、desktop 4，共50项。`cargo fmt`未通过，原因是当前工具链未安装`rustfmt`；不安装组件，不把该项写成通过。

真实链路证据：

- 桌面进程监听`app_data_dir/agent.sock`，父目录0700、Socket0600，无TCP监听。
- MCP initialize协议版本2025-06-18；首个hello的`ag-s2-owner`绑定连接身份并创建`created`任务。
- 同Agent同幂等键同内容重复创建返回同一`task_id`，`events`仍只有1条；不同内容返回`-32009`。
- `ag-s2-other`读取或取消`ag-s2-owner`任务均返回`-32004`，等同任务不存在。
- 持有者取消任务为`cancelled`，任务、事件和创建记录保留；SQLite复核确认同一事务产物存在。

[结构化结果](../../../apps/desktop/evidence/ag-s2-production-local-20260921/result.json)、[SQLite证据](../../../apps/desktop/evidence/ag-s2-production-local-20260921/sqlite-result.json)、[权限证据](../../../apps/desktop/evidence/ag-s2-production-local-20260921/modes.txt)和无TCP监听输出[后续需人工确认](../../../apps/desktop/evidence/ag-s2-production-local-20260921/tcp-check.txt)共同构成本次证据。未记录正文、截图、输入或完整Agent Payload。
