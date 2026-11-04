当前归属 Story：DS-S1；关联 `e0-validate-desktop-foundation`。日期：2026-10-22。环境：Windows 11 专业版 64 位，版本 10.0.26200。

# Windows 当前版本基础构建与启动验证

## 结论

部分通过。本轮证明当前源码可由 Windows 原生 MSVC 工具链构建和测试，普通桌面可执行文件能够启动、保持响应并创建标题为 `Yonda` 的原生窗口；不据此通过完整 DS-S1、双平台 Verification Goal 或 AD-E0-01。

## 证据

- 补齐 Tauri Windows 固定资源 `apps/desktop/icons/icon.ico`，它由仓库现有 `icon.png` 确定性生成；SHA-256 为 `742975b7b11da65588a762c083cad34faa8fa659f45c74a541dc1ad1c8b0df4a`。
- `cargo test -p yonder-desktop --locked --quiet`：库测试 7、二进制测试 4，合计 11 项通过，0 失败。
- `cargo test --workspace --locked --quiet`：各测试目标合计 73 项通过，0 失败。
- `cargo build -p yonder-desktop --locked`：成功生成 `target/debug/yonder-desktop.exe`；OpenSSL 静态对象缺少 `ossl_static.pdb` 仅产生 LNK4099 调试符号告警，没有链接失败。
- 启动后进程 `yonder-desktop` 为响应状态，原生主窗口句柄非零，标题为 `Yonda`；该进程拥有的 TCP 连接数为 0。
- 机器可读结果见 [`result.json`](../../../apps/desktop/evidence/ds-s1-windows-20261022/result.json)。详细 Cargo 日志保存在验证机 `%TEMP%\yonder-windows-workspace-test.log` 与 `%TEMP%\yonder-windows-desktop-build.log`，不纳入仓库。

## 本轮修正

Windows 构建最初因 Tauri 所需的 `icons/icon.ico` 缺失而失败，补齐资源后通过。桌面库的三个测试最初在清理仅由 macOS 创建的 `observations` 目录时失败；清理改为目录存在时执行，并让 Gateway 测试使用当前目标平台。业务断言未放宽，产品运行逻辑未改变。

## 保留门禁

- 未取得 Windows 原生窗口、托盘、九态动画、透明边缘、尺寸和人工拖动的截图或视频证据。
- 未验证三分钟四边休眠、点击/键盘恢复、核心事件延迟、五分钟 CPU/内存和完整退出生命周期。
- Windows 产品 Named Pipe Gateway 尚未接入桌面组合根；本轮只证明桌面进程没有打开 TCP，不把独立 Named Pipe 协议样本当产品接线证据。
- 未完成正式 Release/签名安装包验证，也未进行非实现者独立复核。

因此 `tasks.md` 中 Windows 完整实机、性能、视觉/拖动及总 Verification Goal 条目继续保持未完成，Change 不 Archive。
