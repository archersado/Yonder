# Windows 非 CUA 修复与复测

日期：2026-10-24

## 已通过

- Rust Workspace：75 passed，0 failed。
- 架构检查：17 passed，0 failed。
- OpenSpec：64 项中 63 项严格校验通过；唯一失败项为本轮明确排除的 CUA Change `e0-compare-cua-drivers`。
- FI-S1：硬链接、目录 Junction 等价身份、并发锁、路径逃逸、原子提交、真实 Microsoft Word 文件锁全部通过。
- AG-S1 本地 Transport：Windows Named Pipe 已进入产品 Desktop/CLI；CLI 能完成握手并到达 Agent 登记门禁；无 TCP 监听。
- AG-S5 隔离传输：Named Pipe 与本地 TLS WSS 会话绑定/输入确认通过。
- 云端 Connector Spike：可信 WSS 连续复测、Ping/Pong、重连、无效证书拒绝和帧上限通过。
- VI-S1 降级反馈：Windows 未实现原生语音时明确显示“暂不可用”，提供重试和关闭，不再卡在权限请求态。
- TM-S5：补齐撤回的终态删除 delta spec，严格校验通过。

## 尚未通过或被外部条件阻塞

- BU-S1 Windows ego-lite：安装和固定路径发现通过，Bridge 已编译并启动真实 CLI；当前 Agent 沙箱无法连接 ego_cli bootstrap，create 超时，完整 Task Space 生命周期未通过。
- DS-S1 Windows 自动停靠：直接原生命令可缩为 112×56，但 190 秒自动空闲触发仍未通过。
- EX-S1 Jev：缺少真实服务凭据/密钥，只完成本地自检。
- Windows 原生语音录制：仍受 Spike 门禁约束，本次仅修复明确失败反馈。
- AG-S5 产品输入链：仍需一个真实已登记、支持输入确认的 Agent Runtime 完成端到端验证。

未执行或修改 CUA。
