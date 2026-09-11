# AD-E0-02 CUA Driver（临时结论）

- 状态：Accepted
- Story：E0-S2
- OpenSpec：`e0-compare-cua-drivers`
- 日期：2026-09-10

## 当前决定

首版选择 `@trycua/cua-driver@0.25.0` 作为唯一 CUA Driver；Qwen CUA SDK 0.20.5 因 Windows 官方输入构件未签名而淘汰，不进入依赖树。首版范围为 Windows，macOS 对等验证移入后续 Epic。

## 证据

两者在 Linux/Windows 的只读能力与生命周期故障语义基本一致。Qwen 对普通权限记事本输入也强制依赖未签名 worker，无法安全执行。trycua 在普通权限记事本完成 AX 文本输入，报告 `effect=confirmed`；但现代 XAML 快捷键失败。Windows WPS 为 High integrity，trycua 的 Medium integrity 输入被 UIPI 拒绝。

trycua 的 Node 进程异常退出后，新实例在 Windows 821ms 内恢复 observe。文件资源管理器 AX 点击虽然返回 `effect=unverifiable`，动作后 observe 能确认测试文件进入选中状态，证明其可支撑 Yonder 的 observe-after-step 执行模型。

## 架构围栏

- 禁止关闭 Authenticode 校验或安装未签名 UIAccess worker。
- 禁止为了 CUA 默认提升整个 Yonder、Agent Gateway 或 Node 进程权限。
- 应用启动不得依赖模糊名称命中的 `launch_path`；必须验证可执行文件身份。
- 输入前必须唯一确认窗口/文档身份；不能确认时 fail closed。
- 不长期维护 Qwen 与 trycua 双栈。
- 首版只开放通过场景级验证的 trycua 能力；每个动作后必须 observe，效果未确认不得继续。
- WPS 文档内容操作优先使用 OOXML Adapter；CUA 不得绕过 Windows 完整性边界。

## 未决项

macOS/Finder 与 Microsoft Office 对等验证移入后续 Epic。若未来必须驱动 High integrity 应用，另立 Epic 评估由 Yonder 自有签名的窄权限 worker，不回退到整体提权。
