当前归属 Story：CU-S1；规划：`docs/specs/epic-CU/story-CU-S1/README.md`。旧编号保留历史追溯。

# 任务

状态：Frozen。勾选的迁出项表示范围已有正式承接方，不表示目标Story全部完成。

- [x] 核对候选仓库、版本、许可证和分发结构
- [x] 排除旧 Qwen open-computer-use 作为正式候选
- [x] 审查两候选的原生安装脚本和下载完整性机制
- [x] 定义统一 Driver Port 与 JSON 用例格式
- [x] 实现最小无 Agent Harness
- [x] 完成 Windows 只读基础能力与生命周期故障用例
- [x] 完成 Windows 输入动作与崩溃恢复用例
- [x] 完成 Windows WPS/文件管理器可发现、窗口观察用例
- [x] 验证 Qwen UIAccess worker 安装门禁（官方 0.20.5 构件未签名，安全失败）
- [x] 淘汰 Qwen 0.20.5，不接收未签名替代构件
- [x] 复核 trycua 对复用 WPS 标签页的发现能力（无法安全确认文档身份）
- [x] 在无其他文档的隔离 WPS 会话复核 trycua 输入（被 Windows UIPI 阻断）
- [x] 完成普通权限记事本 AX 输入对照（trycua 通过，Qwen 被 worker 阻断）
- [x] 完成 Windows 文件资源管理器 AX 输入与动作后观察
- [x] Microsoft Office 输入动作移入后续 Epic；首版以 WPS 代表办公套件
- [x] macOS 对等用例移入后续 Epic
- [x] 汇总矩阵并产出 AD-E0-02
- [x] 完成 Verification Goal

## macOS接管停止补充（2026-09-14）

- [x] CU-S1三份设计/有期限Spike计划
- [x] 唯一trycua只读生命周期探针（原生语义断言通过）
- [x] 独立Verification Goal与结构化证据（仅STOP-READ01–04）
- [x] ADR及TM/RC停止前置回写（不扩大实际输入已验证范围）
- [x] 实际输入、Worker监督、停止与平台验证迁出到CU-S2/TM-S3及其独立Change

- [x] STOP-SUB01–04已提交只读取消/关闭竞态设计与原生断言
- [x] 独立verification-submitted-macos Goal（只读范围PASS）
- 上游独立Driver构件获取前置已撤回：用户SDK-only，Accepted AD-CU-01；真实输入/停止/Observe样本改为SDK路线。

- [x] STOP-PKG01–03供应链设计与最小核验探针、路径/链接拒绝自检
- [x] 固定版本裸二进制哈希/签名核验及Gatekeeper拒绝证据（不等于安全分发通过）
- [x] 官方安装脚本静态审查，确认App目录载体并核对脚本发布哈希
- 完整App目录包评估已撤回；下载完成但未解包/评估/运行，verification-package-macos=withdrawn，不称PASS。

- [x] SDK-only集成AD-CU-01及CU-S1三份设计/OpenSpec更新
- [x] STOP-SDK01–04自管SDK子进程监督/断连/恢复探针
- [x] 独立verification-sdk-worker-macos（macOS只读生命周期PASS）
- [x] SDK隔离输入/Observe历史验证保留；正式宿主权限责任链迁出到CU-S2

- [x] STOP-IN01当前SDK测试宿主权限只读检查（Accessibility已授权，不代替正式Yonder宿主）
- [x] STOP-IN02–03隔离AX输入/Observe/动作间关闭测试设计与原生测试窗/探针实现
- [x] 独立verification-input-macos（构建退出0后SDK隔离输入/Observe/动作间关闭PASS，首次启动失败保留）
- [x] 执行中停止、Worker残留及正式Yonder权限归属迁出到CU-S2/TM-S3

- [x] 实测输入交付route=accessibility/effect=confirmed（独立合成排空样本未成立，不称通过）
- 合成延迟控件排空Goal FAILED，按AD-CU-02停止该合成样本研发；不是通用SDK排空失败结论，不Archive。
- [x] 步骤边界控制、竞态、Observe与unknown占用迁出到CU-S2/TM-S3独立验证；宿主权限门禁继续由目标Change保留

2026-09-14补充FOCUS-SDK原生工作定位Spike，关联TM-S3/CU-S2三份设计与AD-TM-07/AD-CU-02；期限及样本见TAKEOVER-STOP-PLAN.md。只验证SDK契约与隔离AX/AppKit精确前置/恢复/失效拒绝，不授权产品任务接管，不引入附加App；独立verification-focus-macos.md，Windows暂缓。

2026-09-14：沿AD-CU-03工作定位Spike扩展身份复核，设计依据WORK-IDENTITY-PLAN与TM-S3/CU-S2三份设计/Proposed AD-TM-08。只扩展隔离夹具及原生探针保留模式，固定SDK0.25.0无截图Observe，无产品协议/持久化迁移。独立verification-work-identity-macos.md已PASS macOS隔离子范围；Windows、正式宿主权限、多Space/显示器、步骤停止及Recording门禁保留。

本Change冻结且不Archive；正式依赖、打包与运行验证见`cu-s2-supervised-dispatch-observe`。
