# Verification Goal：CU-S6 临时 Observation transcript 与元素句柄

状态：协议 1.48、隔离 Driver/Adapter 回归、正式 QQ 音乐原图坐标样本与企业微信企业切换回归通过（2026-10-08）。Windows 对等验证按主人决定暂缓，不 Archive/Done。

## 目标

通过正式 Yonder CLI → Agent Gateway → Application → Sky Worker 链路证明：协议 1.44 可把有界、经过安全过滤的当前 transcript、可操作元素句柄和 `observation_ref` 返回归属慢脑；任务持久事件不保存 transcript 或标签正文。

## 自动化结果

- `node --check crates/adapters/src/sky_cua_worker.mjs`：通过。
- `cargo run -p yonder-protocol --example generate --locked -- --check`：通过。
- `cargo test -p yonder-protocol -p yonder-application -p yonder-adapters --locked -- --test-threads=1`：155 项通过（Protocol 18、Application 59、Adapters 78）。
- `openspec validate cu-s6-ephemeral-element-handles --strict` 与 `git diff --check`：通过。

## 2026-10-08 Observation 绑定坐标扩展

- Rust 协议升级至 1.48，并从唯一来源重新生成 TypeScript 与 Response Schema；含截图的 Observation 以原子字段返回原始像素宽高，坐标候选必须携带同次 `observation_ref`。
- `python3 apps/desktop/check-sky-cua-worker.py`：通过。隔离 Sky MCP 夹具从 PNG 字节头读取 100×100 原图尺寸；有效坐标只派发一次；过期引用、`x == width` 越界和动作前截图变为 120×120 三类负样本均未派发动作，并携带新鲜 Observation 以 `unverifiable` 交回。
- `cargo test -p yonder-protocol`：18 项通过；`cargo test -p yonder-application`：70 项通过；`cargo test -p yonder-adapters`：81 项通过。
- `cargo test --locked -p yonder-desktop --lib`：27 项通过；发布契约与 Gateway 最高协议同步到 1.48。
- `openspec validate cu-s6-ephemeral-element-handles --strict`、`node --check crates/adapters/src/sky_cua_worker.mjs` 与 `git diff --check`：通过。
- 架构脚本在本变更文档章节校验通过后，被 dev 既有 `TM-S9/README.md` 缺少机器可读 `Story/Epic/Status/OpenSpec` 字段阻断；该文件不属于本 Story，本次未越界修改。
- dev 正式 `Yonda.app` 已重新构建并启动；重启后的 Yonder stdio MCP 已由宿主重连，全程未改走 Local Socket、CLI 或 Driver。

## 2026-10-08 QQ 音乐正式原图坐标样本

- 正式任务 `task_847a13519eaf417314f7f12ea6314e4e` 由 Yonder MCP 创建；慢脑一次提交包含启动、聚焦、输入和搜索的四槽计划片段。输入动作按既有安全规则以新鲜 Observation 交回，而不是将 Driver 回执直接提升为成功。
- 交回 Observation 返回 `screenshot_width_px=2560`、`screenshot_height_px=1748` 与 `observation_attempt_14`；慢脑从该原图像素空间选择第一条目标歌曲的命中点，不使用对话缩放坐标或桌面窗口偏移。
- 计划版本 2 提交绑定同次引用的 `(700, 780)` 双击候选。Driver 只派发一次，动作后 Observation `observation_attempt_19` 保持相同原图尺寸，并以“暂停播放”控件和当前歌曲/歌手结构事实证明播放已开始。
- 任务在 Observation 序号 20 完成目标核验，并于序号 24 进入 `completed`。该样本通过 OEH-10～OEH-12；截图与 transcript 正文未写入持久验证证据。

## macOS 正式产品样本

- 构建并打包 `Yonda.app`，由正式 CLI MCP 入口创建任务 `task_debd427edc6b0ece3802b95f49762a8c`。
- 经 `task_plan_submit` 与 `task_plan_execute` 启动企业微信；返回协议 1.44 Observation，包含 `element_count=260`、30 个有界可操作句柄、非空 `observation_ref` 和非空 transcript。
- transcript 与标签正文不写入本文件。随后读取 `task.events(after_sequence=0)`，序号 1～7 只包含创建、步骤、attempt 与状态事实，不含 Observation transcript、元素标签或截图正文。
- 使用 Observation 序号 5 提交目标核验，任务在序号 9 正常完成；证明临时响应不阻塞既有 Goal/完成链路。

## 2026-10-08 企业微信企业切换正式回归

- 正式任务 `task_3f654d71def2a3ffed138056fb7c4937` 经 Yonder MCP 创建并由 Agent Gateway 管理；未绕过到 Local Socket、Driver 或系统自动化脚本。
- 初始 Observation 中企业入口没有文本标签。旧预览坐标候选未产生有效 Observe 后，慢脑没有重放旧动作，而是读取 `task.get` 与 `task.events`，基于新鲜 Observation 重新规划。
- 计划版本 3 使用 `observation_attempt_10` 的 image 句柄打开个人菜单；动作结果为 `action_succeeded=true`、`observe_valid=true`。新鲜菜单 Observation 随后导出当前企业按钮，计划版本 4 使用绑定 `observation_attempt_15` 的元素句柄完成企业切换。
- 切换后 Observation 中窗口内容由原企业会话集变为目标企业会话集；计划版本 5 再次执行无副作用的应用观察，得到 `observation_attempt_27`，其动作结果仍为 `action_succeeded=true`、`observe_valid=true`，并进入 `awaiting-goal-verification`。
- 目标由 Observation 序号 28 核验为 `achieved`，任务在序号 32 进入 `completed`。本样本没有输入或发送消息；截图、transcript 与会话正文未写入持久验证证据。

## 结论与边界

macOS 子范围通过 OEH-01～OEH-12；正式 QQ 音乐与企业微信样本证明慢脑可用新鲜 transcript、元素/image 句柄和 Observation 绑定坐标完成消歧、动作后 Observe、交回重规划及目标核验。本 Goal 不证明 Windows 等价行为；Windows 验证继续暂缓，不能由本结果外推。

## 2026-10-07 企业切换回归

- 正式任务 `task_a889c79782cb8d776a799d6a103b9d3d` 成功取得 transcript、元素句柄并通过句柄打开头像菜单。
- 头像弹窗把入口表达为 AX selectable row，既有导出器只返回按钮，慢脑无法提交语义行点击。
- 视觉坐标点击返回 `worker-failed/unknown` 且缺少动作后 Observation；重新置前 Observe 证明未切换企业。
- 任务在序号 51 取消，未发送消息、未重放 unknown 点击。OEH-09 未通过，新增 OEH-10 后返回实施。
