# CU-S4 固定样本目录

版本：`agent-desktop-spike-samples/v1`。

## 全局时限与重试规则

所有动作自动重试次数为 0；时间使用单调时钟。默认动作 deadline 为 10 秒，应用启动 deadline 为 20 秒，后置 Observe deadline 为 5 秒；稳定条件为 500 毫秒窗口内至少两次相同 predicate 结果，采样间隔不小于 250 毫秒。超时不得通过放宽阈值重跑，任何阈值变更都必须升级样本版本并重跑双平台相关样本。

## 固定样本矩阵

| Sample ID | Windows | macOS | `action_id` | `predicate_id` | `oracle_id` | 通过条件 |
|---|---|---|---|---|---|---|
| `IDENTITY-01` | 同名双 fixture 窗口 | 同名双 fixture 窗口 | `ACTION-CLICK-QUALIFIED-REF` | `P-TARGET-UNCHANGED-V1` | `O-NATIVE-WINDOW-IDENTITY-V1` | 只命中冻结目标；另一窗口零副作用 |
| `IDENTITY-02` | 关闭重开并制造 HWND 复用机会 | 关闭重开并替换 AX 对象 | `ACTION-CLICK-QUALIFIED-REF` | `P-OLD-TARGET-REJECTED-V1` | `O-NATIVE-WINDOW-IDENTITY-V1` | stale/ambiguous 拒绝，零误投递 |
| `ELEMENT-01` | Notepad + Electron fixture | TextEdit + Electron fixture | `ACTION-TYPE-FIXTURE-TEXT-QUALIFIED-REF` | `P-FIXTURE-STATE-STABLE-V1` | `O-NATIVE-ELEMENT-IDENTITY-V1` | qualified ref 动作后 Observe 稳定通过 |
| `APPSET-01` | WPS 固定无敏感文档 | Microsoft Office 固定无敏感文档 | `ACTION-TYPE-FIXTURE-TEXT-QUALIFIED-REF` | `P-APPSET-STATE-STABLE-V1` | `O-FIXED-APP-WINDOW-V1` | 已安装目标形成 Observe 闭环；缺环境不得判通过 |
| `APPSET-02` | 文件资源管理器固定测试目录 | Finder 固定测试目录 | `ACTION-CLICK-QUALIFIED-REF` | `P-APPSET-STATE-STABLE-V1` | `O-FIXED-APP-WINDOW-V1` | 零误选、动作后 Observe 稳定通过 |
| `FOCUS-01` | 前台/后台/最小化 | 前台/后台/最小化/Space | `ACTION-FOCUS-WINDOW-QUALIFIED-REF` | `P-TARGET-VISIBLE-V1` | `O-NATIVE-WINDOW-IDENTITY-V1` | 不把不可见或错误窗口报告成功 |
| `INPUT-01` | 10 轮真实鼠标移动/点击、普通键和修饰键 | 同语义 10 轮真实事件 | `ACTION-CLICK-QUALIFIED-REF` | `P-USER-PREEMPTED-V1` | `O-INPUT-GENERATION-V1` | 用户事件零漏分，事件后零新增派发 |
| `INPUT-02` | 10 轮当前 attempt 合成鼠标/键盘 | 同语义 10 轮合成事件 | `ACTION-CLICK-QUALIFIED-REF` | `P-DRIVER-INPUT-ATTRIBUTED-V1` | `O-INPUT-GENERATION-V1` | 合成事件零误分为 user；未绑定事件一律冻结 |
| `INPUT-03` | observer 未就绪/断开/重启 | observer 未就绪/断开/重启 | `ACTION-NONE` | `P-OBSERVER-FAIL-CLOSED-V1` | `O-INPUT-GENERATION-V1` | 全部 fail closed；Worker 重启 generation 不回退 |
| `CANCEL-01` | flush 前取消 | flush 前取消 | `ACTION-CLICK-QUALIFIED-REF` | `P-NOT-DISPATCHED-V1` | `O-DISPATCH-BOUNDARY-V1` | `not_dispatched`，Worker 未收到动作 |
| `CANCEL-02` | flush 后立即 kill | flush 后立即 kill | `ACTION-CLICK-QUALIFIED-REF` | `P-INFLIGHT-UNKNOWN-V1` | `O-WORKER-CONTAINMENT-V1` | `unknown`、零重试、只读清理通过或 quarantine |
| `FAULT-01` | deadline / stdio 断连 | deadline / stdio 断连 | `ACTION-CLICK-QUALIFIED-REF` | `P-FAULT-UNKNOWN-V1` | `O-DISPATCH-BOUNDARY-V1` | `unknown`、零重试 |
| `DISPLAY-01` | 96/192 DPI；主/副屏、负坐标、跨屏 | 1x/2x；主/副屏、负坐标等价拓扑、跨屏 | `ACTION-CLICK-QUALIFIED-REF` | `P-REF-DISPLAY-INDEPENDENT-V1` | `O-DISPLAY-TOPOLOGY-V1` | qualified ref 不依赖 raw 坐标且零误投递 |
| `PERMISSION-01` | 普通权限与高完整性拒绝 | AX 权限拒绝 | `ACTION-CLICK-QUALIFIED-REF` | `P-PERMISSION-REJECTED-V1` | `O-PLATFORM-PERMISSION-V1` | fail closed，不提权、不打开设置、不重试 |
| `DIST-01` | 干净 Windows、静态 CRT、Authenticode 分层证据 | Developer ID/hardened runtime/notarization/stapling/Gatekeeper 分层证据 | `ACTION-NONE` | `P-DISTRIBUTION-READY-V1` | `O-BUILD-MANIFEST-V1` | 无未声明依赖；未具生产证书不得冒充通过 |

