# Proposal：EN-S3 阶段 2 Gateway 能力协商与分发收敛

关联 Story：EN-S3（`docs/specs/epic-EN/story-EN-S3/`）。关联 AD：无新增 ADR；本 Change 受外部行为完全等价约束，Architecture Impact 为 `none`。

## 问题

Gateway 握手目前由数十个顺序赋值、两条人工维护的版本链和逐项 `push` 组成，`can_audit` 已出现重复赋值。请求分发同时依赖多段早返回判断，新增协议变体缺少单一可审计入口。协议常量已演进至 1.50，而既有 Gateway 能力链最高只投影 1.49；本轮先保持该外部行为并显式锁定，继续复制条件会放大静默降级风险。

## 方案

- 先固化 minor 0–50 的 macOS 完整存储/运行时能力协商 golden，锁定协议版本、能力顺序、版本、availability 与 reason。
- 用单一能力快照与声明式规格表完成握手计算和 `CapabilityInfo` 生成，删除重复赋值与两条嵌套版本链。
- 将请求所需协商能力收敛为穷举映射，由统一入口返回既有 `-32010` 错误；业务处理与授权语义不变。
- 每个阶段运行 Application/Workspace、协议生成和架构门禁；不修改 SQLite、wire 类型、依赖方向或用户界面。

## 非目标

不实施 EN-S3 阶段 3–5，不改变能力可用性或错误文案，不新增协议版本，不处理 Windows 原生验证。
