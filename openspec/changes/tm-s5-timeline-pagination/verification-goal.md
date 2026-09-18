# 独立 Verification Goal：既有任务时间线分页

日期：2026-09-18  
关联 Story：TM-S5  
关联 Change：`tm-s5-timeline-pagination`  
结论：macOS 子范围 PASS；Windows 按用户要求暂缓，不 Archive。

## 验证结果

- 页面初始使用 `after_sequence=0, limit=20`，未完整时提供“加载更多时间线”。
- ego-browser TaskSpace 71 验证后续页失败保留已有 3 条并显示重试；恢复后精确请求 `after_sequence=3, limit=20`，追加第 4 条且移除完成后的继续入口。
- 既有 Adapter/Application 回归验证 `task.events` 使用排他序号游标、按序返回且受 1..100 限额和授权约束；本次无协议或 SQLite 变更。
- 正式 macOS 应用 PID 57087 使用真实任务“给宫健的分身发送 hi”（快照序号 54）：首批显示 `#1..#20`，点击后显示至 `#40`，仍提供继续加载，未生成测试任务或修改历史。

## 证据

- `apps/desktop/evidence/timeline-pagination-20260918/browser-result.json`
- `apps/desktop/evidence/timeline-pagination-native-20260918/result.json`
- `apps/desktop/evidence/timeline-pagination-native-20260918/native-after-load.png`

## 自动检查

- `cargo test --offline --locked -p yonder-protocol -p yonder-application -p yonder-adapters -p yonder-desktop`：48 项通过。
- `node --check`：产品页面与 ego-browser 回归脚本通过。
- `scripts/check_architecture.py`、15 项门禁单测与 `git diff --check` 通过。

已知 `ts-rs` 的 `deny_unknown_fields` 解析警告为既有生成器警告。未来产物、附件或可变正文仍须实现编码后 256 KiB 响应预算；本结论只覆盖既有有界事件字段。Windows 证据依用户决定继续暂缓。
