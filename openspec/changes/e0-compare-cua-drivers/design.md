当前归属 Story：CU-S1；规划：`docs/specs/epic-CU/story-CU-S1/README.md`。旧编号保留历史追溯。

状态：Frozen。下文仅保留候选对照与后续Spike历史；产品运行时以Accepted AD-E0-02/AD-CU-01和CU-S2正式实现为准，不再从本Change或`spikes/cua-driver-comparison`引入依赖、协议或执行入口。

# 设计

STOP-DRAIN01–03关联CU-S1三份设计/TAKEOVER-STOP-PLAN/AD-CU-01：在可选延迟AX setter的同一隔离目标内观测真实写入开始，SDK在自管子进程获取自己的token并输入；父进程看到原生开始且输入未返回时发送shutdown，核实关闭晚于原生应用，新SDK无截图Observe匹配最终标记。2秒延迟为样本设计建议，不是产品阈值；不修改生产协议/任务状态/Recording。

当前SDK-only决定以Accepted AD-CU-01及CU-S1三份设计为准，撤回上游App/可执行文件分支。STOP-SDK01–04复用现有SDK子进程探针，由Node父进程通过继承私有IPC观测SDK metadata就绪后发送SIGTERM并等待退出，或主动断开宿主通道验证子进程关闭退出；恢复复用新实例只读样本。仅Spike，无产品协议/数据库/UI变更，不把进程退出作为原生副作用结果确认。

STOP-IN01先复用stop-macos-probe的permissions模式，导入SDK只读检查当前宿主两权限布尔，不初始化输入目标、不请求权限。Accessibility不足则STOP-IN02–03保持未验证，不执行输入；权限足够后才建立隔离AX输入/Observe和动作间停止样本。执行中输入停止仍不由这些首批检查代替。

STOP-IN02–03按TAKEOVER-STOP-PLAN使用临时Swift测试可执行窗，唯一PID/窗口/AX字段后台固定标记输入，SDK动作后无截图Observe与目标布尔状态匹配；shutdown后第二输入拒绝，新SDK Observe及750ms目标状态稳定。测试窗不打包App、不进入产品、不记录原文。结果失败不向其他目标输入或自动重试。

供应链补充STOP-PKG01–03关联CU-S1三份设计及TAKEOVER-STOP-PLAN：下载官方固定版本macOS二进制归档到忽略证据目录，核对发布SHA-256并安全解包，原生codesign/Gatekeeper只读评估，失败不运行、不重签、不移除隔离属性；不运行安装脚本，不写系统目录。

裸二进制评估拒绝后返回实施，按固定版本官方脚本选用含CuaDriver.app的macOS目录包，单独核对发布哈希、完整App签名/标识与系统评估；不覆盖裸构件失败证据。

Harness 以固定 JSON 用例直接驱动两个 SDK，不包含 Agent 或规划逻辑。两者当前公开 API 与 Contract 相同，因此不预建两个空壳 Adapter；只有字段实际分叉时才增加最薄映射。每一步保存请求、脱敏响应、耗时、进程资源和预期条件结果。

首版核心用例：Windows 列举应用/窗口、获取 AX 树与截图、按元素点击、文本输入、取消、Driver 崩溃后重新观察。办公任务以 WPS 代表办公套件，并覆盖 Windows 文件资源管理器。Microsoft Office 与 macOS/Finder 对等验证移入后续 Epic。

候选按版本固定。安装脚本执行前必须检查包清单、完整性、许可证和下载目标；不得直接执行远程 shell。结论只允许 Qwen、trycua 或两者均淘汰。

macOS只读补充STOP-SUB01–04按CU-S1三份设计执行，探针复用stop-macos-probe的独立子进程，JS提交后下一事件循环Abort并记录竞争结果，再验证新读可用；另一次未等待读返回即shutdown并验证关闭后拒绝。结果不含应用正文；不观察或发出键鼠，不把JS Promise边界等同原生动作停止。仅运行已选trycua，历史双候选逻辑不重启。

2026-09-14补充FOCUS-SDK原生工作定位Spike，关联TM-S3/CU-S2三份设计与AD-TM-07/AD-CU-02；期限及样本见TAKEOVER-STOP-PLAN.md。只验证SDK契约与隔离AX/AppKit精确前置/恢复/失效拒绝，不授权产品任务接管，不引入附加App；独立verification-focus-macos.md，Windows暂缓。

2026-09-14：沿AD-CU-03工作定位Spike扩展身份复核，设计依据WORK-IDENTITY-PLAN与TM-S3/CU-S2三份设计/Proposed AD-TM-08。只扩展隔离夹具及原生探针保留模式，固定SDK0.25.0无截图Observe，无产品协议/持久化迁移。独立verification-work-identity-macos.md已PASS macOS隔离子范围；Windows、正式宿主权限、多Space/显示器、步骤停止及Recording门禁保留。
