---
title: '技术研究：vercel-labs/agent-desktop 作为 Yonder CUA 候选'
type: 'technical'
topic: 'vercel-labs/agent-desktop 作为 Yonder CUA 候选'
decision: '能否作为 Yonder Windows CUA Driver，或跨平台替代现有 trycua'
source: '公开仓库、源码、提交历史、Release、Issue与官方文档'
status: complete
preset: 'standard'
validation: 'normal'
claims_verified: 9
claims_overturned: 1
claims_unverified: 0
created: '2026-11-04'
updated: '2026-11-04'
---

# 技术研究：vercel-labs/agent-desktop 作为 Yonder CUA 候选

**本研究服务的决策：** 判断该项目能否满足 Yonder 的受监管 CUA Driver 边界，并给出采用、限时 Spike 或淘汰建议。

## 执行摘要

**结论：可以把它当作低层 Driver 候选，有条件进入限时 Spike；不能把它连同任务控制一起直接接入。** `agent-desktop` 是模型无关的 Rust 跨平台桌面自动化内核，提供 CLI、stdio MCP 和 C ABI；它不是 Agent，也不拥有规划循环。其 snapshot 限定 ref、动作前重识别、歧义拒绝、交付结果分类和 machine-wide 跨进程交互锁，适合作为 Yonder 六边形架构中的低层 Adapter 候选。[1][2][3]

它当前不满足三个硬门槛：没有用户真实键鼠输入触发立即暂停，没有执行中取消协议，也不强制每个动作后生成完整 Observe；machine-wide lease 只串行化一次 dispatch，不提供任务级所有权或 fencing。[2][3] Windows 的 raw 坐标与 foreground queue 按键路径缺少同等级别的窗口身份保护；hit-test结果不确定时会放行（fail open）。[2][3][4]

与 Sky 路线相比，它最大的战略优势是 Apache-2.0 源码可构建、可裁剪、可由 Yonder 自行签名，没有证据表明必须显示 ChatGPT/Codex 固定控制身份。[5][7] 但 v1.1.0 Windows二进制没有Authenticode，公开发布流程也未见macOS平台签名/公证步骤；缺少VC++ Runtime的干净Windows 11 ARM64已复现启动失败。项目的1.x历史只有约六天，公开提交中约九成由一人贡献。[5][8][9][10][11][17]

建议把它定位为“`agent-desktop` 引擎 + Yonder 监管包装层”，而不是让其命令模型进入 Domain/Application。Yonder仍负责任务状态、前台桌面租约的任务级 ownership、用户接管、逐步 Observe、结果 `unknown`、确认和审计；候选引擎只负责窗口/元素发现与有界动作。

## 硬门槛评估

