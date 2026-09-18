# E0-S1 验证证据

## 当前开发环境

- 日期：2026-09-09
- 平台：Linux x86_64（仅用于编译自检，不替代 Windows/macOS 验证）
- Node：24.19.0
- npm：11.17.0
- Rust/Cargo：1.98.1
- WebKitGTK：2.52.6
- Ayatana AppIndicator：0.5.94

## Linux 编译与运行自检

- IPC 单元测试：1/1 通过。
- IPC 真实进程往返：`{"version":"0.1","payload":"yonder-e0"}`。
- Tauri 2.11.5 最小应用：构建成功并保持运行；透明无边框置顶窗口与 Tray 均完成初始化。
- 运行采样：CPU 约 0.7%，Debug RSS 约 190 MB。
- 图形环境出现 Mesa/EGL 软件渲染警告，应用未崩溃。

结论：Linux 开发环境的编译、Local Socket 与 GUI 启动自检通过。Debug RSS 超过 150 MB 目标，须在 Windows/macOS Release 构建中重新测量；本结果不能通过 E0-S1 门禁。

## Windows 实机验证

- Windows：11，版本 10.0.26100
- WebView2：152.0.4191.66
- Rust/Cargo：1.98.1，`x86_64-pc-windows-msvc`
- Visual Studio Build Tools：17.14.40
- MSVC：14.44.35207
- Windows SDK：10.0.26100.0
- Tauri：2.11.5

### 编译与 IPC

- IPC 单元测试：1/1 通过。
- Tauri GUI 测试构建：通过。
- Named Pipe 真实进程往返：`{"version":"0.1","payload":"windows-e0"}`，服务端退出 0。
- 发现并修正：`interprocess` 可统一传输 API，但名称必须按平台构造；Windows 使用 `GenericNamespaced`，Unix/macOS 使用 `GenericFilePath`。
- Windows Schannel 对 Rust/crates.io 出现 SNI 证书异常；验证通过官方 SHA-256 校验的 Rust 1.98.1 独立 MSI，并以临时 Cargo vendor 完成离线编译。vendor 未进入仓库。

### Release 五分钟空闲测量

- 窗口句柄建立：1192 ms
- 空闲 CPU：0.109%
- 平均 RSS：73,236,890 bytes（约 69.84 MiB）
- 峰值 RSS：73,314,304 bytes（约 69.92 MiB）
- 进程响应：是
- 测量结束后退出：已确认

结论：Windows 的编译、Named Pipe、窗口启动及资源预算通过。仍需人工确认托盘可见性与窗口透明/置顶行为，并补充 macOS 实机证据。
- rustc：1.98.1
- cargo：1.98.1
- interprocess：2.4.4

## 当前结果

- Rust 最小工具链安装成功。
- Local Socket Spike 源码和协议自检已创建。
- Cargo 已成功解析并下载锁定依赖。
- 编译阻塞：系统缺少 C linker `cc`。
- Tauri GUI 阻塞：尚未安装 WebKitGTK、AppIndicator 等官方 Linux 开发依赖。

## 恢复条件

重复执行：

```bash
cargo test --manifest-path spikes/desktop-foundation/Cargo.toml
cargo test --manifest-path spikes/desktop-foundation/src-tauri/Cargo.toml
cargo build --manifest-path spikes/desktop-foundation/src-tauri/Cargo.toml
```

Linux 结果只证明开发环境可编译；E0-S1 仍须 Windows/macOS 实机证据才能通过。

## 2026-09-11 macOS Release 编译

环境：macOS 26.5.1（25F80），Apple Silicon，Rust/Cargo 1.98.1，Xcode Command Line Tools。

- `cargo build --release --locked --manifest-path spikes/desktop-foundation/src-tauri/Cargo.toml`：通过，耗时 1 分 18 秒，使用锁定的 Tauri 2.11.5。首次离线编译缺少缓存，经用户授权下载依赖后完成。
- 桌面产物：`spikes/desktop-foundation/src-tauri/target/release/yonder-desktop-spike`，Mach-O arm64 可执行文件，约 9.5 MiB。
- SHA-256：`349c540d0c436a3bfc5c214df5c27435ae3cda737686409d1bcb18aa1eb47461`。
- `cargo build --workspace --release --locked --offline`：正式核心 Workspace 编译通过，耗时 1 分 05 秒；保留 ts-rs 既有属性解析警告。
- Tauri 构建自动生成 `src-tauri/gen/schemas/macOS-schema.json`，未手写或修改协议生成物。

本次仅编译，未启动 GUI、测量资源或验证透明/置顶/托盘/拖动；未打包 `.app` 或安装包。界面仍为 Yonder E0 占位桌宠，小龙概念图尚未接入。此结果不使 AD-E0-01 定案，也不代表双平台 E2E 通过。


## 2026-09-11 macOS 原生续验

以下更新取代上文历史“仅编译、未启动”的现状描述。透明小龙与动作已接入，已启动本机 .app；本轮加入原生托盘图标及找回/退出菜单，Release 构建和两项 Rust 测试通过。独立 UDS 双进程往返、无 TCP 套接字和服务端退出清理通过。详细结果、复现入口和未通过项见 `../../openspec/changes/e0-validate-desktop-foundation/verification-native-macos.md`；AD-E0-01 仍待定。

2026-09-12 原生 AX 菜单与窗口证据见 `evidence/native-macos-20260912/observations.json`；唤醒截图裁切未通过，详细范围见 OpenSpec `verification-tray-macos-20260912.md`。
