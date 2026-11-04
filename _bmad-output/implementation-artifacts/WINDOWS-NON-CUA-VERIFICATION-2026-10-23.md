# Windows 非 CUA 剩余验证报告

日期：2026-10-23
范围：当前仓库中可在 Windows 执行的剩余验证；按用户要求排除全部 CUA Story、Driver 和相关 Change。

## 结论

| 范围 | 结果 | 说明 |
|---|---|---|
| Rust Workspace | PASS | 73 项通过，0 失败；协议生成 `--check` 通过 |
| CM-S1 命令执行 Spike | PASS | 字面参数、Job Object 进程树终止、64 KiB 输出上限通过 |
| DO-S2 OOXML | PASS | Rust/Node 样本通过；真实 DOCX/XLSX/PPTX 经 Office 打开且无修复提示 |
| Windows Context Native Messaging | PASS | UTF-8、分片、连续帧与超限拒绝通过 |
| AG-S5 输入协议样本 | PASS/PARTIAL | 隔离 Named Pipe 与 WSS 通过；产品 AgentSession 仍缺失 |
| AG-S1 云端 Connector | FAIL | 首次通过，随后两次可信 WSS 回显超时，不可重复 |
| FI-S1 文件身份 | FAIL | 硬链接、锁、逃逸、原子提交和 Word 锁通过；符号链接权限不足 |
| VI-S1 语音 | FAIL/PARTIAL | API 能力探测通过；产品窗口卡在请求权限；未执行显式音频 |
| EX-S1 Jev | BLOCKED | 自检通过；缺少真实 `TYPESAFE_API_KEY` |
| DS-S1 桌面基础 | FAIL/PARTIAL | 贴边后窗口仍为 200×200，唤醒未恢复；5 分钟资源证据已记录 |
| AG-S1 本地 CLI/MCP | FAIL | desktop/CLI 没有 Windows Named Pipe 产品实现 |
| BU-S1/BU-S2 | BLOCKED | Windows 未安装并完成 ego-lite/ego-browser 图形化初始化 |
| CX/RC Windows 原生路径 | BLOCKED | Windows 圈选、录制/来源识别仍未实现或仅有 macOS 实现 |

## 仓库级检查

- OpenSpec：61 项通过、2 项失败。`e0-compare-cua-drivers` 属于本次明确排除范围；非 CUA 的 `tm-s5-terminal-delete` 因 Change 下没有 delta spec 而失败。
- 架构检查：15 项通过、2 项错误，均为 Windows 路径分隔符导致 OCT-S1 Story/Proposal 双向关联检查误判；未绕过或改写已归档 Change。
- `git diff --check` 在本轮证据整理后另行执行。

## 仍需外部条件或实现

1. 在 desktop 与 CLI 中实现 Windows Named Pipe 产品 Transport 后，重跑 AG-S1、AG-S5、DS-S2 与 TM E2E。
2. 安装并完成 ego-lite 图形化初始化后，执行 BU-S1/BU-S2 Browser Task Space 原生验证。
3. 提供有效 Jev 候选模型密钥后，运行 EX-S1 Windows 统一样本。
4. 在允许创建符号链接的 Windows 验证机重跑 FI-S1。
5. 实现 Windows 语音、圈选和录制 Adapter 后，再执行权限、设备切换、停止释放及双平台证据。

CUA 未运行、未修改，也未纳入以上通过率。
