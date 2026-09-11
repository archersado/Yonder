# CUA Driver 对照结果

## 候选与供应链

| 候选 | 版本 | 许可证 | 原生交付 |
|---|---:|---|---|
| Qwen CUA SDK | 0.20.5 | MIT；Node runtime 含 MPL-2.0 notice | GitHub Release；checksums.txt + SHA-256；Windows UIAccess worker 再验 Authenticode |
| trycua CUA Driver | 0.25.0 | MIT | 固定版本的按平台 optional npm package |

旧 `@qwen-code/open-computer-use@0.2.3` 是 Go/PowerShell MCP 实现；当前 Qwen Code 已迁移到 CUA SDK，因此不作为最终候选。

## Linux 只读探针

相同 Harness 直接调用 SDK，不含 Agent/模型/Planner，只执行 metadata、listToolsJson 和 list_apps。

| 指标 | Qwen | trycua |
|---|---:|---:|
| Driver 版本 | 0.20.5 | 0.25.0 |
| Contract | 0.7.0 | 0.7.0 |
| MCP 协议 | 2025-06-18 | 2025-06-18 |
| 工具数 | 60 | 59 |
| 初始化至响应 | 142 ms | 128 ms |
| list_apps | 成功、无降级 | 成功、无降级 |

工具差异：Qwen 独有 `perform_secondary_action`；trycua 无独有工具。两者 `list_apps` 文本长度一致，表明底层契约高度接近。

Qwen 在当前 Wayland compositor 缺少 data-control 协议时提示剪贴板初始化失败并降级 X11；只读应用枚举不受影响。

## 当前结论

### Windows 只读与生命周期基线

| 指标 | Qwen | trycua |
|---|---:|---:|
| 工具数 | 57 | 56 |
| 应用、窗口枚举 | 成功、无降级 | 成功、无降级 |
| WPS 窗口 | 可发现 | 可发现 |
| 文件资源管理器窗口 | 可发现 | 可发现 |
| 未知工具 | 返回 `isError` | 返回 `isError` |
| 调用前取消 | 抛出 `AbortError` | 抛出 `AbortError` |
| 重复关闭 | 幂等成功 | 幂等成功 |
| 关闭后调用 | 拒绝 | 拒绝 |

当前 Windows 未安装 Microsoft Word/Excel/PowerPoint，不能伪造该项证据。Qwen 枚举到 52 个窗口，trycua 枚举到 35 个窗口；字段结构相同，但差异原因尚待以相同时间点快照复核，不能据此判断优劣。

Qwen `launch_app` 已使用明确的 WPS 主程序路径成功打开隔离临时文档。发现 `list_apps` 的模糊名称匹配可能命中 WPS Installer，因此正式实现不得凭名称执行 `launch_path`，必须校验可执行文件身份或使用已确认的应用注册项。

### Windows WPS 输入探针

- Qwen：可以观察独立临时文档；WPS 正文没有暴露可编辑 AX 元素，转为像素输入后返回 `uiaccess_worker_unavailable`，文件未被修改。
- trycua：可以调用 WPS，并枚举到 2 个 WPS 顶层窗口；WPS 复用现有窗口/标签页后，窗口 ID、标题与 AX 查询均未暴露随机测试文档标记。为避免误操作其他已打开文档，探针按 fail-closed 原则没有执行输入。
- Qwen 官方安装脚本已实际执行到安全门禁并停止：SHA-256 与 Release `checksums.txt` 一致的 `0.20.5 windows-x86_64-binary` 包中，`qwen-cua-driver-uia.exe` 的 Authenticode 状态为 `NotSigned`。官方脚本要求 `Valid`，因此没有写入 Windows `Program Files`，也没有绕过校验。
- 这构成 Qwen 0.20.5 的 Windows 安全分发阻断项。除非上游提供有效签名构件，或 Yonder 明确承担自行构建和签名该 worker 的维护成本，否则该版本不能作为可发布执行引擎。
- 在仅有一个 WPS 窗口的隔离会话中，trycua 可以继续到 observe 与像素输入调用，但输入和关闭均被拒绝：目标 WPS 为 High integrity，Driver 为 Medium integrity，Windows UIPI 会丢弃跨完整性级别消息。测试文件未被修改。全量提升 Yonder/Driver 权限不符合轻量桌面工具的默认安全模型。

两者均通过 Linux 与 Windows 只读冒烟，生命周期故障语义一致。WPS 用例表明 Qwen 的权限结构更完整但发布构件不可安全安装，trycua 则会按 UIPI fail closed；因此继续用普通权限应用区分首版可用性。原始应用和窗口列表属于本机敏感信息，仅保存在 gitignore 的 `evidence/` 中。

### Windows 普通权限文本应用

- Qwen：记事本启动与 observe 成功；输入仍要求安全路径中的签名 UIAccess worker，因此失败。
- trycua：记事本启动与 observe 成功，AX 文本输入返回 `effect=confirmed`、`route=accessibility`。
- trycua 对现代 XAML 记事本的 `Ctrl+S`、`Alt+F4` 无法找到 UIA accelerator，且不静默伪装成功；文件保存与关闭失败。这要求 Yonder 在每步动作后 observe，并为快捷键准备经验证的替代路径。

## 首版选型

首版 CUA Driver 选择 `@trycua/cua-driver@0.25.0`，只启用已验证的普通权限应用能力。Qwen 0.20.5 因官方 Windows 输入构件未签名而淘汰，不进入依赖树。

trycua 的 Windows 范围暂不宣称支持 High integrity 应用，也不宣称现代 XAML 快捷键完整可用。WPS 优先走 OOXML Adapter；需要 GUI 操作时，仅允许目标与 Driver 同为 Medium integrity 且动作通过 observe 验证。否则 fail closed，不整体提权 Yonder。

### trycua 补充门禁

- 异常恢复：Driver 所在 Node 子进程未经 `shutdown` 以退出码 23 终止后，新进程在 Windows 821ms 内重新初始化并成功 `list_apps`；Linux 对照为 62ms。
- 文件资源管理器：在随机隔离目录中唯一定位测试窗口和 `ListItem`，AX `click` 调用成功；动作自身标记 `effect=unverifiable`，后续 observe 确认文件 `selected=true`，窗口随后正常关闭。
- 这验证了 Yonder 的执行规则：Driver 返回成功不等于任务成功，必须用动作后的状态断言决定继续、局部重规划或停止。
