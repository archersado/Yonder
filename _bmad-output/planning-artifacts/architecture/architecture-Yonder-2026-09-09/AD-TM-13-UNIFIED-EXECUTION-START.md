# AD-TM-13 统一执行启动状态

状态：Accepted（Application 统一启动边界；Document/Command 接线仍受 FI-S1、CM-S1 与协议门禁）  
日期：2026-09-21  
Architecture Impact：architecture-change（新增跨能力 Application 用例与资源声明；协议/持久化字段待后续设计确认）

## 决策

首次派发 CUA、BUA、Document 或 Command 副作用前，可信 Application 必须用同一启动事务将任务从 `created` 迁移至 `running`，并写入步骤、prepared attempt、事件和 Outbox。提交失败不得派发。Adapter 不拥有任务状态，也不得以 UI 动画、进程启动、文件临时写入或浏览器创建替代该事实。

`running` 后的下一步复用已有准入、Observe 与步骤边界；unknown 不自动重试。用户输入、暂停、取消、接管、完成和失败沿用 TM-S2/TM-S3 的停止确认与资源释放，不新增平行生命周期。

## 理由

CUA 与 BUA 已分别在首次 attempt 前调用准入/启动逻辑；Document 与 Command 尚无正式执行入口。若各能力各自启动，会出现副作用已开始但任务仍显示 `created`、事件与 Outbox 不一致，或 UI 对执行中做出不同解释。

## 后果与门禁

Application 成为唯一跨能力启动协调者；能力 Adapter 仅声明受监管资源并派发/Observe。Document 写入继续依赖 FI-S1；Command 继续依赖 CM-S1 的双平台 Spike；新增 Gateway 请求、资源持久化或协议类型必须从 Rust 唯一模型生成并先更新本 ADR。

本 ADR 不授权 Document/Command Gateway、通用 shell、直接 OOXML 文件写入或自动 Resume。实现前需由 TM-S7 三份设计及 OpenSpec 明确具体用例、资源和迁移影响。
