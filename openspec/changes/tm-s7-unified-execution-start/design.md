# 设计

抽取 Application 级 `start_execution`：对 created 任务同事务写入 running、步骤、prepared attempt、sequence、事件与 Outbox；成功后才调用能力 Adapter。running 任务仅在已 Observe 的步骤边界准备下一 attempt。

CUA 与 BUA 回归该用例。Document 与 Command 不提供绕行入口；各自在 FI-S1、CM-S1 和协议门禁通过后声明资源并接入。派发或 Observe 不确定时记录 unknown，不自动重试。
