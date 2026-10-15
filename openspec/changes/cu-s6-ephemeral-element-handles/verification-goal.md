# Verification Goal：CU-S6 临时 Observation transcript 与元素句柄

状态：协议 1.48、隔离 Driver/Adapter 回归与正式 QQ 音乐原图坐标样本通过；企业微信企业切换证据经复核撤回并返回实施（2026-10-08）。Windows 对等验证按主人决定暂缓，不 Archive/Done。

## 目标

通过正式 Yonder CLI → Agent Gateway → Application → Sky Worker 链路证明：协议 1.44 可把有界、经过安全过滤的当前 transcript、可操作元素句柄和 `observation_ref` 返回归属慢脑；任务持久事件不保存 transcript 或标签正文。

## 自动化结果

- `node --check crates/adapters/src/sky_cua_worker.mjs`：通过。
- `cargo run -p yonder-protocol --example generate --locked -- --check`：通过。
- `cargo test -p yonder-protocol -p yonder-application -p yonder-adapters --locked -- --test-threads=1`：155 项通过（Protocol 18、Application 59、Adapters 78）。
- `openspec validate cu-s6-ephemeral-element-handles --strict` 与 `git diff --check`：通过。

## 2026-10-09 元素动作后 Observe 收敛

- Sky 元素点击仍只派发一次；首次动作后结构状态未变化时，Worker 在 900ms 内按
  100ms 间隔只读 Observe，取得既有结构后置事实即停止。坐标动作与结果待核实动作
  不进入该循环，期限耗尽仍按 unknown/handback 处理。
- `python3 scripts/verification/desktop/check-sky-cua-worker.py`：通过。延迟状态夹具在后续 Observe
  才出现变化，动作日志与调用方提交步骤一一对应，没有新增点击。
- `python3 scripts/verification/desktop/check-sky-message-worker.py`：通过；消息草稿、确认与发送 no-op
  边界未受影响。
- `cargo test --offline --locked -p yonder-adapters`：85 项通过；`cargo test
  --offline --locked -p yonder-desktop --lib`：29 项通过。
- `openspec validate cu-s6-ephemeral-element-handles --strict`、架构检查、Node/Python
  语法检查与 `git diff --check` 均通过。正式 QQ 音乐播放正向样本仍需重建产品后复验。

## 2026-10-09 可证实 no-op 的受限恢复

- Accepted AD-CU-12 修订把动作调用正常返回、动作后 Observe 有效、完整 transcript
  未变化且同一角色/标签目标仍唯一存在的非敏感元素激活定义为受限恢复边界。
  Worker 不复用旧数字索引，而是按新鲜角色/标签重新解析，并最多追加一次动作；
  坐标、输入、发送、确认与一次性引用动作仍不得自动重试。
- `python3 scripts/verification/desktop/check-sky-cua-worker.py`：通过。夹具把“可恢复控件”从索引
  40 漂移到 42；Worker 两次都使用新索引 42，第二次出现结构变化并确认成功。
  “无响应控件”恰好派发两次后返回 `action-unconfirmed-*`，证明不会无限循环。
  坐标超时/拒绝和未确认文本输入的既有零重试断言继续通过。
- `python3 scripts/verification/desktop/check-sky-message-worker.py`、Adapters 85 项、Desktop 29 项、
  OpenSpec strict、架构检查与 Node 语法检查全部通过。实现提交 `41e3f46` 已
  ff-only 合并 `dev`，重新构建正式 `Yonda.app`，宿主 PID 为 93986。
- 正式 Gateway 任务 `task_eced437b5a1b88c84db64e592f05a94d` 取得 QQ 音乐
  新鲜播放按钮后执行受限恢复；两次 AX 元素激活后按钮仍为“播放”，最终按
  `driver-action-unconfirmed` 交回。慢脑没有结束任务，而是继续提交“置前后 SPACE”
  替代片段；置前动作一度 `observe-failed`，只读重新 Observe 恢复后，SPACE 仍以
  `driver-action-unconfirmed` 交回。任务于序号 27 取消。
- 因此 OEH-21、OEH-22 的恢复与耗尽边界通过，但 QQ 音乐正向播放仍被 Sky 输入
  未生效阻断；本 Goal 不把“发生过重试”提升为业务目标成功，正式播放任务保持未通过。

