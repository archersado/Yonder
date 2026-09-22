# AD-EX-02 Jev 快脑技术接入路线

状态：Proposed  
日期：2026-09-21  
关联：EX-S1、AD-EX-01

## 待决问题

AD-EX-01 已允许 Yonder 内置有界快脑循环，但尚未证明 TypeSafe AI Jev 能以可接受的质量、token 成本和延迟服务于 CUA、ego-lite BUA、Document 与 Command。也未确定模型位于本机还是远端、调用认证与数据边界、故障时如何交回慢脑。系统边界已接受不等于具体技术路线已接受。

## Spike 判定

EX-S1 在 2026-10-05 前冻结统一样本、基线、数值门槛和淘汰条件，再验证动态候选操作/目标、目标新鲜度、输入文本缺参、独立完成判断、取消/接管、`unknown`、模型超时与断线。相同任务对照慢脑逐步决策，记录成功率、误动作、慢脑 token、端到端时延、Jev 请求量/费用和交回率。Windows/macOS 均需结构化证据；Windows 暂缓期间只可形成 macOS 子结论。

若不能从受支持 Observe 形成足够的候选动作，或每步仍依赖慢脑补参，或发生权限绕过/旧目标动作/`unknown` 自动重试，淘汰对应执行层路线；不得为了四类覆盖引入第二执行栈。Jev 服务形态、授权、数据传输及隐私必须在技术结论中明确。通过后才将本 ADR 转为 Accepted，并按 Story → OpenSpec → 实施 → 独立验证推进；不修改 AD-EX-01 已接受的职责分工。

## 2026-09-22 macOS 子结果

EX-S1 使用 `@typesafe-ai/sdk@0.6.0` 与 `jev-latest` 完成 8 个固定样本、3 轮重复的 macOS 隔离验证：24/24 成功、0 误动作、token 降低 50.142%、Jev/基线 P95 时延比 1.012、单次最大决策 993.844ms、成功路径交回率 0%、取消停止 1.101ms。证据见 `openspec/changes/ex-s1-jev-ultrafast-spike/verification-macos.md`。费用单价未公开，费用字段保留为不可用。Windows 证据与费用仍缺失，本 ADR 仍为 Proposed，不得进入 EX-S2 实施。

Windows 暂缓期间不配置远端 CI 或仓库 Secret；恢复验证后在本地 Windows 环境运行同一探针并产出 `windows-r3.json`。产物不得包含密钥、完整状态或模型响应正文。