| 门槛 | 结论 | 依据与缺口 |
|---|---|---|
| 模型无关Driver | 通过 | 官方明确它是Agent调用的工具，不包含LLM或Agent loop；Rust core与平台Adapter可通过C ABI隔离。[1] |
| Windows/macOS原生能力 | 条件通过 | Windows UIA和macOS AX均有正式构建；发布包包含两平台的x64/ARM64 CLI与FFI，但公开Issue仍有Windows窗口枚举、按键和Notepad输入缺陷报告。[6][11][15][19][20] |
| 不新增HTTP/TCP/P2P | 条件通过 | 核心CLI/C ABI/stdio不要求服务端；必须禁用可选`launch --cdp`，它会开放loopback HTTP/WebSocket端点。[13] |
| 可信目标绑定 | 条件通过 | qualified ref与动作前重识别较强；必须禁用raw `--xy`、无目标Windows `press`，并把hit-test unknown从fail-open改为拒绝。[2][3][4] |
| 单前台桌面租约 | 部分通过 | 已有machine-wide跨进程interaction lease，但只覆盖单次dispatch；任务级ownership、fencing和崩溃恢复仍由Yonder提供。[2] |
| 用户输入立即暂停 | 外部方案待Spike验证 | 未发现内建监听或takeover API；候选方案是由Yonder维护平台输入generation并在变化时停止后续派发，但能否区分合成输入与用户输入、能否停止在途动作尚未证明。[3][4] |
| 执行取消 | 队列/步间可补，在途仅能unknown | Yonder可取消未派发请求并终止隔离Worker；公开C ABI没有cooperative cancel，已投递副作用不可回滚。[16] |
| 每动作后Observe | 不通过 | 只有部分stateful动作局部回读；完整Observe由调用方编排。[3][14] |
| unknown与禁止误重试 | 条件通过 | 完整JSON envelope可区分`not_delivered`、`delivered_unverified`等结果；部分C ABI status-only调用不暴露完整disposition，Spike必须选保留完整结果的入口或补接口。[14] |
| 自有签名与品牌 | 条件通过 | Apache-2.0允许自建；v1.1.0有Sigstore provenance，Windows无Authenticode，公开macOS发布流程未见平台签名/公证，须由Yonder构建、签署并验证控制提示。[5][7][11] |
| 隐私默认值 | 不通过 | 结构化trace默认开启但会脱敏部分字段；screenshots默认关闭、启用后未脱敏，notification、snapshot、clipboard和command output也可能含私密内容。Yonder应关闭底层trace/screenshots并实施内容排除、访问控制、配额限制和清理。[18] |
| 商业分发许可 | 通过，待法务清单 | 主项目Apache-2.0；需附许可证、标明修改，并对Cargo.lock生成SBOM和逐依赖复核。[7] |
| 当前Windows干净机部署 | 不通过 | v1.1.0在缺`VCRUNTIME140.dll`的干净Windows 11 ARM64已复现失败；x64制品有相同动态导入但未见干净机直接复现。访问时修复PR尚未合入，官方二进制也未Authenticode签名。[5][10] |

## 架构与集成现实

### 可利用的部分

- Rust core将平台实现分为Windows UIA、macOS AX和Linux AT-SPI Adapter，上层有CLI与C ABI；Yonder可以固定tag后，把C ABI封装到受监管子进程而不是每命令拉起CLI。[1]
- snapshot限定ref、窗口/进程代际证据和动作前重识别比裸坐标或仅标题匹配更适合作为可信目标基础；陈旧或歧义目标会返回稳定错误。[2]
- machine-wide interaction lease能防止参与同一lease机制的`agent-desktop`进程交错输入。它不约束用户或其他自动化程序。独立核验推翻了初始研究中“只有进程内租约”的判断；剩余问题是Yonder的任务级ownership、用户接管与取消，而不是重新实现dispatch锁。[2]
- 完整JSON结果给出delivery与retry disposition，能帮助Yonder把超时/断连映射到`unknown`并禁止自动重试；部分C ABI status-only入口不保留这些字段，不能一概而论。[14]

### 不能直接复用的部分

- 工具自己的Observe→Act→Verify是调用约定，不是Driver强制不变量。Yonder必须在Application层封装“动作完成后必须Observe”，不允许外部Agent直接调用底层动作。[3][14]
- raw坐标、Windows全局按键、CDP、`--force`和底层trace不应进入产品允许集；只开放经Spike验证的ref动作与窗口操作。[3][4][12][13]
- 将C ABI直接加载到进程内会扩大宿主的故障域；Yonder应继续使用受监管Worker，使用stdio或现有私有IPC，不建立本地HTTP服务。
- 它是accessibility-first工具，不是视觉模型。对于不暴露UIA/AX的自绘界面，只能返回明确的`unsupported`结果，不得自动降级为任意坐标点击。[1][15]

### 作为纯Driver时的外部控制边界

把它当Driver使用是值得验证的方向，前提是Yonder拥有完整控制平面；以下方案仍须Spike证明：

