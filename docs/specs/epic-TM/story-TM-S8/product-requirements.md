# TM-S8 产品需求

## 问题与目标

来源为产品简报“MVP 主干链路”“Task Space 与权限模型”、补充材料的 Agent Skill/本地主机 CLI/CUA-BUA Task Space，以及 2026-09-28 用户明确要求：“要的是一个可以运行 CUA/BUA/Office/Command 链路的完整产品”，并要求立即用正式链路验证。

Yonder 的验收单位是用户可见、可控、可审计的完整任务，不是单个 Driver、测试脚本或内部 API 成功。

## 范围与非目标

范围覆盖真实 Codex MCP、正式 Yonder GUI、统一 Gateway、Jev 片段内选择，以及 CUA、BUA、Office、Command 四类产品闭环。Yonder 不内置首次规划慢脑；不得新增第二 Gateway、第二任务状态机或通用本地 HTTP 服务。Windows 本轮暂缓，macOS PASS 不代表双平台完成。

## 验收条件

| ID | 要求 |
|---|---|
| TM8-01 | 真实 Codex 通过安装包内 `yonder mcp` 和私有 Gateway 创建任务；任务立即出现在当前 Yonder Task Space。 |
| TM8-02 | CUA 任务展示计划与当前真实步骤，控制条从首次桌面控制持续到任务终态；普通输入不停止，发送前本机确认。 |
| TM8-03 | BUA 任务只复用 Yonder 关联的 ego-lite Browser Task Space，完成 Observe、交回与终态，不复制浏览器 UI。 |
| TM8-04 | Office 任务通过本机文件授权、Document Port、文件锁、expected_hash、默认另存和结果审阅完成，不向 Agent 暴露路径或 XML。 |
| TM8-05 | Command 任务采用结构化 program/args/cwd/env，Yonder 本机预览批准后执行，输出有界，Shell、提权或危险操作不得隐式放行。 |
| TM8-06 | 四类任务共用 task.create、统一 running 启动、步骤/attempt、事件/Outbox、Task Space 与 task.complete/fail；失败不得伪报成功。 |
| TM8-07 | 验证必须来自 `dev` 构建的正式 GUI 和产品 MCP；证据包含无敏感正文的结构化结果与必要原生截图。 |
| TM8-08 | Codex 慢脑负责首次计划和片段外语义 replan；Jev 快脑只在慢脑提交的已验证片段内做候选选择并连续执行，低置信、偏离、预算耗尽或敏感动作经同一 Gateway 交回 Codex。 |
| TM8-09 | 验证至少包含一条多候选步骤，证明 Jev 实际参与选择；同时记录快慢脑分段耗时，与纯 Codex 逐步决策对照，不得用单一路径免 Jev 样本宣称快慢脑链路通过。 |
| TM8-10 | `task.plan.execute` 开始后任务必须与控制条同步为执行中；Jev 在首个动作前交回时仍保持同一 `running` 任务并向慢脑展示下一意图。 |
| TM8-11 | 正式用户库的任务快照与事件必须可读；标称新版本但审计表全缺时只允许无数据覆盖的确定性结构修复，部分缺失须拒绝启动。 |
| TM8-12 | CUA、BUA、Office/Document、Command 及后续统一 Gateway 执行能力开始执行时，均在屏幕顶部打开统一执行浮窗，展示可信当前步骤和可用的规划步骤；非 CUA 不显示“控制电脑”或接管按钮，终态后关闭。 |
| TM8-13 | 对同一批可复位 CUA 任务，以正式 Codex MCP 和正式 Yonder GUI 做“慢脑计划片段 + Jev 快脑”与“纯慢脑逐步决策”的配对对照。至少分别报告目标完成率、端到端主动耗时、慢脑调用次数、交回次数和 `unknown`；用户确认等待单独计时。不得以单次成功、Driver 回执或不同任务的耗时推断快慢脑更优。 |

## 快慢脑效果对照口径

本 Story 的性能结论只覆盖当前已具备两种正式产品执行模式的 CUA。首轮使用三类不产生对外副作用、可恢复初态的任务：确定性本地编辑、动态应用内检索、元素缺失后的 Observe/恢复。每类先完成 3 对预跑，修复测量或产品缺陷后再完成至少 10 个有效配对；同一 `pair_id` 的两种模式必须使用同一目标、相同初态和相同完成判据，并交替执行顺序。

“快慢脑”由慢脑一次提交 2～10 个槽位的完整片段，多候选槽位真实调用 Jev；“纯慢脑”由同一 Codex 在每次 Observe 后经 Gateway 提交下一动作，禁止私有 Driver 或脚本代替决策。成功只接受最新 Observation 绑定的 `task.goal.verify=achieved` 和任务终态 `completed`。超时、`unknown`、错误目标、需要人工修正或未达到目标均计为未完成，不从分母剔除。

计时从首个计划/动作请求发出，到目标核验提交为止；报告墙钟耗时和扣除显式用户等待后的主动耗时。预热与构建时间不计入，但不得排除 Jev 请求、Gateway、Driver、Observe、handback/replan。主结果报告每种模式的完成数/总数及 Wilson 95% 区间、主动耗时中位数/P95；只对两边均完成的配对另报耗时比，不能用它替代总体完成率。
