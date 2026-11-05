# CU-S4 架构设计

## 边界与依赖

Spike 采用“`agent-desktop` 引擎 + Yonder 监管包装层”。探针模拟既有 Application ComputerUse Port；Adapter 内部启动一个受监管子进程，子进程运行固定 revision 的最小 Rust bridge，通过 inherited stdio JSONL 保留完整 disposition。不得使用 status-only C ABI/CLI 或其他 IPC，不得让探针、Driver 或 Worker 修改产品 SQLite、任务事件、Outbox 或 Gateway。

```text
Spike Harness（模拟 Application 语义）
  ├─ Task-level lease / fencing / input generation / cancel / deadline
  ├─ ComputerUse Port-shaped boundary
  └─ Supervisor ── inherited stdio JSONL ── Worker / minimal Rust bridge
                                              └─ agent-desktop core
                                                     ├─ Windows UIA
                                                     └─ macOS AX
```

上游 interaction lease 只串行化参与该机制的一次 dispatch。Yonder 的任务级 lease/fencing 仍是唯一所有权来源。上游 snapshot ref、命令和 disposition 不进入 Domain，也不手写为第二份 Rust/TypeScript 产品协议。

Worker 对可解析但被原生 Backend 拒绝的 snapshot 返回带错误分类的 `snapshot_result`，其中 `snapshot_id` 和 `qualified_refs` 为空，随后终止当前会话；Host 不得因此进入派发状态。只有 `status = ok`、目标身份匹配且恰好一个限定引用时才能继续。Backend 声称成功却缺少这些条件时视为协议故障并终止。

原生 Worker 初始化先于任何平台 API：Windows 使用上游公开 `ensure_owned_process_mta_and_dpi` 并安装 Windows 私有文件操作；macOS 使用上游公开 `ensure_cocoa_multithreaded`。初始化失败即拒绝启动原生 Backend。只读诊断探针可在隔离 fixture 上验证窗口枚举与 snapshot，但其结果不等于动作、抢占或 Observe 闭环通过；诊断输出只含计数、布尔状态和身份摘要。

只读 fixture 的元素探针只接受预置 `AutomationId = YonderProbeButton`，要求完整快照中恰好一个带 ref 的匹配元素；不得以可访问名称、标题或树中首项回退。这个诊断判据不能替代正式样本的 Host 独立身份 oracle；正式样本在目标与元素选择规则冻结前继续禁止原生派发。

若探针验证上游限定引用，须在隔离私有状态目录中用上游公开 `snapshot::run_with_context` 保存 RefStore，并显式关闭 trace/screenshots。只输出限定引用格式是否正确及其摘要，不输出引用对应的树、文本或窗口内容。探针结束后目录按临时 fixture 数据处理，不得混入产品状态。

Windows 实测 `WindowInfo.id` 只编码 HWND、`process_instance` 只编码进程启动时间，快照根节点没有 `native_id`。这不足以证明同一进程内被复用的 HWND 指向原窗口。正式 `target_identity.window_token_sha256` 必须来自可区分窗口对象代际的只读证据，并由 Host 独立采样与 Worker 派发前核对；在证据与双平台等价路径确定前，现有受控 fixture 的限定引用只能作为只读能力证据，不能打开动作门禁。

固定上游源码的 `execute_by_ref` 会从 RefStore 读回 ref，再调用 `resolve_saved_ref` 重新定位窗口；Windows 正常 ref 的窗口归属检查仅比较 HWND、PID 与进程启动令牌。若同一进程复用 HWND，新窗口又具有相同元素谓词，重新定位可能接受替代目标。Host 在调用前单独核验窗口不能覆盖后续解析到实际副作用的竞态；参见[源码审计](evidence/upstream-ref-window-generation-source-audit-2026-11-05.md)。窗口代际证明必须覆盖 Worker 内的这段真实动作路径，不能仅靠 snapshot 前后两个屏障通过。

Host 的 Windows 只读探针可用 Win32 `IsWindow`、`GetWindowThreadProcessId`、`OpenProcess` 与 `GetProcessTimes` 独立核对 HWND 当前归属及进程创建时间，再与上游 `WindowInfo` 比较。它只能证明进程和句柄当前一致，不提供窗口对象代际；不得据此填入 `target_identity.window_token_sha256` 或打开派发。

Windows 窗口生命周期候选探针可在首次目标快照前启动 `SetWinEventHook`，只追踪系统产生的窗口 `CREATE/DESTROY`，在独立消息循环中维护有界事件序列。身份采样前由 Host 创建隐藏的自有窗口，等待它的系统 `CREATE` 回调作为候选事件屏障，再比较目标 HWND、进程启动令牌与这段生命周期序列。观察器未就绪、重启、事件队列溢出、屏障超时或 Host/Worker 序列不一致均拒绝。该方案目前只有受控 WinForms 可见性与顺序证据；在 HWND 复用、目标应用、故障与最终动作竞态验证前不得填充正式窗口代际令牌或开放派发。