1. **派发前抢占**：Yonder取得自己的任务级桌面租约，冻结输入generation，读取新snapshot并复核窗口PID/HWND或平台身份；generation已变化则不调用Driver。
2. **派发中监控**：候选Windows/macOS平台输入观察器只记录“用户输入generation变化”，不记录按键、文本或轨迹正文。发生用户输入时立即冻结本任务后续动作并请求Worker停止；能否可靠区分合成输入与用户输入必须实测。
3. **取消分级**：尚未派发和动作之间的取消可确定完成；已经进入Driver的动作没有cooperative cancel，只能终止隔离Worker。若副作用可能已送达，结果必须是`unknown`，不得自动重试或宣称cancelled。
4. **隔离要求**：不能把C ABI直接加载进Yonder Desktop主进程，否则无法安全终止卡住调用。应放入受监管子进程，以stdio或现有私有IPC通信；Worker退出后必须验证machine-wide lease、按键/鼠标状态和临时文件均释放，再创建新实例。
5. **动作后Observe**：正常动作完成后由Yonder强制新snapshot；Worker被终止后只允许重新Observe和交回外部Agent决策，不继续原计划片段。

外部包装可能补齐控制权，但不能回滚已经发生的物理副作用。Spike的重点是验证：用户输入后能否停止下一次派发、终止在途动作后能否释放资源，以及所有交付状态不确定的结果能否稳定记为`unknown`。

## 平台、安全与分发

Windows 10 1809+/Server 2019+和macOS 13+都有正式平台实现；Windows安全桌面、UAC、登录/锁屏和其他用户会话明确不支持，这与Yonder不控制系统安全界面的边界一致。[4] Windows向更高完整性目标输入仍需要同等提权，不能由Driver绕过。

Windows发布物目前未Authenticode签名，公开macOS发布流程未见Apple签名/公证步骤；不过Release归档和checksums有GitHub/Sigstore build provenance。该风险应表述为“供应链来源可验证，但OS平台身份尚未建立”，而不是“完全无签名”。[5][6][11]

结构化trace默认开启并脱敏部分字段；screenshots默认关闭，显式启用后产生的图片不脱敏。通知正文、snapshot、clipboard和command output也可能包含私密内容。Yonder若采用，必须在包装层关闭底层trace与screenshots，且不得让底层状态目录成为第二事实源。[18]

## 生态、许可与维护风险

项目许可证允许商业集成，原生制品的体积也较小；v1.1.0提供三平台x64/ARM64的CLI和FFI矩阵，Release和CI供应链治理优于多数同龄项目。[6][7][11]

但成熟度不足以支撑“唯一、不可替换基础设施”：仓库约八个半月，1.x只有约六天，公开提交高度集中于单一维护者；项目仅提供best-effort支持，不承诺SLA，公开资料也未发现LTS/弃用周期或逐版本SBOM。[8][9][11][12][17] Windows VCRUNTIME缺陷说明当前干净机发布验证仍有缺口。[10]

## 跨维度判断

1. **技术可塑性高于现成可用性。** 开源Rust/C ABI、Apache许可和小体积使它比固定第三方helper更适合Yonder自签名与品牌控制；同样因为安全编排留给调用方，Yonder必须投入监管包装层，不能把“命令很多”误当“产品门禁完整”。
2. **现有跨进程锁减少了实现量，但没有解除产品风险。** 它解决输入交错，不解决谁拥有桌面、用户何时接管、动作是否可取消以及任务状态如何落库。
3. **快速演进同时是优点和风险。** 开放Issue修复速度快、Release密集，但协议与平台行为仍在变化。必须固定源码revision、建立契约测试，并保留内部fork能力。

## 纠正与反证

- 初始摘要把interaction lease误判为进程内协调；独立核验确认v1.1.0已有machine-wide跨进程锁，最终结论已纠正。[2]
- “Windows工件无签名”需要拆分：它有Sigstore来源证明，但PE没有Authenticode，macOS也没有平台代码签名。[5][6]
- 支持直接采用的最强证据是：三平台正式工件、C ABI、小体积、Apache-2.0、严格ref和machine-wide锁。[1][2][6][7] 这些证据足以支持Spike，但不足以覆盖用户接管、取消、隐私和逐步Observe硬门槛。

