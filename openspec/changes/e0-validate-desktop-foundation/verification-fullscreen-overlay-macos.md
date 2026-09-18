# DS-S1 轻量插件与全屏辅助显示独立验证

Story: DS-S1
OpenSpec: e0-validate-desktop-foundation
日期：2026-09-12
依据：AD-E0-01 全屏验证补充、用户“仅轻量桌面插件，不提供全屏应用形态”。
状态：本机清醒形态场景通过；Story 与 ADR 未完成，不 Archive。

## 最终实现

macOS 使用 Accessory 激活策略与预览包 LSUIElement，保留桌宠和托盘，无普通 Dock 应用入口。NSWindow 设置 FullScreenAuxiliary 与 FullScreenDisallowsTiling；macOS 13+ 按公开 SDK 语义设置 CanJoinAllApplications 并清除冲突角色。普通桌面保持原有 CanJoinAllSpaces。Yonda 不创建全屏主窗口；测试全屏的是自动关闭的纯色辅助程序。

复用已有锁定 objc2 0.6.4 和 objc2-app-kit 0.3.2，仅新增 macOS 直接引用。锁文件只新增桌面包的依赖边，无第三方版本升级。没有任务状态、Gateway、密钥或平台公共协议变化。

## 失败与修复证据

1. 原版本在全屏稳定 4 秒后，小龙所在区域仅剩测试窗口背景；普通桌面与返回后有小龙。初次动画采样还出现离屏坐标，不以它单独定论。
2. 单加 FullScreenAuxiliary 未通过；再加 Accessory 后，插件形态确认生效，但全屏悬浮仍未通过。两个失败版本证据保留。
3. SDK NSWindow.h 明确 CanJoinAllApplications 适用于加入其他应用全屏空间的悬浮窗口。补充此标记后，同一原生测试通过。

## 最终验证

`cargo build --release --offline` 更新直接依赖边后构建成功；最终 `cargo test --release --offline --locked --manifest-path spikes/desktop-foundation/src-tauri/Cargo.toml`：2 项通过。打包后 plist 的 LSUIElement=true 检查通过。

最终证据目录：`spikes/desktop-foundation/evidence/fullscreen-overlay-20260912/`，含二进制 SHA-256、构建结果、原生日志、诊断脚本与普通/全屏 1 秒/全屏 4 秒/返回后的屏幕区域截图。先有一次临时测试未进入全屏超时，不算通过，单独保存在同前缀 timeout 目录。

成功复验：PID 66418、窗口编号 20605，原生 activationPolicy=1（Accessory）；小龙 CG 边界始终为 (1468,713,200,201)，全屏测试窗口为 (0,0,1710,1073)。200×201 是实际采集的 CG 边界，不覆盖产品配置的 200×200 逻辑尺寸。全屏 1 秒和 4 秒、返回后坐标均不变，测试窗口 test_active 始终 true。人工核对全屏 4 秒屏幕区域能看到完整小龙，未抢测试应用焦点。测试程序已自动关闭，Yonda 保持运行。

## 限制

只验证本机 macOS 和清醒形态；最新构建的休眠眼睛全屏、普通桌面之间切换、Windows、全进程性能、宿主 IPC 等门禁仍待验证。macOS 13 以下未设置新标记，不声明其跨全屏支持。前轮单窗口截图与 onscreen 标记的异常原因也未全部确定；本次通过依据实际屏幕区域内容。当前混合工作区未创建 PR，不将 Spike 通过一项宣称产品完成。

续作已补充同一构建的右侧休眠形态及返回唤醒，通过范围见 [独立记录](verification-rest-fullscreen-macos.md)。普通桌面切换、Windows 等其余限制继续保留。