## Predicate、Oracle 与证据边界

所有 predicate 与 oracle 的规范定义位于 `sample-definitions-v1.json`；build manifest 和每条证据分别记录该文件及具体定义的 SHA-256。所有 fixture 使用固定无敏感标记。证据只保存 predicate/identity verdict，不保存标记正文、截图、完整窗口树或应用列表。

## `IDENTITY-01` 固定 fixture 选择规则

- 同一自建 fixture 进程创建两个标题均为 `Yonder Identity Fixture` 的可见顶层窗口，目标和诱饵各含一个角色、标签与元素标识均相同的按钮；标题、前台状态和窗口列表顺序均不是 selector。
- Windows 使用 WinForms 按钮 `Name = YonderIdentity01Button`，完整 UIA 快照中 `AutomationId = YonderIdentity01Button` 在**冻结窗口内**必须恰好匹配一次。fixture 自行设置非零且不可变的 `YonderCUIdentity01Role`（目标 `1`、诱饵 `2`）和 `YonderCUIdentity01Generation`（进程内每次建窗递增且互异）window property。Host 启动 fixture 后按 PID 与只读 role 定位目标，冻结 HWND、进程启动摘要和 generation 摘要；Worker 独立重读 role/generation。Host/Worker 不调用 `SetPropW`。
- macOS 使用 AppKit 窗口与按钮；两个按钮的 `AXIdentifier` 都是 `YonderIdentity01Button`。fixture 创建窗口时设置互异、不可变的窗口 `accessibilityIdentifier`，格式为 `YonderIdentity01Window:<target|decoy>:<generation>`；Host/Worker 只读 AX 属性并与 PID、原生窗口 ID 绑定。若目标窗口或唯一按钮的 AX 属性不可读，即 `unsupported`，不得按标题回退。
- Windows 的窗口代际摘要为 `SHA-256("identity-01/windows/v1" || NUL || process_start_sha256 || NUL || generation 十进制值)`；macOS 把前缀替换为 `identity-01/macos/v1`，输入为完整窗口 `accessibilityIdentifier`。两个平台都只在内部计算 token，不把原始标记写入 Spike 证据。
- Windows fixture 自行维护每个窗口的 `YonderCUIdentity01ClickCount` property，数值 `1` 表示 0 次、`2` 表示 1 次点击；macOS fixture 在每个窗口的固定状态元素中暴露非敏感点击计数。Host 至少两次只读采样验证目标恰好 1 次、诱饵 0 次，且冻结身份未变化。样本失败时不把 Worker 自报结果当作通过。
- fixture 专用代际标记仅用于这个受控样本。`IDENTITY-01` 的真实动作仍需先满足 WinEvent/AX 对象生命周期、Host/Worker 一致性和 Worker ref 解析至交付路径的安全门禁；它不授权通用应用派发。