## 2026-10-08 企业微信负样本后的收紧

- `python3 scripts/verification/desktop/check-sky-cua-worker.py`：通过。隔离夹具证明同尺寸 transcript 变化会使旧坐标失效且不派发动作；可操作按钮下的装饰性 image 与 disabled 按钮不会形成候选，独立 image 仍可通过新鲜引用执行。
- macOS Sky 0.7.1 的公开接口经本机安装包核对，不提供 `screenshotId`；实现没有伪造未受支持字段，也没有对坐标猜测 Retina 比例。

## 2026-10-08 Observation 绑定坐标扩展

- Rust 协议升级至 1.48，并从唯一来源重新生成 TypeScript 与 Response Schema；含截图的 Observation 以原子字段返回原始像素宽高，坐标候选必须携带同次 `observation_ref`。
- `python3 scripts/verification/desktop/check-sky-cua-worker.py`：通过。隔离 Sky MCP 夹具从 PNG 字节头读取 100×100 原图尺寸；有效坐标只派发一次；过期引用、`x == width` 越界和动作前截图变为 120×120 三类负样本均未派发动作，并携带新鲜 Observation 以 `unverifiable` 交回。
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

- 此节此前把菜单/窗口内容变化当成企业切换完成，未取得“当前企业为狼顾”的直接结构事实，结论无效。
- 复核任务 `task_cdc4850bfde6b27541dd7ad4f87eda69` 明确观察到当前企业仍为“鼎捷数智”；左下角自绘入口的 AX image 与截图坐标尝试均未打开企业列表，任务在序号 118 取消。
- 该负样本证明 QQ 音乐 `(700, 780)` 坐标成功不能外推到企业微信，也证明 image 存在不等于拥有可执行 AXAction。OEH-09、OEH-13～OEH-15 保持未通过，直至正式 Yonder 链路动作后直接观察到目标企业名称。

## 结论与边界

macOS 已通过协议、隔离回归和 QQ 音乐坐标样本，但企业微信目标企业切换未通过，CU-S6 仍处于实施阶段。本 Goal 不证明 Windows 等价行为；Windows 验证继续暂缓，不能由 macOS 结果外推。

## 2026-10-13 企业微信新鲜句柄与 Retina 对照复验

- 正式任务 `task_17b1aea068b54dff0d06e1b3337341d9` 由 `Yonda.app → MCP stdio → Agent Gateway → Application → Sky Worker` 创建并执行；启动后的新鲜 Observation 含 336 个 AX 元素、`3420×1986` 原图和临时引用。
- 历史入口索引不能复用：当前新鲜树中的 image 18、image 19 与唯一无标签 button 20 均分别绑定各自的新鲜 Observation 执行，动作后完整树保持 336 个元素，企业列表未出现，Yonder 均交回慢脑而未伪报成功。
- 原图左下角视觉点绑定新鲜引用后，Sky 返回 `driver-action-rejected`；同位置的 2× 逻辑点仅作为一次诊断对照，动作后结构仍无变化。该结果既不支持原图像素路径，也不能把固定 2× 换算提升为产品规则。
- Codex CUA 只读绑定同一窗口后，对原图点与 2× 逻辑点分别做单次对照，两次均由同一 Computer Use 服务返回 `-10005 noWindowsAvailable`；因此不存在“Codex 可投递、Yonder 映射错误”的反例，阻断位于当前 Sky/企业微信窗口坐标投递能力。
- 任务读取最新 `task.get/events` 后在序号 32 正式取消；没有自动重放 unknown 坐标，没有选择企业或产生消息副作用。
- 当前阻断收敛为 Sky 对企业微信自绘左栏缺少可确认的投递路径，而非计划缺少上下文、旧索引误用或 Goal 核验提前成功。OEH-09、OEH-13～OEH-17 与两项正式回归任务继续保持未完成。
- 有界证据：`apps/desktop/evidence/cu-s6-wecom-retina-negative-20261013/result.json`；不保存 transcript、截图、标签正文或完整 Payload。

## 2026-10-12 QQ 音乐播放按钮目标确认回归

