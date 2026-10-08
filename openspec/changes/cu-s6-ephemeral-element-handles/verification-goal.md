# Verification Goal：CU-S6 临时 Observation transcript 与元素句柄

状态：协议 1.48、隔离 Driver/Adapter 回归与正式 QQ 音乐原图坐标样本通过；企业微信企业切换证据经复核撤回并返回实施（2026-10-08）。Windows 对等验证按主人决定暂缓，不 Archive/Done。

## 目标

通过正式 Yonder CLI → Agent Gateway → Application → Sky Worker 链路证明：协议 1.44 可把有界、经过安全过滤的当前 transcript、可操作元素句柄和 `observation_ref` 返回归属慢脑；任务持久事件不保存 transcript 或标签正文。

## 自动化结果

- `node --check crates/adapters/src/sky_cua_worker.mjs`：通过。
- `cargo run -p yonder-protocol --example generate --locked -- --check`：通过。
- `cargo test -p yonder-protocol -p yonder-application -p yonder-adapters --locked -- --test-threads=1`：155 项通过（Protocol 18、Application 59、Adapters 78）。
- `openspec validate cu-s6-ephemeral-element-handles --strict` 与 `git diff --check`：通过。

## 2026-10-08 企业微信负样本后的收紧

- `python3 apps/desktop/check-sky-cua-worker.py`：通过。隔离夹具证明同尺寸 transcript 变化会使旧坐标失效且不派发动作；可操作按钮下的装饰性 image 与 disabled 按钮不会形成候选，独立 image 仍可通过新鲜引用执行。
- macOS Sky 0.7.1 的公开接口经本机安装包核对，不提供 `screenshotId`；实现没有伪造未受支持字段，也没有对坐标猜测 Retina 比例。

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

- 此节此前把菜单/窗口内容变化当成企业切换完成，未取得“当前企业为狼顾”的直接结构事实，结论无效。
- 复核任务 `task_cdc4850bfde6b27541dd7ad4f87eda69` 明确观察到当前企业仍为“鼎捷数智”；左下角自绘入口的 AX image 与截图坐标尝试均未打开企业列表，任务在序号 118 取消。
- 该负样本证明 QQ 音乐 `(700, 780)` 坐标成功不能外推到企业微信，也证明 image 存在不等于拥有可执行 AXAction。OEH-09、OEH-13～OEH-15 保持未通过，直至正式 Yonder 链路动作后直接观察到目标企业名称。

## 结论与边界

macOS 已通过协议、隔离回归和 QQ 音乐坐标样本，但企业微信目标企业切换未通过，CU-S6 仍处于实施阶段。本 Goal 不证明 Windows 等价行为；Windows 验证继续暂缓，不能由 macOS 结果外推。

## 2026-10-08 Retina 坐标单位诊断

- 正式诊断任务 `task_8ad9767345d4224afe9655020713cac2` 经 Yonder Gateway 创建和执行；任务结束前先读取序号 72 的快照与事件，再于序号 73 取消，没有留下运行态任务。
- 企业微信个人菜单 Observation 的截图尺寸为 560×872。直接把截图像素坐标派发给 Sky 时界面无变化；将同一视觉位置按当前显示器系统 2x backing scale 转为窗口逻辑点后，动作立即打开账户详情，动作后 Observation 出现企业字段。
- 该样本只证明 Sky macOS 0.7.1 的动作坐标单位与截图像素单位不同，不证明目标企业已经切换。AD-CU-11 已修订：外部继续使用原图像素，Adapter 必须从可信窗口/显示器取得倍率后在调用边界换算，不能由慢脑猜测。
- 后续正式对照任务 `task_df83da0c773b542c78e2cf8744c5bf2f` 撤销本节结论：3420×1986 原图中精确左下角点 `(58,1929)` 经 2x 除法后动作未命中，动作后界面与 transcript 均未变化；任务于序号 42 取消。此前两个不同点位不能证明坐标单位差异。
- `python3 apps/desktop/check-sky-message-worker.py`：通过；消息引用、草稿幂等和发送 no-op 拒绝未受换算影响。
- `cargo test -p yonder-adapters --locked -- --test-threads=1`：81 项通过；Objective-C 原生桥随 Adapter 完成编译和链接。

## 2026-10-08 Sky 原图像素契约修正

- 已撤销未经有效对照证明的 backing-scale 换算及其 Objective-C 原生桥。Gateway/Agent 提交的坐标继续属于绑定 Observation 的原始截图像素空间，Adapter 校验引用、尺寸、transcript 与边界后，将 `x/y` 原样交给 Sky；窗口置前后仍刷新并绑定实际前台目标。
- `python3 apps/desktop/check-sky-cua-worker.py`：通过。隔离夹具证明原图坐标 `(30, 40)`、`(31, 41)` 等不经倍率换算即派发；过期引用、尺寸变化、transcript 变化和越界负样本仍在动作前拒绝。
- `python3 apps/desktop/check-sky-message-worker.py`：通过；消息输入、确认与发送约束未受坐标契约修正影响。
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
- `python3 apps/desktop/check-sky-cua-worker.py`：通过；夹具分别证明超时返回 `driver-action-timed-out`、窗口/通用调用错误返回 `driver-action-rejected`、成功回执但无后置变化返回 `action-unconfirmed-*`，视觉引用失效仍在派发前拒绝。
- `python3 apps/desktop/check-sky-message-worker.py`：通过；消息引用和发送安全边界未受影响。
- `cargo test -p yonder-protocol -p yonder-application -p yonder-adapters -p yonder-desktop --locked -- --test-threads=1`：211 项通过（Protocol 18、Application 71、Adapters 83、Desktop 39），覆盖 SQLite 往返、Gateway 旧协议降级及顶部浮窗中文文案。
- `cargo run -p yonder-protocol --example generate --locked -- --check`、`node --check crates/adapters/src/sky_cua_worker.mjs`、`openspec validate cu-s6-ephemeral-element-handles --strict` 与 `git diff --check`：通过。
- 本扩展完成 OEH-18、OEH-19；企业微信目标企业切换与 Windows 对等验证仍保持未通过/暂缓，Story 不 Archive/Done。

## 2026-10-07 企业切换回归

- 正式任务 `task_a889c79782cb8d776a799d6a103b9d3d` 成功取得 transcript、元素句柄并通过句柄打开头像菜单。
- 头像弹窗把入口表达为 AX selectable row，既有导出器只返回按钮，慢脑无法提交语义行点击。
- 视觉坐标点击返回 `worker-failed/unknown` 且缺少动作后 Observation；重新置前 Observe 证明未切换企业。
- 任务在序号 51 取消，未发送消息、未重放 unknown 点击。OEH-09 未通过，新增 OEH-10 后返回实施。
