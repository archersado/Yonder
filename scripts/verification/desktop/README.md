# Desktop 验证脚本

本目录统一保存 Yonder Desktop 的隔离验证、原生 macOS 检查和前端夹具。脚本文件名保持历史 `check-*` 约定；产品源码、Tauri 配置、fixture 与证据仍分别位于 `apps/desktop/ui`、`apps/desktop/src`、`apps/desktop/tests` 和 `apps/desktop/evidence`。

所有命令从仓库根目录运行，例如：

```bash
node scripts/verification/desktop/check-task-space.mjs
python3 scripts/verification/desktop/check-sky-cua-worker.py
swiftc -parse scripts/verification/desktop/check-region-preview-macos.swift
```

需要真实应用、账户、辅助功能权限或麦克风权限的脚本不会被通用 CI 自动执行；对应 Story 的独立 Verification Goal 必须记录运行环境和结构化证据。
