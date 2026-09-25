# Verification Goal：FI-S1 macOS 原生文件授权入口

状态：通过（macOS 代码与 Task Space 夹具）；Windows 证据按用户决定延期，未计为通过。

## 目标与边界

验证 `fi-s1-macos-file-grant-entry` 在不向 WebView、Gateway 或 MCP 暴露文件位置的前提下，能够由本机用户为当前任务签发、查看和撤销短期授权；归属 Agent 被禁用时，授权必须立即失效。本 Goal 不验证后续文件副作用执行，也不替代 FI-S1 整体归档。

## 已执行证据（2026-09-25，macOS）

| 检查 | 结果 | 覆盖结论 |
| --- | --- | --- |
| `cargo test -p yonder-protocol -p yonder-application -p yonder-desktop -p yonder-cli` | 70 通过 | 协议 1.27、无路径响应、Registry 安全摘要、Gateway 归属隔离、TaskHost 撤权清理、CLI 类型请求。 |
| `cargo test -p yonder-adapters file::tests::create_replace_validation_and_race_preserve_the_committed_target -- --exact` | 1 通过 | 与文件 Runtime 的既有竞争边界兼容。 |
| `cargo test -p yonder-adapters command::tests::timeout_and_cancellation_stop_parent_and_descendant -- --exact` | 1 通过 | 与 macOS 进程组清理边界兼容。 |
| Task Space 浏览器夹具 | 通过 | 渲染四个用途按钮；读取授权后仅显示用途和过期时间；界面明确提示不显示位置，夹具结果未包含位置。 |
| `cargo run --locked -p yonder-protocol --example generate -- --check` | 通过 | TypeScript 与 JSON Schema 均由 Rust 协议源生成。 |
| `openspec validate fi-s1-macos-file-grant-entry --strict`、`python3 scripts/check_architecture.py`、`python3 scripts/test_release.py` | 通过 | Change、架构关联和发布协议契约一致。 |

浏览器夹具的结构化结果为：四个授权用途按钮可用；读取授权签发后显示“读取”和过期时间；“文件位置不会显示在此处”可见；断言未出现路径。夹具只替换 Tauri 调用返回值，不读取用户浏览上下文，也不把真实文件路径交给页面。

## 全仓回归说明

`cargo test --workspace --no-fail-fast` 在两次运行中分别暴露了未触及的既有 macOS 适配器偶发失败：文件竞争测试一次返回 `HostLocked`，命令子进程测试一次因空 PID 输出解析失败；两项各自隔离复跑均通过。定向 FI 回归及所有本 Change 改动所在包回归均通过。此不稳定性不作为本 Change 的通过依据，需在对应 Adapter Story 单独消除。

## 延期与后续

- Windows 原生选择器/确认和证据按用户指示延期，不能标记为 Windows PASS。
- 真实文件读取、创建、替换、回收站副作用仍由后续受控 File/Document 执行 Story 接入；本 Change 没有新增该类 Agent 方法。
- 完整 FI-S1 仍为 `implementing`，因此不 Archive Story；本 Goal 仅关闭本 macOS 产品入口增量。
