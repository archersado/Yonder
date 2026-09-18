# AD-BU-02 Agent Browser Gateway

状态：Accepted；日期：2026-09-16。关联BU-S2、AG-S1、TM-S2。

本地Agent通过统一Gateway请求`task.step.advance`和`browser.execute`，协议minor 1.7/1.8。Agent只提供所属task、期望sequence和动作种类；当前步骤、attempt/Worker/host身份及external_task_ref均由Yonder事实源和Supervisor生成。Create使用Agent任务名称；其余动作读取已持久化引用。

首次Browser动作取得`Resource::Browser`并准备attempt；Permit对象可结束作用域但Admission占用保持，后续步骤复用同一任务占用。finish成功后提交Observe、普通边界和completed，再显式释放。unknown、pending控制、存储失败或Runtime缺失均保留保守占用，不自动重试。Windows声明dependency missing。
