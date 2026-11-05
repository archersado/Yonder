# agent-desktop 隔离 Driver Spike Harness

本目录是 CU-S4 的独立 Rust workspace，不属于 Yonder 产品 Cargo workspace，也不接 Gateway、SQLite、Task/Outbox 或产品 Adapter。它只验证 `agent-desktop` 固定 revision `d3dc190afbf34cd2377252398ddd5c30efbd093a` 的 bridge 契约、监督语义与平台 containment。

## 当前边界

- inherited stdin/stdout JSONL 是唯一 Worker 通道，完整帧（含换行）上限 65,536 bytes。
- `Supervisor` 最后比较 observer、control、fence、generation、Worker 身份与冻结 action 绑定；完整动作帧 `flush` 是唯一派发线性化点。
- hello 同时固定 capability hash 与 attempt/Worker/fence/generation；action/ref 和 sample/predicate/oracle 必须完整匹配冻结目录。宿主共享 generation 上界与 quarantine，重建 Supervisor 不能绕过。
- 序列化或限长在写入前失败为 `not_dispatched`；`write_all` 或 `flush` 失败均按可能已交付的 `unknown` 处理。断连、终止、deadline 或无法 Observe 同样为 `unknown`，重试次数固定为 0。
- cleanup 只执行进程树、lease、输入状态、临时资源及新实例 Observe 的只读探针；本实现不发送合成 key-up/button-up。无法证明释放即全局 quarantine。
- Worker 要求私有绝对路径 `AGENT_DESKTOP_HOME`，并要求 `AGENT_DESKTOP_TRACE=off`、`AGENT_DESKTOP_SCREENSHOTS=off`。协议、stderr 与证据不得包含输入正文、截图、完整窗口树、应用列表或完整 Payload。
- Linux 只允许 `--mock` 契约测试，不构成原生 Driver 结论。
- mock Worker 写入的 fixture marker 只用于验证 Host 独立读取、完整 TargetIdentity 比较和稳定双采样契约；Harness 输出标记为 `contract_only_observe_protocol`，绝不构成真实桌面动作效果证据。
- Windows Worker 使用 suspended spawn，加入 `KILL_ON_JOB_CLOSE` Job 后才 resume；任一步失败都终止并 wait。macOS helper 完整进程树行为仍待原生审计，不视为已验证。

固定上游源码由 `prepare_upstream.py` 检出并应用唯一补丁，Windows/macOS 的 Cargo path dependency 均指向该隔离目录；bridge 现已包含首次 snapshot/result、qualified refs 与冻结 target identity。真实原生动作仍 fail-closed 为 `unsupported`，直到原生 runner 完成并通过双平台样本。不得用 CLI、C ABI、额外 IPC 或隐式目标选择绕过。

`patches/0001-reject-inconclusive-pointer-hit-test.patch` 针对冻结 revision 添加副作用前拒绝及回归测试。在 Windows 隔离副本中，将上游 workspace 临时缩为 `crates/core` 后，补丁版 core 的 1584 个单元测试和 2 个附加测试全部通过。Windows Spike 构建已解析到同一补丁源码；此结果尚不覆盖 Windows/macOS 原生动作。

首次构建前准备源码；可用 `--source /path/to/checkout` 从已存在的固定上游仓库离线准备。脚本校验 commit、补丁改动集和源码摘要；已有目录摘要不符时拒绝继续，不能静默改用未打补丁的上游。

```bash
python3 spikes/agent-desktop-driver/prepare_upstream.py
```

## 验证命令

```bash
python3 spikes/agent-desktop-driver/prepare_upstream.py
cargo fmt --manifest-path spikes/agent-desktop-driver/Cargo.toml -- --check
cargo test --manifest-path spikes/agent-desktop-driver/Cargo.toml --locked
cargo clippy --manifest-path spikes/agent-desktop-driver/Cargo.toml --locked --all-targets -- -D warnings
cargo run --manifest-path spikes/agent-desktop-driver/Cargo.toml --bin generate-schema --locked -- --check
npx -y @fission-ai/openspec@latest validate cu-s4-agent-desktop-driver-spike --strict --no-interactive
```

从 WSL 调用 Windows `cargo.exe` 时，Windows 子进程可能找不到 `git`。此时先用仓库根目录下
`git status --porcelain -- spikes/agent-desktop-driver | sha256sum` 取得 dirty-status 哈希，再通过
`WSLENV=YONDER_GIT_DIRTY_STATUS_SHA256` 显式传入；缺失时校验器会 fail-closed，不得跳过 provenance 校验。

首次生成 schema 使用同一命令去掉 `--check`。本地契约 harness 先构建二进制，再传入 Worker 绝对路径：

```bash
cargo build --manifest-path spikes/agent-desktop-driver/Cargo.toml --locked --bins
cargo run --manifest-path spikes/agent-desktop-driver/Cargo.toml --bin harness --locked -- /absolute/path/to/worker
```

Windows 原生只读诊断入口为 `cargo run --manifest-path spikes/agent-desktop-driver/Cargo.toml --bin native-read-probe --locked -- <测试窗口 PID>`。它要求该 PID 恰好有一个窗口，只输出身份摘要和快照计数，不派发动作；结果不替代统一样本。

## 已验证与未验证

- 已实现：协议/证据 Rust 类型及唯一生成 schema、固定 action/ref/predicate/oracle 映射、64 KiB framing、握手与会话身份、deadline、admission/flush/cancel 状态机、宿主 quarantine、mock backend、Windows suspended Job Object/macOS process group 接口及实际哈希 evidence verifier。
- 未验证：Windows/macOS 原生样本、真实用户/合成输入 observer、完整只读 cleanup probes、WPS/Office/Finder/Explorer 任务、干净机静态 CRT、SBOM/许可证、Authenticode 与 macOS 签名/公证责任链。
- 在上述双平台证据完成前，任何契约测试通过都不得表述为 `agent-desktop` 候选通过或替代 trycua。