- 修复前正式任务 `task_04cb1b11f7cf69eabc9db994ebd88d47` 直接激活新鲜“暂停播放”按钮；动作后按钮标签保持不变，但页面其他结构变化使旧 Worker 错误产生 `observed(action_succeeded=true)`。归属慢脑依据按钮事实提交 `not-achieved`，任务取消，未把 Driver 回执提升为业务成功。
- Accepted AD-CU-12、Story 与 OpenSpec 已补充目标相关确认：按钮只在原角色/标签目标消失时确认结构效果；目标仍存在但其他结构变化时，禁止确认成功，也不得以不可归因变化触发恢复重放。
- 修复后重新构建正式 `Yonda.app`。任务 `task_78ac5b1e03f22dfec3356b8aafeae3b9` 从 Observation 序号 5 取得“播放”按钮及新鲜引用，直接激活该句柄；动作后序号 11 明确为 `unknown/driver-action-unconfirmed`，序号 12 交回慢脑，按钮仍为“播放”，没有再次计划或自动重试不可核实结果。任务于序号 13 正式取消。
- 隔离 Worker 夹具新增“目标按钮不变、无关 transcript 变化”样本，断言只派发一次且返回未确认；目标明确保持原状态且完整结构不变时仍按既有规则至多恢复一次。`python3 scripts/verification/desktop/check-sky-cua-worker.py` PASS；Adapters 86/86（单线程）、Desktop 33/33、OpenSpec strict 与架构检查通过。
- 本样本满足本 Goal 的“播放按钮切换为暂停或安全交回”判据，并关闭播放按钮回归项；它不解决企业微信企业切换，也不外推 Windows。

有界证据见 `apps/desktop/evidence/cu-s6-qqmusic-button-target-confirmation-20261012/`，不保存 transcript、截图、完整 Payload 或用户数据。

## 2026-10-08 Retina 坐标单位诊断

- 正式诊断任务 `task_8ad9767345d4224afe9655020713cac2` 经 Yonder Gateway 创建和执行；任务结束前先读取序号 72 的快照与事件，再于序号 73 取消，没有留下运行态任务。
- 企业微信个人菜单 Observation 的截图尺寸为 560×872。直接把截图像素坐标派发给 Sky 时界面无变化；将同一视觉位置按当前显示器系统 2x backing scale 转为窗口逻辑点后，动作立即打开账户详情，动作后 Observation 出现企业字段。
- 该样本只证明 Sky macOS 0.7.1 的动作坐标单位与截图像素单位不同，不证明目标企业已经切换。AD-CU-11 已修订：外部继续使用原图像素，Adapter 必须从可信窗口/显示器取得倍率后在调用边界换算，不能由慢脑猜测。
- 后续正式对照任务 `task_df83da0c773b542c78e2cf8744c5bf2f` 撤销本节结论：3420×1986 原图中精确左下角点 `(58,1929)` 经 2x 除法后动作未命中，动作后界面与 transcript 均未变化；任务于序号 42 取消。此前两个不同点位不能证明坐标单位差异。
- `python3 scripts/verification/desktop/check-sky-message-worker.py`：通过；消息引用、草稿幂等和发送 no-op 拒绝未受换算影响。
- `cargo test -p yonder-adapters --locked -- --test-threads=1`：81 项通过；Objective-C 原生桥随 Adapter 完成编译和链接。

## 2026-10-08 Sky 原图像素契约修正

- 已撤销未经有效对照证明的 backing-scale 换算及其 Objective-C 原生桥。Gateway/Agent 提交的坐标继续属于绑定 Observation 的原始截图像素空间，Adapter 校验引用、尺寸、transcript 与边界后，将 `x/y` 原样交给 Sky；窗口置前后仍刷新并绑定实际前台目标。
- `python3 scripts/verification/desktop/check-sky-cua-worker.py`：通过。隔离夹具证明原图坐标 `(30, 40)`、`(31, 41)` 等不经倍率换算即派发；过期引用、尺寸变化、transcript 变化和越界负样本仍在动作前拒绝。
- `python3 scripts/verification/desktop/check-sky-message-worker.py`：通过；消息输入、确认与发送约束未受坐标契约修正影响。
- `cargo test -p yonder-adapters --locked -- --test-threads=1`：81 项通过；`node --check crates/adapters/src/sky_cua_worker.mjs` 与 `git diff --check`：通过。
- 企业微信目标企业切换仍须在重新构建的正式 Yonder 产品链路中重跑；在动作后直接观察到“狼顾”之前，本节不提升 Story 状态。

