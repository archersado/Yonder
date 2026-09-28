# TM-S8 产品需求

## 来源与目标

来源为产品简报“MVP 主干链路”“Task Space 与权限模型”、补充材料的 Agent Skill/本地主机 CLI/CUA-BUA Task Space，以及 2026-09-28 用户明确要求：“要的是一个可以运行 CUA/BUA/Office/Command 链路的完整产品”，并要求立即用正式链路验证。

Yonder 的验收单位是用户可见、可控、可审计的完整任务，不是单个 Driver、测试脚本或内部 API 成功。

## 验收

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

## 非目标

Yonder 不内置首次规划慢脑；内置 Jev 仅是受限执行快脑。不得新增第二 Gateway、第二任务状态机或通用本地 HTTP 服务。Windows 本轮暂缓；macOS PASS 只代表 macOS 产品闭环。
