# Windows 待验证总审计（2026-11-01）

范围：仓库内全部仍带 Windows/双平台门禁的未完成 OpenSpec 任务，以当前代码、任务清单和独立 Verification Goal 为准。按此前用户要求，本轮不执行 CUA；CUA 相关项保留但不计入本轮可推进队列。历史 FAIL/BLOCKED 若已有更新日期的 PASS 报告，以更新证据为当前结论，旧记录继续保留追溯。

2026-11-01 23:11 实机前置审计见 `windows-prerequisite-audit-20261101.json`：单显示器；存在2个麦克风类端点；无`TYPESAFE_API_KEY`；当前debug可执行文件未签名且未安装Yonder正式包；存在多个启用本地账户但当前会话非管理员。证据只记数量和布尔值，不记设备名、用户名或Secret。

## 已通过但清单曾滞后

| Change | 当前结论 | 权威证据 | 本轮处理 |
|---|---|---|---|
| `ag-s1-cloud-connector-spike` | Windows PASS | `verification-windows-20261024.md`：可信 WSS、Ping/Pong、重连、无效证书、帧上限 | Windows Goal 已改为完成；ADR结论和产品Connector仍单列 |
| `fi-s1-file-identity-spike` | Windows PASS | `verification-windows-20261024.md`：硬链接/Junction身份、授权根、锁、原子提交、Word锁 | Windows统一样本已改为完成；双平台ADR/产品Proposal仍待评审 |
| `e0-validate-desktop-foundation` 四边停靠 | Windows PASS | `verification-windows-idle-dock-20261025.md`、`verification-windows-edges-wake-20261029.md` | Windows四边原生子项已改为完成；双平台总项仍受macOS缺口约束 |
| `ag-s1-local-cli-mcp` | Windows Transport PASS | `verification-windows-20261024.md` | 清单原本已完成，无需复测 |
| `bu-s1-ego-lite-bridge-windows` | Windows active Runtime与真实生命周期 PASS | `verification-windows-active-runtime-20261101.md` | 已修复多版本误选并完成真实Task Space |
| `cm-s1-command-executor-spike` | Windows Spike PASS | `verification-windows.md` | 双平台已各有样本，但ADR接受/产品Proposal属于后续决策，不自动勾选 |
| `do-s2-ooxml-semantic-transform` | Windows子范围 PASS | `verification-windows.md` | 保留Story其余门禁 |
| `ag-s6-local-agent-registration` | Windows产品E2E PASS | `verification-windows-20261101.md`：UI登记/禁用/启用/撤权、Named Pipe权限、活动输入会话断开、重启持久化 | 测试任务均正式取消；测试Agent均禁用或撤权 |
| `ag-s2-local-task-registration` | Windows产品E2E PASS | `verification-windows-20261101.md`：Named Pipe命名任务创建、原生面板Agent/详情、UI取消与历史保留 | 复用同日DS-S2正式会话，避免重复副作用 |
| `ag-s3-step-declaration` | Windows真实BUA步骤链路 PASS | `verification-windows-bua-20261101.md`：声明、推进、6轮Observe、finish与34条事件 | 生产签名认证和其他Driver仍保留 |

## 本轮已完成的自动化验证（非 CUA）

| Change/Story | 缺口 | 通过所需证据 | 推进方案 |
|---|---|---|---|
| `ds-s2-running-pagination` | Windows任务总览进行中分页 | 正式Gateway创建可回收任务，原生窗口分页/空态截图和结构化结果 | **已于2026-11-01 PASS**：20个前序任务正式取消，最大ID任务真实BUA running；过滤分页与原生窗口均通过，BUA最终completed |
| `ds-s1-terminal-presentation` | Windows终态原生证据未在该Change登记 | 真实BUA success一次性展示与恢复、任务completed、空间关闭 | **成功终态已于2026-11-01映射PASS**；真实failed仍受CUA Runtime门禁并按本轮范围排除 |
| `ds-s4-mascot-pack-import` | Windows原生导入/拒绝/旧包保留 | 真实托盘、系统文件选择器、无效ZIP反馈、active tree前后hash | **已于2026-11-01 PASS**：宿主结果为invalid，原生桌宠显示拒绝反馈，资源树摘要前后一致 |
| `tm-s3-pending-cancel`、`tm-s3-cancel-retain-data`、`ds-s2-task-card-actions` | Windows created任务取消和保留数据 | 原生卡片按钮、任务cancelled、事件/详情仍存在 | **created子范围已于2026-11-01 PASS**；执行中停止、Recording与交回仍保留 |
| `tm-s5-readonly-timeline`、`tm-s5-timeline-pagination` | Windows原生时间线与分页 | 正式任务事件、原生详情、分页追加、序号无重复 | **已于2026-11-01 PASS**：20→28条，最终BUA completed |
| `tm-s2-attempt-result` | Windows Adapter合约证据缺失 | Windows真实SQLite结果事务、幂等/旧身份/unknown/Outbox回滚 | **事务/协议及真实BUA Driver已于2026-11-01 PASS**；CUA/其他Driver与Archive仍保留 |
| `tm-s6-running-state-query` | “平台验证”未拆分 | Windows真实SQLite/准入占用只读汇总 | **核心平台与桌面消费者长时原生门禁已于2026-11-01 PASS**；仅PR/Archive保留 |
| 其他任务面板项 | Windows证据原先分散 | 当前版本原生窗口、无障碍树、正式Gateway状态 | 已合并到同一次可回收原生会话并分别建立映射报告 |