WinEvent 零计数仅在目标确实位于观察器覆盖范围时有意义。Windows Host/Worker 只读 oracle 必须核对目标与自身会话、输入桌面及进程完整性级别；目标不可读、在其他桌面或高于当前权限时直接拒绝，不进行动作或以零事件视为稳定。当前诊断先验证相同权限的受控窗口；拒绝路径与高权限样本仍需单独证据。

Host/Worker 一致性实验可以启动两个独立的**诊断观察器进程**，在同一受控窗口上同时接收生命周期事件并比较屏障前后计数。诊断入口只输出事件计数、布尔状态和进程身份摘要，不实现动作、不替代唯一产品 Worker bridge，也不改变 OpenSpec 的 bridge/evidence 协议。

`IDENTITY-01` 正式 fixture 在同一测试进程创建两个可见顶层窗口，标题、按钮角色与按钮无敏感标签相同；每个窗口恰好一个可点击按钮。目标由 Host 启动该 fixture 后读取 fixture 自身发布的窗口角色与原生窗口 ID 来冻结，不能靠标题或前台状态选择。两个按钮使用相同的固定元素标识：Windows `AutomationId = YonderIdentity01Button`，macOS `AXIdentifier = YonderIdentity01Button`；唯一性按冻结窗口内的完整快照验证。每个窗口在创建时由 fixture 自行设置不可变、互异的窗口代际标记；Windows Host/Worker 只读窗口 property，macOS Host/Worker 只读窗口 AX identifier，二者都不得向目标写入标记。点击计数只在 fixture 内按窗口分别累加，Host 独立只读核验目标为 1、诱饵为 0 且两次稳定采样一致。该 fixture 专用标记不代表任意应用的窗口代际能力；动作准入仍由前述 Worker 路径门禁控制。

固定上游补丁扩围属于 AD-CU-07 的[待定决策](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-07-AGENT-DESKTOP-DRIVER-SPIKE.md)：v1 唯一 hit-test 补丁不能被原位修改。v2 若要成立，须在代码前明确最终交付点怎样绑定原窗口对象，以及物理注入如何绑定当前 attempt/Worker；Windows `dwExtraInfo` 与 macOS `eventSourceUserData` 只是候选事件载体，不是已通过的归属机制。UIA Invoke/Legacy 与 AX 语义动作另需原生验证。无法提出可检验的交付点机制时，停止 v2，不能仅凭 fixture 的前置身份拒绝开启动作。

允许 `IDENTITY-02` 的**只读诊断**在自建 fixture 中保存限定引用后，要求 fixture 自行销毁并重建目标窗口，再用上游公开 `resolve_saved_ref` 尝试重新定位旧引用；诊断只输出新旧窗口 ID 是否相同、代际是否变化、旧引用是否拒绝和两个点击计数，不调用 `execute_by_ref` 或原生动作。新旧 HWND 不相同时的拒绝只证明普通 stale 路径；只有完整 HWND 复用的受控样本才能检验该特定风险。此诊断不计入正式 `IDENTITY-02` 动作样本通过。

同一诊断可先保持原窗口不变，让 fixture 销毁按钮并创建同 `AutomationId`、同标签、同位置的新按钮，再对旧限定引用调用 `resolve_saved_ref`。只记录旧引用是否重新绑定、窗口代际是否不变及零点击；即使观察到重新绑定，也只是只读解析风险证据，不能通过执行动作扩大样本。按钮替换不等价于完整 HWND 复用。

## 允许集与拒绝集

允许集：snapshot、qualified ref 动作、动作前目标重识别及经样本验证的窗口操作。拒绝集：raw 坐标、Windows 无目标按键、`--force`、CDP 和上游 MCP。显式关闭默认开启的结构化 trace，保持 screenshots 禁用，并阻止 notification/snapshot/clipboard/command output 进入证据。hit-test `unknown` 在包装层转换为拒绝，不调用动作。

固定 commit `d3dc190afbf34cd2377252398ddd5c30efbd093a`（v1.1.0），不因 CRT 缺陷漂移源码。Windows 固定静态 CRT 构建策略并以干净机验证；不能成立即淘汰。首次原生运行前写入 build manifest：上游 SHA、空或唯一 patch hash、Cargo.lock hash、Rust toolchain、target triple、features、CRT/链接策略、bridge/evidence schema、Yonder harness commit/dirty diff 与制品哈希。

## 抢占与取消状态

