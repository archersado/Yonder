# AD-EX-02 Jev 快脑技术接入路线

状态：Accepted（macOS-only，Windows 后补）
日期：2026-09-24
关联：EX-S1、AD-EX-01

## 待决问题

AD-EX-01 已允许 Yonder 内置有界快脑循环，但尚未证明 TypeSafe AI Jev 能以可接受的质量、token 成本和延迟服务于 CUA、ego-lite BUA、Document 与 Command。也未确定模型位于本机还是远端、调用认证与数据边界、故障时如何交回慢脑。系统边界已接受不等于具体技术路线已接受。

## 2026-09-24 决策

用户变更：产品接线先限定 macOS，Windows 与费用证据后补。基于 macOS Spike 结果，接受 **macOS-only 远端 Jev 子路线**：远端端点为 `https://api.typesafe.ai`，SDK 固定 `@typesafe-ai/sdk@0.6.0`，模型为 `jev-latest`。API Key 由独立 Jev 设置面板一次性输入并直接写入 macOS Keychain，服务名为 `Yonder`、账号名为 `jev`；不得进入 Jev 配置、SQLite、任务数据、日志、进程参数或 UI 回显。

面板只读取“是否已配置”，不得读取、导出或回填密钥。空字段保持既有凭据；用户显式提交非空字段才替换。写入必须使用原生 Keychain API，禁止以 `security ... -w <key>` 或任何会把密钥放入 argv、环境变量、临时文件的路径保存。

本决策只授权 macOS 运行时代码。Windows 路径不编译、不注册、不验证；补齐 Windows 同等证据并更新本 ADR 前，不得以“双平台已完成”关闭 EX-S2。2026-09-25 TypeSafe 官方发布页列出 Jev input 单价为 `$0.042 / 百万 tokens`（即 `$42 / 10 亿 tokens`），并明确 output tokens 免费；因此可以形成完整的 Jev 官方价格回算，但不得把同价基准对照误写成外部慢脑供应商的实际账单。除此之外，AD-EX-01 的职责分工、单一 Gateway、单一状态源和每步 Observe 不变。

2026-09-24 产品接线复验修订单次超时：原 Spike 的 1500ms 在 2026-09-22 样本中最大为 993.844ms，但当前 macOS 网络仅无凭据 TLS/端点往返已约 1795ms，产品真实调用因此在进入安全超时分支前无法取得响应。官方说明 Jev 端到端响应约 70–500ms，但该数字不含所有客户端地域网络差异。macOS 产品 Adapter 的单次 SDK/Worker 上限调整为 **3000ms**，SDK 重试仍为 0；这是地域网络兼容上限，不修改 EX-S1 原始样本结果，也不代表放宽质量、取消或 `unknown` 门禁。若 3000ms 仍超时，立即交回，不再次调用。

## Spike 判定

2026-09-22 已冻结实验口径（见 `openspec/changes/ex-s1-jev-ultrafast-spike/design.md`）：样本版本 `ex-s1-v1`，共 8 条样本、每类执行层 2 条；每步候选上限 10；Jev 置信阈值 0.75；单次决策预算 1500ms；任务成功率不低于 90%；误动作为 0；慢脑 token 降低率不低于 50%；端到端 P95 不高于慢脑基线 1.2 倍；成功路径交回率不超过 20%；取消至停止新动作不超过 500ms。相同任务对照慢脑逐步决策，记录成功率、误动作、慢脑 token、端到端时延、Jev 请求量/费用和交回率。Windows/macOS 均需结构化证据；Windows 暂缓期间只可形成 macOS 子结论。

若不能从受支持 Observe 形成足够的候选动作，或每步仍依赖慢脑补参，或发生权限绕过/旧目标动作/`unknown` 自动重试，淘汰对应执行层路线；不得为了四类覆盖引入第二执行栈。Jev 服务形态、授权、数据传输及隐私必须在技术结论中明确。通过后才将本 ADR 转为 Accepted，并按 Story → OpenSpec → 实施 → 独立验证推进；不修改 AD-EX-01 已接受的职责分工。

## 2026-09-22 macOS 子结果

EX-S1 使用 `@typesafe-ai/sdk@0.6.0` 与 `jev-latest` 完成 8 个固定样本、3 轮重复的 macOS 隔离验证：24/24 成功、0 误动作、token 降低 50.142%、Jev/基线 P95 时延比 1.012、单次最大决策 993.844ms、成功路径交回率 0%、取消停止 1.101ms。证据见 `openspec/changes/ex-s1-jev-ultrafast-spike/verification-macos.md`。按 2026-09-25 官方 input 单价与 output 免费规则回算，Jev 候选摘要路径 8088 个 input tokens、834 个 output tokens 的完整公开价格为 `$0.000339696`；完整状态基线 17061 个 input tokens、834 个 output tokens 按同一 Jev 价格基准为 `$0.000716562`，费用下降 52.594%。这不是外部慢脑供应商的实际账单。Windows 证据仍缺失，因此本 ADR 只形成并接受 macOS-only 子路线，不形成双平台结论。

Windows 暂缓期间不配置远端 CI 或仓库 Secret；恢复验证后在本地 Windows 环境运行同一探针并产出 `windows-r3.json`。产物不得包含密钥、完整状态或模型响应正文。
