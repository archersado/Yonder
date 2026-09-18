# DS-S1 原生Enter与空格唤醒独立 Verification Goal

Story：DS-S1；OpenSpec：e0-validate-desktop-foundation；日期：2026-09-13。

依据：增量规格「点击唤醒」的Enter/空格入口。状态：本机已获焦点的双眼按钮通过两种原生按键唤醒；不代表Windows或所有键盘导航完成。

## 方法

扩展check-accessibility-macos.swift的--enter/--space模式，仍限定bundle及PID，只读取目标窗口树。等待真实休眠按钮，要求其AXFocused=true；如未获焦点先尝试设置并再读确认，不满足即停止。

通过CGEvent创建按下/松开，使用postToPid仅投递Yonda进程，间隔50ms。不使用system-wide对象、不发全局按键、不调用AXPress，也不额外激活应用。不加速产品闲置时间。判断依靠之后的清醒按钮名称和200至201窗口尺寸，不把API调用本身当作成功。

最初尝试的AXUIElementPostKeyboardEvent被当前Swift SDK明确禁用，编译失败；当时旧探针因参数不支持退出2，未发送按键。已改用公开CoreGraphics接口重新编译后执行，没有绕过SDK限制。

## 结果

当前PID9322，窗口27386；产品二进制仍为无障碍修复版本，SHA-256 `fa06f9f3c8e6159c116438d1eb59e4b409f0c313097d05f725644807f40b5e3f`，本轮未改产品代码。

| 输入 | 证据（spikes/desktop-foundation/evidence/） | 前置 | 后置 | 结果 |
|---|---|---|---|---|
| Enter | keyboard-enter-20260913.jsonl | 休眠AXButton、focused=true、app_active=false | 清醒标签、200×201、约359.87ms | 退出0 |
| 空格 | keyboard-space-20260913.jsonl | 休眠AXButton、focused=true、app_active=false | 清醒标签、200×201、约595.31ms | 退出0 |

命令分别为 `/private/tmp/yonda-check-accessibility 9322 <上述JSONL绝对路径> --enter` 和 `--space`。按键发送权限预检true，日志记录目标PID与输入模式。

Enter最初沿用旧通用字段press_result=0，此值是工具流程状态，不是AXPress返回或系统按键收件确认；本轮未调用AXPress。随后明确区分字段，空格日志ax_press_result=-1表示该API不适用。两轮均以实际后置状态判成功，历史原始日志未改写。

延迟包含焦点核查、50ms按键间隔和20ms轮询，标签在waking阶段已变化，不是动画完成时间，也不是架构300ms窗口状态更新链路指标。

## 范围

本机原生按键事件到WebView的Enter/空格处理路径已覆盖；不是人工物理键盘操作记录。未证明通过Tab从其他控件导航到桌宠、可见焦点样式、长按repeat、减少动态效果或Windows。不要因此新增“必须人工按物理键盘”门槛；若验收交互导航，则另行覆盖该具体流程。

DS-S1其余门禁仍保留，不Archive。