平台观察器只产生宿主会话级 `u64 input_generation` 与 `driver_current_attempt | user | unknown | observer_lost`，不保存具体输入。只有绑定当前 attempt/Worker 的合成事件不递增 generation；其余分类，以及 observer 未就绪、重启或 generation 回退时，都递增 generation 并按 fail-closed 处理。Worker 重启不重置 generation。

派发、用户抢占和取消共用 Application-owned admission gate。严格顺序为 observer ready → task fence/lease → capture generation → snapshot/target oracle → compare control/fence/generation → 将完整动作帧写入绑定当前 attempt/Worker 的 stdio 并完成 flush。flush 是唯一线性化点；之前是 `not_dispatched`，之后是 `in_flight`。Worker 收帧后才等待上游 interaction lease，因此等待期间抢占仍按在途处理。

取消结果分为：

1. `not_dispatched`：尚未进入 Driver，可确定取消；
2. `between_steps`：上一步已 Observe、下一步未派发，可确定停止新动作；
3. `in_flight_unknown`：动作已进入 Driver，终止 Worker 后仍可能产生副作用；必须 `unknown`，禁止重试；
4. `observed`：动作返回且新鲜 Observe 验证预期条件，才是已知结果。

Windows Worker 使用 Job Object `KILL_ON_JOB_CLOSE`，macOS 使用受监管进程组并验证无脱离 helper。Worker 退出后用无副作用只读探针检查进程树、machine-wide lease、修饰键/鼠标按钮、临时文件，以及由全新实例执行的 Observe；禁止发送合成 key-up/button-up。清理失败进入 global desktop quarantine，任务仍 `unknown`；只有宿主重启或明确只读恢复程序全部通过才能重新准入 CUA。

## Observe 与结果映射

探针不能把上游动作返回直接当作成功。每个正常动作后必须请求新 snapshot，并验证样本预先冻结的 target identity oracle 与 Observe predicate。唯一 Rust bridge 必须输出完整 JSON delivery/retry disposition；丢失任一必需字段即淘汰，不允许按平台改换入口。

`delivered_unverified`、Observe 失败、Worker 崩溃/断连、deadline、身份变化及未知 hit-test 一律映射 `unknown` 或明确拒绝，不自动重发。

## 双平台统一样本

下表保留 Story 级验证范围；精确 action、predicate、oracle 与阈值以 OpenSpec [固定样本目录](../../../../openspec/changes/cu-s4-agent-desktop-driver-spike/sample-catalog.md) 为准。

| 组 | Windows | macOS | 关键判定 |
|---|---|---|---|
| 文本与元素 | Notepad + Electron fixture | TextEdit + Electron fixture | qualified ref 成功；正文不进证据 |
| 固定应用集 | WPS + 文件资源管理器 | Microsoft Office + Finder | 已安装目标必须形成 Observe 闭环；环境缺失不得判通过 |
| 身份漂移 | 同名双窗口、重启、句柄复用 | 同名双窗口、重启、AX对象替换 | 零误投递 |
| 可见性 | 前后台、最小化、多显示器 | 前后台、Space/显示器 | 不把不可见误报成功 |
| 用户抢占 | 真实鼠标/键盘与合成输入交错 | 真实事件与合成输入交错 | 用户输入后零新增派发 |
| 故障 | deadline、Worker kill、断连 | deadline、Worker kill、断连 | unknown、不重试、资源释放 |
| 显示 | 96/192 DPI、多显示器 | Retina/缩放、多显示器 | ref 语义一致；不启用 raw 坐标 |
| 权限/分发 | 普通权限、干净机、Authenticode | AX拒绝、签名/公证 | fail closed、责任链可建立 |

## 淘汰与结论

AD-CU-07 的任一硬淘汰条件命中即停止扩围。通过不能由总分抵消安全失败。Spike 结论只更新 ADR，不修改产品 Adapter；若候选双平台通过，后续另建 Story/OpenSpec Apply 并证明替换路径，不保留单平台长期双栈出口。

## 验证与证据

结构化证据必须符合 `agent-desktop-spike-evidence/v1`，包含 revision/build manifest hash、platform/arch、sample/predicate、attempt/worker/fence、generation before/after、派发边界、delivery disposition、Observe identity/predicate verdict、cleanup verdict、单调耗时和 outcome 封闭枚举。样本 predicate 与 read-only target identity oracle 在开跑前冻结；不得包含输入正文、截图、完整窗口树、应用列表或 Agent Payload。Windows/macOS 各自需要原生执行日志和独立 Verification Goal。

## 架构影响

本 Story 是 architecture-change Spike，但当前不修改运行时架构。只有后继 Accepted ADR 与 Apply Change 可以改变 CUA Driver 依赖或产品接线。
