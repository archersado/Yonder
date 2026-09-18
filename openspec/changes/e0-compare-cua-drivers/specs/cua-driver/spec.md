# CUA Driver 对照增量规格

状态：Frozen历史Delta。唯一选型要求已完成；后续产品要求已迁入CU-S2/TM-S3及关联OpenSpec，本Delta不再授权Apply。

### Requirement: 隔离AX执行中排空

STOP-DRAIN01–03 SHALL仅在原生setter报告写入开始且SDK输入尚未返回后请求shutdown；SHALL校验关闭确认不早于原生应用及新SDK Observe与目标状态匹配。SHALL NOT用JS提交、未证明原生开始或动作间关闭替代执行中证据；场景结果不得外推强制中断或其他SDK输入。

### Requirement: SDK原生输入权限门禁

STOP-IN01 SHALL在导入SDK宿主只读检查权限，不请求权限或采集正文。Accessibility不足时SHALL NOT派发输入或使用像素/剪贴板兜底；无截图AX样本SHALL NOT因缺屏幕录制而自动请求该权限。权限检查不代表正式Yonder宿主或输入停止已验证。

### Requirement: SDK隔离AX输入与动作间停止

STOP-IN02–03 SHALL仅向唯一PID/窗口/AX字段的临时隔离目标后台输入固定测试标记，动作后无截图Observe与目标状态同时匹配才通过；shutdown后第二输入SHALL拒绝，新SDK Observe及750ms目标布尔/长度稳定验证SHALL保留结果。SHALL NOT向其他窗口兜底、保存原始AX正文或将动作间关闭宣称执行中动作中断。

### Requirement: 仅SDK的自管Worker验证

按Accepted AD-CU-01，当前方案SHALL仅加载trycua SDK，不分发或启动上游App；STOP-PKG要求撤回，保留历史拒绝证据。STOP-SDK01–04 SHALL记录自管SDK子进程就绪、监督停止/退出、通道断开退出及新实例只读恢复，超时SHALL判失败；SHALL NOT由此宣称原生输入停止或开启Recording。

### Requirement: 固定版本macOS独立构件门禁

此要求已由AD-CU-01撤回，仅保留历史，不是当前SDK输入前置。

Harness SHALL按CU-S1 STOP-PKG01–03核对官方发布哈希、安全解包并记录原生签名/系统评估。任何门禁失败SHALL NOT运行、重签或移除隔离属性；下载不等于系统安装或真实输入能力通过。

## macOS只读停止补充范围

关联CU-S1 STOP-SUB01–04。Harness SHALL记录提交后的Abort竞态、取消后新读与并发关闭结果，并校验关闭后拒绝；SHALL NOT将JS调用提交/取消或排空完成冒充原生输入停止确认。样本仅list_apps，不触发输入、截图或Recording，结果仅布尔/错误分类。

#### Scenario: 取消与只读完成竞争

- **WHEN** 只读调用提交后下一事件循环发出Abort
- **THEN** 记录Abort是否先于Promise完成及真实成功/拒绝结果
- **AND** 校验后续只读可用，不断言原生动作已经停止

## ADDED Requirements

### Requirement: 无 Agent 对照

Harness SHALL 直接调用候选 Driver/SDK，不得加载 Agent、模型或 Planner。

#### Scenario: 执行固定动作

- **GIVEN** 两候选运行在相同设备与应用状态
- **WHEN** Harness 提交相同语义动作与预期条件
- **THEN** 保存每个候选的原始结果、耗时与验证结论

### Requirement: 故障语义验证

Harness SHALL 验证取消、超时、进程崩溃和结果未知状态。

#### Scenario: 动作中 Driver 退出

- **WHEN** Driver 在副作用动作完成确认前退出
- **THEN** Harness 将结果记录为 unknown
- **AND** 不得自动重复动作

### Requirement: 唯一选型

Spike SHALL 依据预先声明的淘汰门槛输出唯一结论。

#### Scenario: 候选未通过首版 Windows 核心用例

- **WHEN** 候选无法在 Windows 安全完成首版核心用例
- **THEN** 淘汰该候选
- **AND** 不得以双栈进入产品实现

### Requirement: 平台范围延期

E0-S2 SHALL 不以 macOS 和 Microsoft Office 阻塞首版 Windows CUA Driver 决策。

#### Scenario: 首版范围验证完成

- **GIVEN** WPS 代表办公套件场景
- **WHEN** Windows 的普通权限输入、文件资源管理器、故障语义与崩溃恢复均已验证
- **THEN** 允许完成 E0-S2
- **AND** macOS 对等验证进入后续 Epic

2026-09-14补充FOCUS-SDK原生工作定位Spike，关联TM-S3/CU-S2三份设计与AD-TM-07/AD-CU-02；期限及样本见TAKEOVER-STOP-PLAN.md。只验证SDK契约与隔离AX/AppKit精确前置/恢复/失效拒绝，不授权产品任务接管，不引入附加App；独立verification-focus-macos.md，Windows暂缓。
