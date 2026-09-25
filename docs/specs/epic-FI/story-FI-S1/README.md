# FI-S1 规范文件身份与受控操作

Story: FI-S1
Epic: FI
Status: implementing
OpenSpec: fi-s1-macos-file-grant-entry

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

AD-FI-01 已接受 macOS-only Runtime：临时目录样本和 WPS 真实打开/关闭 DOCX 已通过文件身份、别名归并、授权根、重复写锁、宿主锁与原子提交验证。Windows 统一样本与 Office/WPS 锁探针已就绪但按用户决定延期，产品路径保持 unavailable。Accepted AD-FI-02 已定案临时文件授权核心；Agent Gateway 仍等待原生选择器、任务接线及可信覆盖/删除确认产品闭环。

2026-09-25 Accepted AD-FI-03 已定案 macOS 原生选择器、TaskHost 任务绑定、撤权清理及协议 1.27 安全摘要；本轮以 `fi-s1-macos-file-grant-entry` 实施，不开放文件副作用，Windows 明确延期。

## OpenSpec 与验证

[Spike Change](../../../../openspec/changes/fi-s1-file-identity-spike/proposal.md)。只验证技术路线，不代替产品实施Proposal。

[Windows运行包就绪记录](../../../../openspec/changes/fi-s1-file-identity-spike/windows-readiness.md)不是Windows PASS证据。

产品 Runtime 增量：[fi-s1-macos-file-runtime](../../../../openspec/changes/fi-s1-macos-file-runtime/proposal.md)。

临时文件授权引用核心：[fi-s1-ephemeral-file-grants](../../../../openspec/changes/fi-s1-ephemeral-file-grants/proposal.md)。该增量只建立任务/Agent/用途绑定的内存授权与新目标预检；产品原生选择器、Gateway、任务事件和 Windows 仍未开放。

原生授权入口与 Gateway 可见性：[fi-s1-macos-file-grant-entry](../../../../openspec/changes/fi-s1-macos-file-grant-entry/proposal.md)。

2026-09-25：授权核心及其[独立 Verification Goal](../../../../openspec/changes/fi-s1-ephemeral-file-grants/verification-goal.md)已通过。LocalUser-only 签发、任务/Agent/用途/期限隔离、读取复用、写引用一次消费、撤销与 256 项容量门禁，以及新目标父目录身份绑定均有回归；全仓 131 项 Rust、71 个活动 OpenSpec、33 项 Python 测试和架构/协议门禁通过。完整 FI-S1 仍为 `implementing`。

2026-09-25：macOS 原生文件授权入口及其[独立 Verification Goal](../../../../openspec/changes/fi-s1-macos-file-grant-entry/verification-goal.md)已通过。协议 1.27 仅返回授权标识、用途和过期时间；TaskHost 将文件事实保留在受控层，Agent 撤权立即清空内存授权；Task Space 的四种用途与撤销反馈已由夹具验证。Windows 证据按用户决定延期，完整 FI-S1 仍为 `implementing`。
