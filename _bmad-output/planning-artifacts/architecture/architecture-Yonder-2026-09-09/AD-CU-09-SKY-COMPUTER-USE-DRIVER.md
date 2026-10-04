# AD-CU-09 Codex Computer Use / Sky 单一 CUA Driver

- 状态：Accepted（macOS-only，Windows 暂缓）
- Story：CU-S4
- OpenSpec：`cu-s4-sky-product-driver`
- 日期：2026-09-30

## 问题

正式 QQ 音乐样本中，trycua 0.25.0 与 0.30.4 对已运行应用的精确窗口 Observe 均返回空元素或 `observe-failed`，无法支持元素优先定位。相同运行实例通过 Codex Computer Use 的 `@oai/sky` 应用级 `get_app_state` 可稳定取得完整 AX transcript，包括“文本框 搜索”等可操作元素及其新鲜 `element_index`。继续围绕 trycua 增加前台快捷键、坐标或第二观察器会形成双执行栈，并不能解决通用元素定位。

## 决定

1. macOS 产品 CUA 只使用 Codex Computer Use 的 `@oai/sky` Driver；移除 trycua 的生产依赖、环境选择、打包资源和运行时回退。历史证据保留，不作为当前产品接线。
2. Sky 作为外部已安装的受信任能力，Yonder 固定校验 `@oai/sky@0.7.1`，通过受监管按需 Node Worker 调用；不复制或重新签名 Sky 包及 Computer Use App，不开放 HTTP/TCP。
3. Worker 以应用 bundle id 或由可信 PID 解析的唯一 App bundle 绑定目标。计划启动应用后，同一任务后续步骤复用该绑定，不改投当前前台或模糊同名应用。
4. 每一步先读取新鲜应用级 AX transcript，优先使用 `element_index` 动作；只有 AX 缺失、多义或不可操作时才使用同一应用截图。动作后必须再次 `get_app_state`，不得复用旧 index。
5. AX transcript 仅用于 Worker 内定位和本次 Gateway 运行时 Observation；不得写入 SQLite、事件、Outbox、日志或顶部浮窗。对外只返回有界元素数量与既有临时截图引用。本次不新增持久化或第二状态源。
6. `focus-control/input-text/activate-control` 由 Worker 映射为 Sky 的 `click/set_value/press_key` 等原生动作；私有文本与语义标记不得进入日志。动作无法形成新鲜后置事实时保持 unknown/交回，不自动重试。
7. CUA Driver 仍模型无关；慢脑经 Gateway 提交有界计划片段，Jev 只在片段内选候选。Sky 不承担首次规划、语义 replan、任务状态或 UI 状态所有权。
8. Windows 路线和证据继续暂缓；macOS 结论不得外推。恢复 Windows 时必须另做同样本 Driver 决策，不恢复长期双栈。

## 取代关系

- 本决定取代 AD-E0-02 与 AD-CU-07 中“trycua 是 macOS 唯一产品 Driver”的结论。
- AD-CU-01 的 SDK-only 原则、AD-CU-05 的受监管单会话、AD-CU-08 的元素优先/同目标视觉降级，以及每步 Observe、unknown 不重试、单桌面租约继续有效。

## 验收

- 正式包不含 trycua 包或 Worker，组合根不存在 Driver 环境开关，仅在固定 Sky 能力可用时公布 CUA available。
- 隔离 Worker 测试证明应用级 transcript 元素计数、语义动作映射、任务目标绑定、动作后重新 Observe 与敏感字段清理。
- 正式 Yonder Gateway 的 QQ 音乐多步骤计划能读取搜索元素并推进；失败时携带新鲜同应用 Observation 交回。
- 独立 Verification Goal 复核包边界、Adapter 合约和 macOS 原生证据；Windows 项明确 deferred。

## 2026-10-01实施门禁

正式产品验证发现新的不可绕过边界：`@oai/sky`的macOS服务只接受ChatGPT/Codex可信宿主通道；Yonder直接连接同版本native-pipe会在ping响应前被关闭，阶段为`transport-closed`。SDK/服务版本和二进制哈希一致，Codex CUA REPL仍可正常读取同一QQ音乐实例，故不是应用AX或版本缺失问题。

因此本决定禁止Yonder直接连接native-pipe。随后在同一固定Sky包内发现官方签名的`SkyComputerUseClient ... mcp`桥接程序：它公开标准MCP stdio的`list_apps/get_app_state/click/set_value/press_key/...`工具，并持有服务要求的OpenAI Team ID与App Group entitlement。产品接线改为“Yonder受监管Worker → 固定身份的签名MCP Client → Sky服务”，Yonder本身无需也不得伪造OpenAI签名；桥接仍是同一Sky Driver的transport，不是第二Agent或第二执行栈。

组合根必须从已验证`@oai/sky@0.7.1`包相对定位该Client及同发行物的OpenAI签名Node，校验普通文件、包版本、Team ID、Client identifier与App Group。Sky Client还会验证直接父进程的固定OpenAI Team ID，因此只能由该签名Node运行受监管Worker并派生Client；Yonder不得用自身、通用Node或shell直接派生Client。Worker只通过stdio MCP调用并监管其生命周期，不直连Socket、不依赖ChatGPT GUI父进程、不复制签名/entitlement。

Client发出的应用使用elicitation只允许由同一任务已校验`launch_app`的唯一规范bundle id与显示名收敛：Worker仅接受固定版本生成的空对象Schema和精确应用提示，且只返回会话级accept；其他server request一律decline。该授权不替代Yonder对发送、删除等副作用的确认。该桥接通过独立Goal前，`cu-s4-sky-product-driver`仍处于Apply。