## 未通过：已有实现但需要人工或硬件条件

| 项目 | 未通过原因 | 解决方案 |
|---|---|---|
| DS-S1 Windows人工拖动 | 合成原生输入技术链路已PASS，但验收明确要求用户亲手拖动，自动化不能替代 | 用户在当前Windows桌面拖动小龙并确认释放后不跟随；保存人工确认记录，现有合成证据继续作为技术证据 |
| `cx-s2-display-change-cleanup`真实模式变化/断开/副屏/负坐标 | Windows原生`WM_DISPLAYCHANGE`观察器、产品预览共享清场、资源销毁和正常退出已于2026-11-01通过合成通知验证；通用`WM_DEVICECHANGE`会被无关USB变化触发，故明确不订阅；同时修复`display-change`原因被错误降级为`close`。合成通知不能证明真实系统通知，当前环境也没有第二显示器 | 用户允许后短暂切换并finally恢复主屏模式；连接副屏后覆盖负坐标/不同DPI和拔线。证据只保存结构化清场状态，不保存桌面内容 |
| `vi-s1-voice-input-spike`显式授权/统一音频/停止释放 | 未打包候选能打开麦克风和播放公开句，但识别计数为0且`StopAsync`超时。稀疏MSIX构建PASS；当前用户签名信任以`0x800B0109`失败，官方未签名OID路线以`0x80073D2C/0x80080204`失败，均自动回滚且零残留。见两份2026-11-02验证报告与`msix-identity-registration-attempts.json` | 需要用户另行明确授权管理员/UAC，将短期测试证书公钥导入`LocalMachine\TrustedPeople`（不使用Root）并注册；先证明运行时身份，再另行授权统一语音样本，最终精确卸载包和证书 |
| `ex-s1-jev-ultrafast-spike` Windows对照 | 缺少有效`TYPESAFE_API_KEY`，只能完成自检，无法取得真实模型时延/准确率 | 提供有权限且有配额的测试Key，在Windows运行冻结样本；证据只记统计值，不记录Key或正文 |

## 未通过：产品前置或实现尚缺

