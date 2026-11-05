# CU-S4 Windows 补丁构建证据（2026-11-05）

## 范围

仅证明固定 `agent-desktop` revision 加唯一补丁后的 CU-S4 Spike 能在 Windows x86_64/MSVC 构建，并通过当前契约测试。原生 Backend 仍拒绝动作；本记录不证明 CUA 原生样本、用户输入抢占、干净机分发或 macOS 可用。

## 来源与环境

| 项目 | 值 |
|---|---|
| 上游 revision | `d3dc190afbf34cd2377252398ddd5c30efbd093a` |
| 补丁 SHA-256 | `3f625a14254d66126f931cb479fb67b698e0ea2a5a2e4a8caad1f8f9e3149023` |
| 补丁后源码树 SHA-256 | `78f37b45808177601ea6f5b3af1fcd9095ead4ecb9dcd405c3b7abe0e20f9872` |
| Spike Cargo.lock SHA-256 | `696f2f8d29649c74ab940390f94f06fced39675bf347087d22289c862f3d4268` |
| Toolchain | `rustc 1.89.0 (29483883e 2025-08-04)` |
| Target | `x86_64-pc-windows-msvc` |
| CRT 配置 | `-C target-feature=+crt-static`；尚未做干净机运行库检查 |
| Worker 开发构建 SHA-256 | `58246f3edc0c7eda03ffbb881d134dec14f7c36e0ca818f4df1f6990fe23e42d` |

## 执行结果

- `prepare_upstream.py --source <固定上游本地 checkout>`：通过；已有目录再次校验得到相同 revision、补丁和源码树摘要。
- `cargo.exe +1.89.0 test --manifest-path spikes/agent-desktop-driver/Cargo.toml --locked --all-targets --offline --quiet`：19 个单元测试、1 个 Windows containment 测试、5 个 Worker 合约测试全部通过。
- `cargo.exe +1.89.0 clippy --manifest-path spikes/agent-desktop-driver/Cargo.toml --locked --all-targets --offline -- -D warnings`：通过；Cargo 解析到隔离目录中的补丁版 core 与 Windows adapter。
- `cargo.exe +1.89.0 build --manifest-path spikes/agent-desktop-driver/Cargo.toml --locked --bins --offline`：通过，生成 Worker 开发构建。
- `generate-schema --check`：构建清单及证据 fixture 校验通过。

固定工具链此前安装不完整，官方分发站连接重置；使用 [Rustup 支持的 `RUSTUP_DIST_SERVER`](https://rust-lang.github.io/rustup/environment-variables.html) 和 [RsProxy 镜像](https://rsproxy.cn/) 补装后通过上述验证。镜像仅用于构建环境，不进入 Worker 或产品运行时。

## 未通过门禁

尚无 Windows 原生动作/Observe 样本、macOS 原生样本、静态 CRT 干净机验证、签名与分发证据；不得据此把 `agent-desktop` 判为候选通过或替换 trycua。
