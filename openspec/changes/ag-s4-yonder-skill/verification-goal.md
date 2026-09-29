# AG-S4 Yonder Agent Skill 独立 Verification Goal

状态：结构与规格检查 PASS；本机安装、真实慢脑生成及四类产品 Runtime 样本待验证。Windows按用户决定暂缓，Change不Archive。

## Goal

证明发布包可被Agent发现，并能在不建立旁路的前提下把一个用户目标编排为一个Yonder任务；CUA慢脑生成完整多步计划片段，Yonder/Jev在片段内连续Observe，仅在元素语义路径不可用时使用视觉兜底。

## 当前证据（2026-09-29）

| 检查 | 结果 | 证据 |
|---|---|---|
| Skill结构与frontmatter | PASS | 官方`quick_validate.py skills/yonder`返回`Skill is valid!` |
| manifest JSON | PASS | `python3 -m json.tool skills/yonder/manifest.json` |
| Agent UI元数据 | PASS | Ruby YAML解析通过；默认提示显式引用`$yonder` |
| OpenSpec | PASS | `openspec validate ag-s4-yonder-skill --strict` |
| 包边界 | PASS | 包内只有Markdown、JSON、YAML；无执行代码、凭据、用户数据或协议类型副本 |
| 架构门禁 | BASELINE BLOCKED | `scripts/check_architecture.py`因既有`TM-S9` README缺少唯一`Story:`字段失败；同一失败可在未修改的`dev@b34fd1d`复现，与本Change无关 |

## 企业微信前向样本

输入：使用Yonder打开企业微信，找到“宫健的分身”，准备消息“hi”，取得发送确认后发送；应用无需切前台，元素不可用时才使用视觉。

必须满足：

1. 只创建一个Yonder任务并复用同一`task_id`。
2. 慢脑一次提交覆盖“启动/恢复应用、解析会话、准备草稿、确认后发送、验证结果”的多槽位片段，不提交单动作片段循环。
3. 每槽位候选完成同一语义步骤；元素或原生语义候选在前，视觉候选仅作该槽位兜底。
4. 草稿准备不要求发送确认；真正发送候选必须引用Yonder一次性确认。
5. 每步由Yonder/Jev Observe并继续；候选耗尽、预期不满足或`unknown`才交回慢脑。
6. `unknown`发送不重试，只读取`task_get/task.events`并等待核实。

## 待验证

- 将合并后的`skills/yonder/`安装至本机Agent Skill搜索路径并验证可发现。
- 由新的Agent会话加载Skill，检查上述企业微信样本的真实计划输出。
- 复用TM-S8正式Yonder GUI分别完成BUA、CUA、Document、Command样本；不得用私有脚本或Driver探针替代。
- Windows恢复后补对等安装、发现与原生能力证据。
