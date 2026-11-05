# AD-CU-07 agent-desktop 受监管 Driver Spike

状态：Proposed；日期：2026-11-05。关联 CU-S4、CU-S2、TM-S3、AD-E0-02、AD-CU-02/04、AD-TM-08 与 `cu-s4-agent-desktop-driver-spike`。Architecture Impact：architecture-change（候选 CUA 引擎、Worker 监督边界与平台分发）。

## 决策

允许 `vercel-labs/agent-desktop` 以低层、模型无关 CUA Driver 候选进入五个工作日的 Windows/macOS 隔离 Spike。它只负责 snapshot、目标解析和经允许的原子动作；Yonder Application 继续独占任务状态、任务级桌面租约、派发准入、用户输入抢占、取消、deadline、每步 Observe、`unknown` 映射与恢复决策。

当前 Accepted 基线仍是 `@trycua/cua-driver@0.25.0`。Spike 通过并产生后续 Accepted/Rejected ADR 前，不替换产品依赖、不接 Agent Gateway、不修改任务协议或 SQLite、不开放用户能力，也不长期维护双执行栈。

Spike 只允许形成两种结论：双平台共享底层引擎，或整体淘汰。任一平台硬门槛失败都淘汰候选路线，不允许通过单平台采用形成长期双执行栈。

## 集成边界

