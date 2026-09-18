# DS-S1 本地 IPC 拒绝路径独立 Verification Goal

Story：DS-S1；OpenSpec：e0-validate-desktop-foundation；日期：2026-09-13。

依据：DS-S1 架构设计「失败与验证」、增量规格「本地 IPC 验证」。状态：本机独立 Spike 的协议拒绝与端点清理通过；不替代全部 IPC 门禁。

## 方法与结果

macOS 26.5.1 arm64。扩展现有 check-ipc.py：在独立服务端进程中分别执行正常往返、错误版本、无效JSON；请求为固定测试样本，不保存用户内容。每例检查进程终止及端点清理。测试失败的finally仅终止自己创建的服务端，不删除未知端点。

`python3 spikes/desktop-foundation/check-ipc.py` 退出0：

- 正常请求echo匹配，服务端退出0，无TCP套接字，端点已移除；临时父目录权限0700。
- 错误版本未返回成功响应，服务端退出1，诊断含协议版本不匹配，端点已移除。
- 无效JSON未返回成功响应，服务端退出1，诊断为解析错误，端点已移除。

随后 `cargo build --release --offline --locked --manifest-path spikes/desktop-foundation/Cargo.toml` 成功，无需重编译，确认测试使用当前源码对应的已构建产物。摘要留存 `spikes/desktop-foundation/evidence/ipc-rejection-20260913.json`；二进制 SHA-256：`28f0ba406d3b4c70483f1630f71f0d05b27a0f371153b3d62503b1e16695db1e`。

## 边界

协议拒绝不是跨用户权限拒绝。目录0700是权限属性证据，不等于已由另一个用户实际尝试连接。该程序是单次请求后退出的技术Spike，退出1是本用例的预期行为，不据此规定生产Gateway的错误处理方式。

桌宠宿主尚未接入该独立IPC程序，本轮没有增加或绕过宿主协议。未测Windows（用户暂缓）、恶意大输入限额、生产Gateway身份/截止期以及跨用户访问。DS-S1/AD-E0-01仍未完成。

## 跨用户权限续验

check-ipc.py新增显式 --other-user 模式：仅尝试通过 `sudo -n -u nobody` 运行固定连接探针，不新建账户、不改端点权限。只有探针确实使用不同UID，并因EACCES/EPERM拒绝后，才允许判跨用户拒绝通过。

实际两次运行均收到 `sudo: a password is required`，退出1；没有真正运行nobody连接，跨用户验收仍未通过，也不把sudo拒绝当作Socket权限拒绝。

首次失败强制结束单次服务端后发现Socket遗留。已确认无IPC进程，将本轮遗留Socket重命名为同临时目录下 `.sock.sudo-denied-20260913` 留存。测试工具随后修正finally：只在强制结束自己创建的服务端后，核对之前捕获的dev/inode再清理自己创建的端点；不清理身份不同的端点。

第二次相同sudo失败后，正常模式立即成功运行三条用例，确认失败路径不再阻塞后续测试。此清理是测试工具行为，不冒充产品崩溃清理通过。跨用户实机验证仍需可用的既有不同用户执行权限，本轮不要求用户输入密码。
