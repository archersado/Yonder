# EX-S1 Jev Ultrafast 隔离探针

本目录只服务 `ex-s1-jev-ultrafast-spike`，不接产品 Gateway、SQLite、任务状态或真实用户任务。探针使用官方 `@typesafe-ai/sdk@0.6.0` 的 `systemOne` 选择题接口；Jev 只能在固定候选 ID 中选择，不生成自由文本、命令、文档正文或新权限。

## 样本

- 版本：`ex-s1-v1`
- 8 条样本：CUA、BUA、Document、Command 各 1 条成功路径、1 条失败路径。
- 每步候选上限：10；实际每条 2 个候选，其中必含 `handback`。
- 置信阈值：0.75；每次决策超时：1500ms，SDK 重试关闭。
- 请求间隔：250ms；间隔用于降低连发超时，不计入单次决策时延。
- 对照组使用完整 Observe 状态逐步决策；Jev 组使用候选摘要。两者共用目标、计划允许动作、候选 ID 和安全闸。
- 失败样本的正确结果是安全交回，不是自动重试或执行不可派发候选。

## 运行

```bash
npm install
npm run check
TYPESAFE_API_KEY=<key> npm run probe -- --repeat 3 --output evidence/macos-result.json
```

Windows 使用同一 Node 探针，不引入平台分支：

```powershell
npm install
npm run check
$env:TYPESAFE_API_KEY = "<key>"
npm run probe -- --repeat 3 --output evidence/windows-r3.json
```

Windows 暂缓期间不配置远端 CI 或仓库 Secret；恢复后在本地执行上述同构命令。产物只包含结构化指标，不包含密钥、完整状态或模型响应正文。

可选 `TYPESAFE_PRICE_PER_MTOK` 提供每百万 token 价格；未提供时证据中的 `estimated_cost` 为 `null`，不虚构费用。`.env`、`evidence/` 和 `node_modules/` 均不入库。SDK 日志关闭，输出只包含指标、样本 ID、错误分类和 token 用量，不包含 API Key、完整状态或模型响应正文。

## 指标

输出统计任务成功率、误动作、基线/Jev token、决策时延 P95、成功路径交回率、请求数与可选费用。失败样本会提高整体交回比例，因此 ≤20% 的交回门槛只对成功路径计算；整体交回率另行保留。取消检查在发起一次 Jev 请求后立即 Abort，只记录停止耗时与错误分类，不执行后续动作。