- 固定 `agent-desktop v1.1.0` 对应 commit `d3dc190afbf34cd2377252398ddd5c30efbd093a`，两个平台使用同一 revision 和 patch set，禁止跟随 `main`。已知 CRT 缺陷不得通过切换到后续 commit 绕过；必须使用 Spike 的固定构建策略验证，验证不通过即判定失败。
- 唯一入口为 Worker 内最小 Rust bridge，经继承 stdio 传输版本化 JSONL 信封并保留完整 delivery/retry disposition；不得使用 status-only C ABI、CLI、其他私有 IPC 或平台各自入口。Desktop 主进程不得加载最小 Rust bridge 或候选引擎；不得调用上游 MCP，不得开放 HTTP/TCP。可选 CDP 路径必须关闭。
- 首次动作前，Host 必须通过同一 bridge `snapshot` 请求提交由只读 oracle 冻结的目标定位与身份（平台、进程、原生窗口、进程启动令牌哈希和拓扑 generation）；Worker 只能返回该目标下的 `snapshot_id` 与 opaque qualified refs。后续 `dispatch` 和 `observe` 必须引用这次 snapshot 与完全相同的冻结身份，不得按标题、前台窗口或其他隐式规则重新选目标。
- Application 只依赖既有 ComputerUse Port。上游命令名、参数模型、interaction lease、snapshot ref 和 delivery disposition 均为 Adapter 内部细节，不得成为 Domain、Gateway 或持久化模型的第二来源。
- 只允许 snapshot、qualified ref 动作和经样本验证的窗口操作。禁止 raw 坐标、Windows 无目标按键、`--force`、CDP 和上游 MCP；必须显式关闭默认开启的结构化 trace，保持 screenshots 禁用，并阻止 notification/snapshot/clipboard/command output 等敏感字段进入证据。hit-test `unknown` 必须拒绝，不能沿用 fail-open。
- 固定 revision 的原始 `execute_by_ref` 对 pointer hit-test `Unknown` 采用 fail-open。CU-S4 唯一补丁已将 `Unknown` 与不支持 hit-test 的物理派发改为副作用前拒绝；上游 core 回归测试通过，两平台构建必须解析到同一补丁源码。该修复只关闭 hit-test 缺口，不授权原生派发。
- 2026-11-05 Windows 只读证据显示：`WindowInfo.id` 是 HWND，`process_instance` 是进程启动时间令牌，快照根节点无 `native_id`。仅凭 HWND 与进程令牌不能排除同一进程内销毁并复用 HWND。Host 独立只读 oracle 与 Worker 在派发前必须比较可证明区分窗口对象代际的令牌；若该令牌在既定平台样本中不可得，则按身份门槛拒绝并淘汰候选，不得以标题、前台窗口、延时或动作后 Observe 代替。此路线尚未定案，原生 Backend 继续 `unsupported/not_delivered`。
- 固定上游源码审计进一步确认，`execute_by_ref` 会读回 RefStore 并在动作路径内重新解析 ref；Windows 窗口归属条件只含 HWND、PID 和进程启动令牌，没有窗口对象代际。即使 Host 调用前已核验身份，仍无法单凭该核验证明后续解析至副作用期间未换成同 HWND 的新窗口。须在正式样本中验证复用并覆盖 Worker 实际动作路径的代际证据；这是风险推导，尚非实测误投递。见[源码审计](../../../../docs/specs/epic-CU/story-CU-S4/evidence/upstream-ref-window-generation-source-audit-2026-11-05.md)。
- Windows 只读 `IDENTITY-02` 诊断还发现：同一窗口内销毁并重建同标识、同标签、同位置按钮后，旧限定引用通过上游公开 `resolve_saved_ref` 重新解析成功；随后销毁并重建整窗且完整 HWND 不同时，旧引用被拒绝。两阶段均零点击。前者证明谓词重绑定真实发生，尚未证明误投递；后者不覆盖完整 HWND 复用。任何 v2 设计还须说明元素对象替换时的语义边界。见[诊断证据](../../../../docs/specs/epic-CU/story-CU-S4/evidence/windows-identity02-read-only-stale-ref-2026-11-05.md)。
- 同一 fixture 的附加只读诊断显示：在按钮替换前保留的上游 `NativeHandle` 替换后读取返回 `StaleRef`，而同一旧限定引用重新解析会绑定新按钮。这支持进一步评估 Worker 内仅保存并使用原生对象、不在派发时按谓词重新选元素的 v2 候选；它不证明其他 UIA provider、macOS AX 或最终动作交付竞态，不能据此开放原生 dispatch。见[同一诊断证据](../../../../docs/specs/epic-CU/story-CU-S4/evidence/windows-identity02-read-only-stale-ref-2026-11-05.md)。
- Windows UI Automation 的 RuntimeId 也不能单独当作永久窗口代际令牌：[微软文档](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomationelement-getruntimeid)明确允许标识符随时间复用；[CompareElements](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomation-compareelements) 的结果基于运行时标识符。若采用 UIA 对象比较，必须保留原对象、验证销毁重建样本并证明派发解析不会在比较后重新定位到替代对象；否则仍拒绝。
- 受控 Windows 探针证明同权限跨进程 `SetPropW` 可在窗口上附加标记，但该方法会写入目标窗口、受 UIPI 限制，且派发前读取与实际副作用之间仍有竞态；本 Spike 不将其视为只读身份 oracle，也不据此放开动作门禁。见 [实验记录](../../../../docs/specs/epic-CU/story-CU-S4/evidence/windows-window-property-oracle-2026-11-05.md)。
- `IDENTITY-01` 的 Yonder 自建 fixture 可在创建窗口时自行设置不可变的窗口代际标记；Host/Worker 只读该标记，不跨进程写入目标。Windows 读 fixture 自有 window property，macOS 读 fixture 自有 [AppKit accessibilityIdentifier](https://developer.apple.com/documentation/AppKit/NSAccessibility-c.protocol/accessibilityIdentifier?changes=_3)；该标记仅能证明此 fixture 的窗口代际，不能外推到 WPS、资源管理器、Office 或 Finder，也不能单独关闭读后到动作交付的竞态。正式样本的动作准入仍须通过 Worker 实际 ref 解析与交付路径的身份门禁。
- 受控 Windows `SetWinEventHook` 探针在独立进程中收到了窗口销毁、创建、再销毁事件，说明只读生命周期追踪具备进一步验证价值。由于 out-of-context 回调异步，正式门禁仍要求可验证的事件屏障、观察器故障拒绝、复用样本与最终身份核对到副作用之间的竞态证明；当前尚不把事件计数当作合格窗口代际令牌。见 [实验记录](../../../../docs/specs/epic-CU/story-CU-S4/evidence/windows-winevent-lifecycle-2026-11-05.md)。
- 额外自建隐藏窗口探针在 512 次重建中观察到表槽重用但完整 HWND 值变化，未观察到完整值复用；扩大运行时事件队列溢出并按门禁拒绝。微软对句柄标识位的说明属于可变更实现细节，不能据此放松窗口代际要求。见[有界探针](../../../../docs/specs/epic-CU/story-CU-S4/evidence/windows-hwnd-reuse-bounded-probe-2026-11-05.md)。
- 两个独立 Rust 观察器在同一受控 WinForms 窗口销毁前后得到一致的事件计数和窗口失效判定；新窗口的 HWND 不同，且尚无同一动作帧内的 Host/Worker 身份冻结与派发证据。这只是可见性证据，不解除上述窗口代际门禁。见 [双进程实验](../../../../docs/specs/epic-CU/story-CU-S4/evidence/windows-dual-lifecycle-observer-2026-11-05.md)。
- 固定上游 Windows 鼠标和键盘 `SendInput` 路径的 `dwExtraInfo` 均为 `0`，没有绑定 Yonder attempt/Worker 的归属标记；低级钩子的 injected 标志只证明程序注入，不证明属于当前 Worker。UIA Invoke/Legacy 路径另需验证其事件可见性。当前不能据此通过 `INPUT-02` 或放开原生派发；如需扩大固定上游补丁范围，必须先修订本决策和 OpenSpec。见 [输入归属源码审计](../../../../docs/specs/epic-CU/story-CU-S4/evidence/upstream-windows-input-attribution-source-audit-2026-11-05.md)。
- Windows 低级输入钩子只能作为**诊断**：微软明确说明超时后鼠标和键盘低级钩子可能被[静默移除](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc)、[应用无法获知](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc)。线程仍运行、安装调用成功或短时计数增长都不证明钩子持续覆盖输入；不能作为派发准入的 observer ready 事实。若采用 Raw Input 等替代来源，仍须另证其他程序注入事件的覆盖与当前 Worker 归属。原生派发继续关闭。
- 固定上游 macOS CGEvent 鼠标/键盘构造路径也未设置当前 attempt/Worker 标记。Apple 提供 `eventSourceUserData` 候选字段，但当前没有 macOS Event Tap 接收与归属的原生证据；不能把 API 可用性冒充 `INPUT-02` 通过。见 [macOS 输入归属源码审计](../../../../docs/specs/epic-CU/story-CU-S4/evidence/upstream-macos-input-attribution-source-audit-2026-11-05.md)。
- Windows 生命周期观察器只覆盖当前桌面；普通权限进程不能以 UI Automation 访问提升权限应用的 UI。Host 与 Worker 的只读身份 oracle 必须拒绝无法确认位于当前输入桌面、同一会话或高于自身完整性级别的目标，不能把此类目标的零事件计数当作窗口未变化。依据：[SetWinEventHook 范围](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook)、[UIAccess 安全边界](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-securityoverview)。
- 上游 machine-wide interaction lease 只作为单次 dispatch 的附加互斥。任务级 ownership、fencing、崩溃恢复和唯一前台桌面租约仍由 Yonder 持有。

## 受限补丁扩围决策门槛（待定）

当前唯一 hit-test 补丁及 `prepare_upstream.py` 的精确改动文件集属于已冻结的 v1 Spike 输入。Windows 的窗口代际交付缺口和 `SendInput` 归属缺口不能靠修改 Worker 前置检查或忽略所有 injected 事件消除。扩大上游 patch set 会使现有 patch hash、patched source tree hash、build manifest 和既有原生证据失去对新源码的证明力；不得在原目录静默覆盖。

如继续试验 v2，必须在改上游源码前另行冻结并审阅以下同一套 Windows/macOS 设计：

1. 指定**最终副作用交付点**的身份门禁，说明窗口销毁、同进程重建和原生窗口 ID 复用时如何避免重新绑定替代对象；只有调用前后两次只读核验或 UIA RuntimeId 比较不够。若固定引擎无可验证的交付点机制，则停止 v2 并淘汰候选。
2. 指定能由 Host 观察器验证的当前 `attempt_id + worker_instance_id` 合成输入归属机制，覆盖 Windows `SendInput` 鼠标/键盘及 macOS 合成事件；UIA Invoke/Legacy、AX 语义动作须单独说明是否产生低级输入事件。未知或未绑定事件一律递增 generation 并冻结；不得按时间窗口、注入标志或 `dwExtraInfo = 0` 推断归属。
3. 新 patch set 使用同一固定上游 revision、同一跨平台补丁文件集、独立目录与新哈希/manifest；相关样本定义版本升级并在双平台重跑。v1 只读证据继续保留，不能冒充 v2 动作证据。产品 Adapter、Gateway 和持久化仍不变。

本节记录评估 v2 的必要条件，**不批准实施 v2 补丁或执行原生动作**。在设计与证据尚不能满足第 1、2 项时，维持 v1 原生 `unsupported/not_delivered`；五工作日期限内仍无可行机制，则以 Rejected 收口，不以缩小样本范围延长。

## 抢占、取消与结果语义

用户输入观察器只维护宿主会话级单调递增 `u64 input_generation` 与封闭分类 `driver_current_attempt | user | unknown | observer_lost`，不记录按键、文本或轨迹正文。只有能绑定当前 `attempt_id + worker_instance_id` 的合成事件归为 `driver_current_attempt`；其余三类都递增 generation 并冻结派发。observer 未就绪、丢失、重启或 generation 回退时，一律按 fail-closed 处理；Worker 重启不得重置 generation。

派发、用户抢占和取消共用 Application-owned admission gate。唯一派发线性化点是 Supervisor 在最后一次比较 control/fence/generation/目标身份后，将绑定当前 attempt/Worker 的完整 JSONL 动作帧写入继承的 stdio 并完成 flush：之前为 `not_dispatched`，之后保守视为 `in_flight`。等待上游 interaction lease 发生在 Worker 接收帧之后，因此等待期间发生抢占也按在途 `unknown` 处理。

未派发请求和步骤之间可以确定取消。动作进入 Driver 后，公开接口没有可依赖的协作式取消；Supervisor 只能终止独立 Worker。只要副作用可能已经交付，结果必须是 `unknown`，不得报告 `cancelled`、不得自动重试。

Worker 必须受 Windows Job Object `KILL_ON_JOB_CLOSE` 或 macOS 受监管进程组约束，禁止派生脱离监管的 helper。退出后使用无副作用只读探针检查进程树、machine-wide lease、修饰键/鼠标按钮、临时文件，以及由全新实例执行的 Observe；禁止发送合成 key-up/button-up 冒充清理。任一残留或无法证明释放时进入全局桌面 quarantine，任务结果保持 `unknown`，禁止任何新 CUA 派发，直至宿主重启或明确只读恢复程序全部通过。

正常动作返回后，Application 必须强制新 snapshot/Observe。`delivered_unverified`、Observe 失败、断连、身份不一致或入口未保留完整 delivery/retry disposition 时统一进入 `unknown`。Driver 响应、进程退出或 lease 释放都不能单独成为任务成功或安全停止事实。

Host 的成功判定不得采信 Worker 自报 predicate。Host 只把 Worker Observe 当作候选快照，然后由独立只读 oracle 在 500 毫秒稳定窗口内执行至少两次、间隔不少于 250 毫秒的目标身份与 predicate 采样；两次结果均匹配且稳定才可完成该步。

## Spike 门槛

期限为 2026-11-05 至 2026-11-11。Windows/macOS 必须使用同一验收语义，覆盖可信目标、身份漂移、用户输入抢占、取消/故障、显示拓扑、权限、分发以及固定应用任务集；固定应用任务集必须包含 Windows WPS/文件资源管理器与 macOS Office/Finder。精确 action、predicate、oracle、deadline 和平台样本以 [固定样本目录](../../../../openspec/changes/cu-s4-agent-desktop-driver-spike/sample-catalog.md) 为唯一执行来源。

缺少已授权测试环境时只能记为 `environment_unavailable` 并保持 Spike 未完成；目标已安装但 Driver 不支持时记录 `unsupported` 并触发淘汰。证据采用版本化 schema，只保存安全枚举、布尔、单调耗时、哈希和版本，不保存输入正文、截图、完整窗口树或 Agent Payload。

任一条件成立即淘汰候选路线：

1. 用户输入发生后仍派发下一动作，或无法可靠区分用户输入与 Driver 合成输入；
2. 目标身份变化后发生一次误投递，而不是 stale/ambiguous/unknown 拒绝；
3. 终止 Worker 后 interaction lease、按键/鼠标状态或临时资源残留；
4. 任一动作不能稳定归为“已 Observe 验证”或“unknown 且不重试”；
5. 不能由 Yonder 从固定源码可重现自建，或不能分别证明构建 provenance、Windows Authenticode 可签/验签、macOS Developer ID/hardened runtime/notarization/stapling/Gatekeeper 责任链；没有生产证书时必须明确为环境未满足，不能以 Sigstore provenance、测试证书或 ad-hoc 签名冒充产品身份通过；
6. 干净 Windows 仍依赖未声明运行库，或不能生成 SBOM 与第三方许可清单；
7. 任一平台硬门槛失败，或只能通过长期保留 trycua 与 agent-desktop 双栈才能采用。

## 结论收口

Spike 结论必须更新本 ADR 或建立后继 ADR，并由独立 Verification Goal 复核；Spike 本身不授权产品接线。

研究依据：[agent-desktop 作为 Yonder CUA 候选评估](../../../research-vercel-labs-agent-desktop-yonder-cua/research-vercel-labs-agent-desktop-yonder-cua.md)。