## 2026-10-08 重建后企业微信正式负样本

- `dev` 已包含 `e5bd5c2`、`04b19fc`，重新构建并启动正式 `Yonda.app`；宿主重连后由 Yonder MCP 创建任务 `task_00fb7b644e67d92c30badc5a20ab50f2`，未改走 CLI、Local Socket 或独立 Driver。
- 首次片段启动企业微信并取得 `3420×1986` 原图与 `observation_attempt_4`。慢脑依据原图裁切复核左下角图标中心为 `(58, 1929)`，提交绑定该引用的坐标点击；动作后 `observation_attempt_11` 的截图与 transcript 均未变化，Yonder 以 `suspected_noop/observe-failed` 交回，未自动重放。
- 新鲜 Observation 上改用 AX 图像句柄验证执行链路：索引 19 成功打开个人菜单，并返回 `560×872` 的动作后 Observation；`bring-to-front` 随后重新绑定主窗口且验证通过。索引 2 没有产生界面变化。由此可排除 Gateway、WindowActivationPort、Sky AX 点击和动作后 Observe 全链路失效。
- 当前阻断收敛为：企业微信左下角自绘企业切换入口既没有可区分标签/几何的临时句柄，原图坐标调用也未被 Driver 证明确认。继续枚举匿名 AX 图像或重放结果待核实坐标不满足安全边界，因此任务在序号 32 取消；OEH-09、OEH-13～OEH-15 仍未通过。

## 2026-10-08 有界 Driver 错误透出

- Accepted AD-CU-12 与协议 1.49 增加六个封闭 Attempt Unknown Reason：动作超时、动作拒绝、动作未确认、目标失效、Observe 失败与传输失败。协议 1.48 及以下分别降级为既有 `timed-out`、`observe-failed` 或 `worker-failed`。
- Sky Worker 坐标动作错误继续执行唯一一次动作后 Observe，并只返回安全 stage；SDK/MCP 原文、动作参数、应用名、AX transcript、截图、PID、窗口 ID 与完整 Payload 均不进入事件或界面。
- `python3 scripts/verification/desktop/check-sky-cua-worker.py`：通过；夹具分别证明超时返回 `driver-action-timed-out`、窗口/通用调用错误返回 `driver-action-rejected`、成功回执但无后置变化返回 `action-unconfirmed-*`，视觉引用失效仍在派发前拒绝。
- `python3 scripts/verification/desktop/check-sky-message-worker.py`：通过；消息引用和发送安全边界未受影响。
- `cargo test -p yonder-protocol -p yonder-application -p yonder-adapters -p yonder-desktop --locked -- --test-threads=1`：211 项通过（Protocol 18、Application 71、Adapters 83、Desktop 39），覆盖 SQLite 往返、Gateway 旧协议降级及顶部浮窗中文文案。
- `cargo run -p yonder-protocol --example generate --locked -- --check`、`node --check crates/adapters/src/sky_cua_worker.mjs`、`openspec validate cu-s6-ephemeral-element-handles --strict` 与 `git diff --check`：通过。
- 本扩展完成 OEH-18、OEH-19；企业微信目标企业切换与 Windows 对等验证仍保持未通过/暂缓，Story 不 Archive/Done。

## 2026-10-07 企业切换回归

- 正式任务 `task_a889c79782cb8d776a799d6a103b9d3d` 成功取得 transcript、元素句柄并通过句柄打开头像菜单。
- 头像弹窗把入口表达为 AX selectable row，既有导出器只返回按钮，慢脑无法提交语义行点击。
- 视觉坐标点击返回 `worker-failed/unknown` 且缺少动作后 Observation；重新置前 Observe 证明未切换企业。
- 任务在序号 51 取消，未发送消息、未重放 unknown 点击。OEH-09 未通过，新增 OEH-10 后返回实施。
