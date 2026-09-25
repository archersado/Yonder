# Verification Goal：DO-S2 macOS OOXML 文件 Runtime

状态：PASS（macOS 文件 Runtime 子范围，2026-09-25）。Windows 按用户决定延期；Agent Gateway、任务步骤、Observe 与完成链路尚未接线，完整 DO-S2 保持 `verifying`，不 Archive/Done。

## 范围

- Story：DO-S2，DO-FILE-01～05 与 DO2-01～06 的 macOS 文件组合子范围。
- OpenSpec：`do-s2-macos-file-runtime`。
- 架构：Accepted AD-DO-01、AD-FI-01；Application 组合 Document Port 与 File Port。
- 环境：macOS；全部文件样本位于唯一临时目录，仅使用仓库合成 DOCX/XLSX/PPTX，不访问用户文档或正式任务库。

## 验证矩阵

| 场景 | 可观察证据 | 结果 |
|---|---|---|
| 三格式读取 | File Port 返回规范路径、身份、SHA-256 与有界字节；Document Port 返回 DOCX/XLSX/PPTX 格式和语义文本，不暴露 XML | PASS |
| 默认另存 | 输出固定使用 `create-new`；三种源文件保持逐字节不变，新文件哈希与回执一致 | PASS |
| 源快照保护 | 源身份租约与共享宿主锁保持到提交；转换期间源内容变化返回 `ContentChanged`，不创建输出 | PASS |
| 目标竞争 | 已存在目标返回 `AlreadyExists`，既有目标内容不变，暂存文件清理 | PASS |
| OOXML 复验 | 暂存字节在提交前重新 inspect 且格式必须一致；无效包返回 `ValidationFailed`，不创建输出 | PASS |
| 覆盖权限 | Agent 在 File Adapter 调用前返回 `PermissionDenied`；只有可信 LocalUser 可进入覆盖用例 | PASS |
| 陈旧与锁 | expected hash 不匹配返回 `HashConflict`；外部 advisory lock 返回 `HostLocked`，原文件不变 | PASS |
| 原子覆盖 | LocalUser 以读取时身份与 SHA-256 取得独占锁并原子替换，输出仍为同格式有效 OOXML | PASS |
| 平台边界 | Windows 未实施；非 macOS File Adapter 保持 `UnsupportedPlatform`，没有旁路文件写入 | PASS（静态边界） |
| 全仓门禁 | Workspace、Python、Architecture、协议生成物及 70 个活动 OpenSpec 严格校验 | PASS |

结构化结果见 [`apps/desktop/evidence/do-s2-file-runtime-macos-20260925/result.json`](../../../apps/desktop/evidence/do-s2-file-runtime-macos-20260925/result.json)。本增量没有 UI 或新增系统权限请求，因此不生成截图；文件与 Document Driver 行为由无正文结构化测试日志覆盖。

## 命令与结果

- `cargo test --workspace --no-fail-fast`：127 项 Rust 测试通过（Adapters 63、Application 31、CLI 5、Desktop lib 7、Desktop bin 11、Domain 1、Protocol 9）。
- `python3 -m unittest discover -s scripts -p 'test_*.py' -v`：33 项通过。
- `python3 scripts/check_architecture.py`：通过。
- `cargo run --locked -p yonder-protocol --example generate -- --check`：通过。
- `openspec validate --all --strict`：70 个活动 Change 与 10 个 Specs，共 80 项通过。
- 四个变更 Rust 文件定向 `rustfmt --check` 与 `git diff --check`：通过。

## 范围限制

本 Goal 不证明 Agent Gateway/CLI/MCP、任务 attempt/事件、Observe、产物发布、覆盖确认 UI、复杂 OOXML 编辑、Office/WPS 私有能力或 Windows 文件路线已完成。任何提交结果不明仍返回 `unknown`，调用方不得自动重试。
