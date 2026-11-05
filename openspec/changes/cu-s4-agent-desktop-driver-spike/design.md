# 设计

## 固定输入

Spike 固定 `agent-desktop` commit `d3dc190afbf34cd2377252398ddd5c30efbd093a`（v1.1.0），不得因 CRT 缺陷漂移到后续源码。Windows 固定静态 CRT 构建并在干净机验证；失败即淘汰。首次原生运行前生成 build manifest，记录上游 SHA、空或唯一 patch hash、Cargo.lock hash、Rust toolchain、target triple、features、CRT/链接策略、bridge/evidence schema、Yonder harness commit/dirty diff、构建命令与制品哈希。

Bridge 语义固定在 [`bridge-contract-v1.md`](bridge-contract-v1.md)，动作固定在 [`action-allowlist-v1.json`](action-allowlist-v1.json)，样本与 predicate/oracle 固定在 [`sample-catalog.md`](sample-catalog.md) 和 [`sample-definitions-v1.json`](sample-definitions-v1.json)，证据必须通过包含样本级通过条件的 [`evidence-schema-v1.json`](evidence-schema-v1.json) 校验。实现时 Rust 类型生成 bridge JSON Schema，避免手写第二份产品协议模型。

## 监督结构

隔离 harness 模拟既有 Application ComputerUse Port 语义，持有任务级 lease/fencing、input generation、cancel 和 deadline。Supervisor 只用 inherited stdio JSONL 启动最小 Rust bridge Worker，bridge 直接使用固定 agent-desktop core 并保留完整 disposition。不得使用 status-only C ABI、CLI 或其他 IPC。Desktop 主进程、产品数据库和 Gateway 不参与。

上游 interaction lease 仅作为单 dispatch 附加互斥。派发、用户抢占和取消共用 Application-owned admission gate；将完整动作帧写入绑定当前 attempt/Worker 的 stdio 并完成 flush，是唯一线性化点。严格顺序为 observer ready → task fence/lease → generation → snapshot/target oracle → 最终比较 control/fence/generation → flush。之前为 `not_dispatched`，之后为 `in_flight`；Worker 收帧后才等待上游 lease。动作返回后必须重新 snapshot/Observe；只有预冻结 predicate 与 target identity oracle 都通过才报告已知结果。

首次 snapshot 也走同一 stdio bridge：Host 提交只读 oracle 冻结的显式进程/原生窗口定位和哈希身份，Worker 返回该目标下的 opaque qualified refs。dispatch 与 Observe 必须引用同一 snapshot、ref 和冻结身份；不得按标题或当前前台窗口隐式选择。Host 对 Observe 结果执行至少两次、间隔不少于 250ms 且位于 500ms 窗口内的独立只读采样，不能以 Worker verdict 代替。

固定上游 Windows 路径会在 `execute_by_ref` 内重新解析 ref，窗口匹配只有 HWND、PID 与进程启动令牌。正式窗口代际验证必须覆盖这次重新解析到动作交付的路径；Host 在调用前的一次只读核验或快照前后相同事件计数不构成放行动作的证据。在此门禁完成前，原生 Backend 只能返回 `unsupported/not_delivered`。

原生 Backend 拒绝 snapshot 时，Worker 应返回带错误分类、空 `snapshot_id` 和空 `qualified_refs` 的 `snapshot_result`，然后终止会话；Host 保持派发门禁关闭。成功响应必须具有匹配身份和唯一限定引用。

`IDENTITY-01` 使用 `sample-catalog.md` 的固定双窗口 fixture 规则：两个同名窗口的按钮标识也相同，Host 依据自启动进程与 fixture 自有窗口角色冻结原生窗口 ID 和不可变代际标记，Worker 独立只读重验，在冻结窗口完整快照中要求唯一限定引用。该标记只用于受控 fixture 的前置身份实验；在 Worker ref 解析至动作交付的生命周期门禁通过前，原生 dispatch 保持 `unsupported/not_delivered`。

受控反例须在同一 fixture 上分别提交目标窗口的错误代际摘要和诱饵窗口的真实身份；两者均须在 Worker 原生 snapshot 阶段以 `identity` 拒绝、返回空引用，且目标和诱饵点击计数均不变。此反例只证明前置拒绝，不替代动作交付路径的代际证明。

`IDENTITY-02` 可先运行不计正式通过的只读诊断：fixture 自行销毁并重建目标窗口，探针用保存的旧限定引用调用上游公开 `resolve_saved_ref`，仅记录是否拒绝、完整 HWND 是否复用、代际是否变化和零点击。探针不调用动作 API；未发生完整 HWND 复用时不能把普通 stale 拒绝当成窗口代际风险已解决。

附加只读反例保持窗口不变而替换其同标识按钮，用相同旧引用检查上游是否重新绑定新按钮；无论结果如何都只输出布尔判定和零点击，不把按钮替换视作 HWND 复用，也不调用动作 API。

## 取消与故障

- 未派发及步骤间取消：冻结准入并确认没有新动作，可确定完成。
- 在途取消：终止 Worker；若可能交付，固定为 `unknown`，不得重试。
- Worker 终止后：Windows Job Object/macOS 受监管进程组保证 containment；无副作用只读探针检查完整进程树、interaction lease、修饰键/鼠标按钮、临时资源和新实例 Observe。禁止合成补偿输入；任一残留进入 global desktop quarantine，宿主重启或明确只读恢复程序通过前不得新派发。
- 用户输入：观察器只维护宿主会话 `u64 generation` 与 `driver_current_attempt | user | unknown | observer_lost`。只有绑定当前 attempt/Worker 的合成事件可忽略；其余分类、observer 未 ready/重启/回退均递增并冻结。固定交错样本要求零漏分、零误分。

## 统一矩阵

两平台使用相同样本 ID 和判定字段：同名双窗口、应用重启/句柄复用、前后台变化、真实用户输入与合成输入交错、deadline/kill/断连、96/192 DPI 或等价缩放、多显示器、权限拒绝、Notepad/TextEdit、Electron fixture、Windows WPS/文件资源管理器、macOS Office/Finder、干净机分发。精确样本、deadline、稳定窗口、显示拓扑和 predicate 见 `sample-catalog.md`。

所有安全硬门槛采用零容忍：误投递、用户输入后新增派发、unknown 自动重试、资源残留、未声明运行库或无法建立自有签名责任链任一出现即淘汰，不能用其他样本得分抵消。

## 证据边界

只记录 `agent-desktop-spike-evidence/v1` 规定的 build manifest hash、平台/架构、样本/predicate、attempt/worker/fence、generation、dispatch boundary、delivery/Observe/cleanup/outcome 封闭枚举与单调耗时。不得保存输入正文、截图、完整窗口树、应用列表、完整命令输出或 Agent Payload。

## 结论

独立验证后更新 AD-CU-07 或建立后继 ADR：双平台共享或整体淘汰。任一平台硬失败均淘汰候选，不保留单平台长期双栈出口。无论哪种结论，本 Spike 都不直接修改产品代码。