| 项目 | 未通过原因 | 解决方案 |
|---|---|---|
| `ag-s5-agent-input-channel`完整Windows/macOS Goal | Windows隔离Named Pipe协议样本PASS，但Codex当前会话start/steer Adapter与产品WSS `AgentSession`尚未交付 | 先完成AG-S1认证/TLS/重连产品会话，再实现模型无关当前会话Adapter；活动turn steer、空闲start及中断后下一条输入分别取证 |
| `ag-s1-desktop-private-stdio`、`ag-s1-host-gateway-query` | 当前用户Named Pipe与细粒度注册/撤权已PASS；机器虽有多个启用本地账户，但无Yonder安装包、debug exe为`NotSigned`且当前会话非管理员，生产安装边界与跨用户ACL仍无法成立 | 先生成并签署安装包，在管理员明确安装后，由第二个Windows用户验证Named Pipe ACL、安装边界及跨用户拒绝；调试构建不能替代签名门禁 |
| `cx-s2-region-capture-spike` | Windows Graphics Capture 单显示器核心样本已于2026-11-01连续三次PASS：150% DPI下240×160逻辑区域返回360×240像素，四色、DisplayConfig、释放和不落盘通过。当前仅1块显示器，且尚未覆盖物理选择/Esc、权限拒绝/撤销与受保护窗口 | 连接副屏并配置负坐标/不同DPI复跑同一公开色块；新增显式选择生命周期探针，由用户物理圈选/Esc并执行系统撤权。完整样本通过前AD-CX-01仍只接受macOS Preview，不扩展Windows产品Port |
| `rc-s1-recording-capture-spike` | macOS两条来源路线已淘汰；Windows来源分类样本尚未建立 | 先定义Windows统一来源样本与淘汰门槛，验证真实用户、Yonder注入、Agent/Replay、安全输入、排除应用和停止后静默；不得采集正文 |
| `ds-s2-task-card-actions`完整项 | Recording与交回清除接管闸未实现，Windows证据因此不能关闭完整项 | 先完成对应Story设计/协议，再做Windows原生卡片验证；created取消子范围可先独立通过 |
| `tm-s1-agent-wait-for-user` | 进入waiting需要已Observe并停止的真实执行attempt；当前公开Gateway没有恢复/终结waiting任务的安全路径。2026-11-01复核还确认：`task.cancel`只允许created，`task.control`必须从running开始；因此不能安全回收paused/waiting。一次paused脚本错误地在step advance前请求控制，宿主重启后按架构留下`interrupted`追溯记录；对应`ego:19`已实际finish、测试Agent已disabled，无活跃空间，但不得改库伪造完成 | 先按SDD补充已审定的用户输入恢复或本地用户终结用例，再做Windows原生waiting证据；验证脚本已恢复为创建任务前安全失败的隔离模式，不能用生产数据目录制造截图 |
| `tm-s3-control-request`、`tm-s3-step-boundary-stop` | Windows定位、Recording、交回Observe及跨显示器未实现；部分依赖桌面Driver | 将纯事务/协议Windows合约与原生定位分开验证；原生Driver部分归入CUA排除队列，Recording先完成设计门禁 |
| `tm-s7-unified-execution-start` | Windows/macOS BUA统一启动已PASS；Document/Command接入仍受FI-S1、DO-S2、CM-S1产品Proposal门禁约束 | 先完成三个技术路线的ADR/产品Proposal，再接入统一启动；BUA证据不能替代其他Driver |
| `e0-validate-desktop-foundation`完整九态/生命周期/性能 | Windows listening、executing、success、四边停靠/唤醒及相关延迟已有证据；waiting、recording、failed真实来源、人工拖动和部分macOS总门禁仍缺 | 先完成对应来源Story（Recording、waiting恢复、真实失败），用户补人工拖动；再按同一版本执行九态完整原生回归，不能用静态预览代替真实事件 |
| `tm-s6-running-state-query` Archive | Windows核心查询、准入预约和桌面长时原生门禁均PASS | 完成PR隔离审阅后Archive；不再重复制造长时任务 |
| `tm-s3-cancel-retain-data`、`tm-s5-readonly-timeline` | Windows所需取消保留与时间线/分页证据已PASS，剩余项是完整Story/PR/Archive，不是新的Windows运行失败 | 完成Story级交叉审阅和PR隔离后再Archive，不重复制造任务 |

## 本轮明确排除的 CUA 项

`cu-s2-supervised-dispatch-observe`、`cu-s2-native-work-focus`、`cu-s3-background-native-action-spike`、`tm-s2-execution-attempt`的Windows CUA Driver对等验证、`tm-s2-agent-fail`真实失败终结，以及依赖真实CUA停止/定位的控制子项。TM-S2 的 Windows BUA attempt/Observe/Stopped 已于2026-11-01 PASS。其余原因是用户此前明确要求“除了 CUA”；解决方案是在用户另行恢复CUA范围后，按trycua/Qwen Spike、签名、权限和双平台原生证据门禁单独推进，不能用BUA或数据库伪造替代。

## 下一执行顺序

1. 当前环境可安全自动化且不需要CUA/外部权限的短时项已完成，并已清理全部本轮非终态测试任务。
2. TM-S6真实BUA运行超过3分钟不收起、完成后允许自然收起的长时门禁已PASS。
3. 人工拖动、物理副屏、麦克风真实采集授权、Jev Key、签名安装包/第二Windows用户需要用户或外部环境配合；语音候选探针的编译及拒绝授权闸已PASS。
4. Region Capture、Recording、Agent输入产品Adapter、Document/Command统一启动等需先完成架构/Spike/产品实现门禁；CUA继续按用户要求排除。
