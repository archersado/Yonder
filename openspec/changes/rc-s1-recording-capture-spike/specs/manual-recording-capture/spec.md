# 手动 Recording 采集 Spike Delta

## ADDED Requirements

### Requirement: 默认关闭与显式边界
探针 MUST 默认不采集；只有测试操作者显式开始后接收样本，显式停止后 MUST 不再增加事件。

### Requirement: 受控会话分类
探针 MUST 只在显式用户控制租约内产生 `controlled_session_input`；已知 `agent_cua/replay` 注入和租约外事件 MUST 为 `external_unknown`。`controlled_session_input` 不得声称是物理用户证明，也不得成为自动回放输入。

### Requirement: 隐私排除
Secure Text Field、Secure Event Input、系统安全界面和排除应用 MUST NOT 产生有效动作；证据只记录稳定排除原因。

### Requirement: 有界与缺口可见
固定容量队列溢出、Tap失效或权限撤销 MUST 形成 gap/失败分类，不得静默丢失或自动重启采集。

### Requirement: 无正文证据
探针输出 MUST NOT 包含按键正文、截图、AX文本、坐标或完整原生事件Payload。
