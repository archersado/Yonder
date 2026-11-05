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
