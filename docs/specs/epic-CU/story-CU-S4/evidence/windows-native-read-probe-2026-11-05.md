# CU-S4 Windows 原生只读探针证据（2026-11-05）

## 目的与边界

验证固定上游 revision 加唯一补丁后，Windows adapter 的公开窗口枚举与 snapshot API 能在真实桌面进程上工作。探针只按明确 PID 查找唯一窗口，不发送输入、不打开截图、不输出窗口正文或完整树。此证据不代表 `snapshot → dispatch → Observe` 样本通过。

## 环境与步骤

- Windows x86_64/MSVC，`rustc 1.89.0`，源码与补丁摘要见[补丁构建证据](windows-patched-build-2026-11-05.md)。
- 启动仅含一个按钮的临时 WinForms 测试窗口；记录测试进程 PID `49992`，结束测试后停止该进程。
- 探针调用 `list_windows`，按 PID 要求恰好一个原生窗口；以精确窗口 ID 调用上游 `snapshot::run_with_context`，在隔离私有目录保存 RefStore，比较前后 PID 和窗口 ID。输出仅含 SHA-256、计数及布尔值。
- 首次调用 `snapshot::build` 返回 `adapter/INTERNAL`；受控诊断定位为 UI Automation 有界客户端创建失败。上游公开 API 要求调用方先初始化 COM/DPI。增加 `ensure_owned_process_mta_and_dpi` 后同一测试窗口成功；原生 Worker 的 Backend 初始化也已按该顺序接线。
- 固定工具链重新构建 `worker.exe` 后，以真实原生 Backend、私有状态目录、trace/screenshot 关闭和空 stdin 启动，进程以 0 退出。此检查只证明初始化与无请求退出，不证明动作执行。

## 成功结果

```json
{
  "schema_version": "yonder-agent-desktop-native-read-probe/v1",
  "platform": "windows",
  "pid": 49992,
  "window_id_sha256": "3b3923b6f5f3de13b113e84e603b5cee07fe1e80d239b145df1f186df373d3be",
  "snapshot_window_id_sha256": "3b3923b6f5f3de13b113e84e603b5cee07fe1e80d239b145df1f186df373d3be",
  "process_instance_sha256": "f860f78a095c7ab54e19027d30b26b662bf7f189a417d6c842c2cf87fd0062e5",
  "ref_count": 6,
  "nodes_observed": 8,
  "complete": true,
  "identity_unproven": false
}
```

随后在固定 `AutomationId = YonderProbeButton` 的新测试窗口（PID `48068`）上运行带 RefStore 的探针。`snapshot::run_with_context` 返回完整快照，固定 ID 匹配 1 个元素及 1 个限定引用，引用带当前 `snapshot_id` 前缀；日志仅记录其 SHA-256：

```json
{
  "pid": 48068,
  "ref_count": 6,
  "nodes_observed": 8,
  "complete": true,
  "identity_unproven": false,
  "selector_matches": 1,
  "selector_refs": 1,
  "selected_qualified_ref_sha256": "8947241262ceec2c34cac926eee3d0a4160e04c2eda48f3133446e6f31e2df9d",
  "snapshot_id_sha256": "9fda39d48ed3709bb8778a3b29a6d900f27682b92295bc2216c527c798ceb52a"
}
```

同一窗口放置两个相同 `AutomationId` 的按钮（PID `60156`）后，探针以 `fixture target is not unique` 拒绝，退出码 1。两个临时测试进程均已停止。此判据仅证明上游快照能生成唯一限定引用，尚未证明 Host 独立 oracle 与 Worker 派发前的冻结身份一致。

额外检查快照根节点的原生身份（新受控窗口 PID `42832`）：`root_native_id_kind = null`、`root_native_id_sha256 = null`；快照仍完整、固定按钮仍唯一。原生窗口 ID 仅为 HWND 编码，进程令牌仅来自进程启动时间。该组合无法单独证明同进程窗口对象的代际，因此正式 Host `window_token_sha256` 尚无合格来源，原生派发继续关闭。此窗口也已停止。

## 后续门禁

当前 bridge 仍未把 Host 的冻结身份独立绑定到唯一元素引用，也未实现原生动作、动作后 Observe、用户输入 observer 或 macOS 原生验证。因此 `NativeBackend::dispatch` 继续返回 `unsupported/not_delivered`，不得把本探针结果标为候选通过。