## 建议

### R1：建立独立、限时Windows/macOS Spike

期限建议为5个工作日，固定`v1.1.0`或包含CRT修复的唯一commit，不跟随`main`。Spike只验证候选技术，不接产品Gateway、不改变任务协议、不宣布替换trycua。

统一样本至少覆盖：同名双窗口、应用重启/句柄复用、目标切前后台、用户真实键鼠接管、动作超时/Worker断连、96/192 DPI、多显示器、权限拒绝、Notepad/TextEdit与一个Electron应用。Windows和macOS使用同一验收语义。

### R2：Spike包装边界

- 使用受监管子进程 + C ABI或最小stdio桥；Yonder Application只依赖现有ComputerUse Port。
- 只开放snapshot、qualified-ref动作和经验证窗口操作；禁用raw坐标、Windows无目标按键、`--force`、CDP、底层MCP和trace/screenshots。
- 复用现有machine-wide interaction lease，同时由Yonder增加任务级fencing/ownership、用户输入暂停、cancel token、deadline和崩溃清理。
- 每次动作后由Yonder强制新Observe；任何`delivered_unverified`、Observe失败或断连都进入`unknown`且不重试。

### R3：淘汰门槛

出现任一项即停止该路线：

- 用户接管后Driver仍发送下一次输入，或无法可靠区分Driver生成的合成输入与用户输入；
- 目标窗口身份变化后出现一次误投递，而不是`STALE_REF/AMBIGUOUS_TARGET/unknown`；
- 无法在不开放本地TCP、不启用CDP的情况下完成统一样本；
- 无法由Yonder自行构建，或无法验证Windows Authenticode、macOS签名与公证的主体；
- 任一动作无法稳定产生“已Observe并验证”或“`unknown`且不重试”两类结果之一；
- 干净Windows机器仍依赖未声明运行库，或制品无法生成SBOM/第三方许可清单。

### R4：ADR结论选项

Spike通过后再写Architecture Decision，建议只在以下两项中选择：

1. `agent-desktop`作为Windows/macOS共享底层引擎，Yonder保留监管包装层；
2. `agent-desktop`仅作为Windows Adapter，macOS继续trycua，并承担双实现维护成本。

若第二项不能显著提高Windows可靠性，不应长期维护双执行栈。

## Open questions

- 维护者是否计划提供LTS、兼容周期、Windows Authenticode、macOS公证或商业支持？
- 固定版本的C ABI在真实Worker crash/cancel下能否稳定释放machine-wide lease？
- Windows与macOS能否使用同一套ref动作覆盖Yonder的目标应用，尤其Office/WPS、Electron和自绘界面？
- cursor overlay能否在不修改上游品牌资产的前提下，合规地显示Yonder身份，并满足系统级可观察性要求？
- 所有Cargo依赖的具体版本许可与MPL-2.0文件级义务能否通过法务清单？

## Source appendix

