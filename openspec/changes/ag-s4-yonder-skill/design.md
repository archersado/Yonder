# Design：统一 Agent Skill

Skill 发布源为 `skills/yonder/`。`SKILL.md`只保留共同生命周期、模块路由、计划片段和数据边界；`references/`按需承载Browser、Computer、Document、Command规则；`manifest.json`声明包版本、最低协议、验证平台、基础工具、可选能力工具与ego-browser上游版本；`agents/openai.yaml`提供发现元数据。

Agent先从当前MCP会话发现工具，再创建唯一任务。版本只构成下限，工具发现才决定具体能力是否可用。所有状态变更携带Yonder返回的最新sequence；CAS冲突、handback、unknown或断连先读取`task_get/task_events`，不直接访问SQLite或Socket。

CUA优先使用协议1.31计划片段：慢脑一次给出当前有界子目标的顺序槽位，每槽位候选完成同一语义步骤；Yonder/Jev在片段内连续选择、派发、Observe。元素或原生语义路径优先，视觉只在元素不可用时兜底；发送等副作用必须绑定一次性确认。计划能力缺失时可使用正式`computer_step`兼容路径，但不得直接调用Driver。

BUA先由Yonder创建和登记ego-lite引用，再按受控ego-browser规则操作同一Task Space；Document仅使用任务文件授权和Document Gateway；Command保持提议—本机批准—一次执行。四类模块都禁止旁路。
