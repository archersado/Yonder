# AG-S2 原生桌宠面板独立 Verification Goal

日期：2026-09-21。环境：macOS，隔离HOME下的原生Yonda预览进程。关联[AG-S2](../../../docs/specs/epic-AG/story-AG-S2/README.md)、Accepted AD-AG-05与本Change。验证只覆盖macOS生产UDS登记后的原生桌宠面板表现；不覆盖Windows、真实执行器、Recording、云端接入和完整Story。

## 结果

状态：macOS原生桌宠面板端到端PASS。完整Story仍不Done/Archive，Windows Named Pipe与原生桌宠端到端保留门禁。

真实链路：

- 通过生产UDS/MCP用`local-test-agent`登记两个命名任务，均保持`created`，不伪造running。
- 悬停小龙后面板打开，切换“全部”后可见两个任务、`Agent · local-test-agent`和真实状态。
- 选择任务后展示名称和`任务 ID`；`接管`在执行前禁用，`取消任务`可用。
- 鼠标离开小龙和面板后任务面板隐藏。
- 同一SQLite复核显示两个任务均为`created→cancelled`，事件序号严格递增，未新增任务或伪造事件。

[原生截图与结构化结果](../../../apps/desktop/evidence/ag-s2-native-panel-20260921/native-1790001879/result.json)、[登记结果](../../../apps/desktop/evidence/ag-s2-native-panel-20260921/agent-result.json)、[取消结果](../../../apps/desktop/evidence/ag-s2-native-panel-20260921/cancel-result.json)和[SQLite证据](../../../apps/desktop/evidence/ag-s2-native-panel-20260921/sqlite-result.json)共同构成本次证据。未记录正文、完整命令输出、密码框、系统安全界面或Agent完整Payload。