| 编号 | 支持的主张 | 发布者/来源 | 发布日期 | 访问日期 | 置信度 |
|---|---|---|---|---|---|
| [1] | Rust core、平台Adapter、CLI/C ABI、非Agent定位 | [AGENTS.md](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/AGENTS.md) | 2026-11-03 | 2026-11-04 | high |
| [2] | snapshot ref、窗口身份、machine-wide interaction lease | [v1.1.0 CONCEPTS.md](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/CONCEPTS.md) | 2026-11-03 | 2026-11-04 | high |
| [3] | ref动作、actionability与Observe责任 | [commands-interaction.md](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/skills/agent-desktop/references/commands-interaction.md) | 2026-11-03 | 2026-11-04 | high |
| [4] | Windows平台、foreground输入、安全桌面与权限边界 | [v1.1.0 Windows Skill](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/skills/agent-desktop-windows/SKILL.md) | 2026-11-03 | 2026-11-04 | high |
| [5] | Windows unsigned、SmartScreen与Smart App Control | [v1.1.0 install.md](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/docs/install.md) | 2026-11-03 | 2026-11-04 | high |
| [6] | 三平台制品、checksums与build provenance | [v1.1.0 Release](https://github.com/vercel-labs/agent-desktop/releases/tag/v1.1.0) | 2026-11-03 | 2026-11-04 | high |
| [7] | Apache-2.0许可与再分发义务 | [项目LICENSE](https://github.com/vercel-labs/agent-desktop/blob/main/LICENSE) | 动态 | 2026-11-04 | high |
| [8] | 仓库年龄 | [GitHub repository API](https://api.github.com/repos/vercel-labs/agent-desktop) | 动态至2026-11-04 | 2026-11-04 | high |
| [9] | Release历史 | [Releases](https://github.com/vercel-labs/agent-desktop/releases) | 至2026-11-03 | 2026-11-04 | high |
| [10] | Windows VCRUNTIME140发布缺陷与修复状态 | [Issue #542](https://github.com/vercel-labs/agent-desktop/issues/542) | 2026-11-04 | 2026-11-04 | high |
| [11] | 多平台构建、Release矩阵与公开签名流程 | [release.yml](https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/.github/workflows/release.yml) | 至2026-11-03 | 2026-11-04 | high |
| [12] | best-effort无SLA | [SUPPORT.md](https://github.com/vercel-labs/agent-desktop/blob/main/SUPPORT.md) | 至2026-11-03 | 2026-11-04 | high |
| [13] | 可选CDP loopback HTTP/WebSocket端点 | [v1.1.0 commands-system.md](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/skills/agent-desktop/references/commands-system.md) | 2026-11-03 | 2026-11-04 | high |
| [14] | delivery/retry disposition、部分stateful回读与C ABI暴露差异 | [v1.1.0 json-output.md](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/docs/json-output.md) | 2026-11-03 | 2026-11-04 | high |
| [15] | Windows窗口枚举开放问题 | [Issue #478](https://github.com/vercel-labs/agent-desktop/issues/478) | 2026-11-02 | 2026-11-04 | medium（用户报告） |
| [16] | C ABI公开接口中未发现cooperative cancel | [C ABI header](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/crates/ffi/include/agent_desktop.h) | 2026-11-03 | 2026-11-04 | medium（公开API缺席性结论） |
| [17] | 公开提交贡献集中度 | [contributors API](https://api.github.com/repos/vercel-labs/agent-desktop/contributors?per_page=100&anon=1) | 动态至2026-11-04 | 2026-11-04 | high |
| [18] | trace、screenshots与敏感数据边界 | [commands-system.md](https://github.com/vercel-labs/agent-desktop/blob/v1.1.0/skills/agent-desktop/references/commands-system.md) | 2026-11-03 | 2026-11-04 | high |
| [19] | Windows按键开放问题 | [Issue #513](https://github.com/vercel-labs/agent-desktop/issues/513) | 2026-11-03 | 2026-11-04 | medium（用户报告） |
| [20] | Windows Notepad输入开放问题 | [Issue #459](https://github.com/vercel-labs/agent-desktop/issues/459) | 2026-11-01 | 2026-11-04 | medium（用户报告） |

## Staleness map

由研究账本和技术研究新鲜度窗口计算：

- 2026-12-01前复查：Windows/macOS签名状态、Windows VCRUNTIME修复、ref/raw输入兼容性。
- 2027-05-01前复查：维护者结构、Release稳定性与SLA/LTS状态。
- 2027-08-01前复查：CDP与本地网络能力边界。
- 2027-11-01前复查：interaction lease、用户接管与取消语义。
- 架构边界最迟2028-11-01复查；许可证文本除上游变更外按稳定事实处理。

最早刷新日期为 **2026-12-01**。执行Spike或ADR前若已超过该日期，应先运行Refresh。
