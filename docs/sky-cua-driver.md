# Sky CUA driver

Yonder 将 Sky 作为 `ComputerUsePort` 的可选执行后端，而不是把 ChatGPT/Codex 应用或其原生服务嵌入产品。默认 driver 仍是 `trycua`；设置 `YONDER_CUA_DRIVER=sky` 后，组合根改用 `sky_cua_worker.mjs`，选择失败时关闭 CUA，不静默回退到另一个 driver。

## 运行边界

- `@oai/sky` 和原生服务来自用户已安装、版本兼容的外部产品。Yonder 不复制、不修改、不重新签名这些文件。
- 当前验证的 SDK 身份为 `@oai/sky@0.7.1`，入口可用 `YONDER_SKY_SDK_PATH` 指定；macOS 默认探测 ChatGPT.app 内公开的包入口。
- macOS worker 通过 Sky 的 native-pipe transport 连接已安装的 `SkyComputerUseService`。服务尚未启动时，优先使用 `SKY_CUA_SERVICE_PATH`，其次探测 `~/.codex/computer-use/Codex Computer Use.app`，最后由 Sky 使用服务 bundle id 启动。
- worker 禁用 Sky 的环境分析网络请求。应用策略和服务端禁止列表仍先于动作执行。
- Yonder 的任务准入是 worker 的授权边界。Sky 要求的 session elicitation 只承接这次已批准的执行，不扩大任务、目标应用或步骤范围。

## 选择方式

```sh
YONDER_CUA_DRIVER=sky \
YONDER_SKY_SDK_PATH=/absolute/path/to/@oai/sky/dist/project/cua/sky_js/src/index.js \
/path/to/Yonda.app/Contents/MacOS/yonder-desktop
```

不设置 `YONDER_CUA_DRIVER` 或显式设为 `trycua` 时，行为与原实现一致。任何其他值都会禁用 CUA。`YONDER_SKY_SDK_PATH` 必须是绝对路径、普通文件且不能是符号链接；包名和版本必须精确匹配。

## 目标绑定和结果确认

Yonder 继续以 `pid + window_id` 锁定工作目标：

- macOS Sky 只接受应用标识。worker 从已锁定 PID 的进程命令解析唯一 `.app` 路径；无法解析时拒绝执行，不选择最近使用应用。
- macOS 的 `launch_app` 使用 Sky 文档约定的 `get_app_state` 透明后台启动语义，并把 `list_apps` 返回的 canonical bundle id 只缓存在同一任务和 worker 会话内。当同一 bundle id 同时存在安装副本与 App Translocation 副本时，worker 仅可从本机运行进程和 `Info.plist` 反向解析唯一的实际 `.app` 路径供 Sky 观察；路径不接受 Agent 输入、不进入协议或日志，无法唯一收敛时拒绝执行。公开 API 没有 `activate_window`，因此后续显式 `bring_to_front` 用该可信 bundle id 解析唯一运行实例及其最大普通窗口，外挂复用 Yonder 的原生 `MacWorkFocus` 精确激活并验证，再由 Sky 对同一实际应用路径做后置观察；不接受 Agent 提交 PID、窗口号或路径替代缓存身份。
- Linux/Windows Sky 使用窗口对象。worker 只接受 `list_windows()` 中与 `window_id` 精确相等的窗口。
- 每个动作前后都执行观察。只有辅助功能文本或截图发生变化时才返回 `confirmed`；无变化返回 `suspected_noop`，由 Rust 侧收敛成未验证结果。
- 截图只写入 Yonder 的 0700 evidence 目录，单文件上限 4 MiB；SDK 错误、辅助功能正文和截图不会进入宿主日志。

## 平台状态

| 平台 | Sky JS 适配层 | 原生后端 | Yonder 状态 |
| --- | --- | --- | --- |
| macOS | `targets/mac` | `SkyComputerUseService` | 已接线、待端到端验收 |
| Linux | `targets/linux` | `sky_linux_<arch>` helper | worker 协议兼容，未交付 helper，未验证 |
| Windows | `targets/windows` | Windows helper transport | worker 协议兼容，未交付 helper，未验证 |

`SkyComputerUseService` 本身不是跨平台组件。跨平台的是 Yonder 的 driver 契约和 `@oai/sky` 的平台分派；Linux/Windows 必须各自获得可合法安装、版本锁定的原生 helper，才可加入发布 manifest。

## Chronicle 与 Record & Replay

Chronicle/Skysight 和 Record & Replay 可以在以后作为 Sky driver 的附加 capability 接入，但不属于基础动作执行路径：

- 基础 driver 只负责观察、动作、结果确认和会话结束。
- record/replay 应记录语义事件和稳定目标，而不是盲目回放屏幕坐标。
- Chronicle 的滚动捕获、分段和摘要需要单独的生命周期、保留期限和隐私开关。
- capability 必须由运行时探测，缺失时不影响基础 Sky CUA；不得因为插件不可用而回退到未经确认的录屏方案。
