# AG-S3 架构设计

## 边界与依赖

Agent→可信GatewaySession→Application声明/读取→TaskStore事务；依赖方向保持。Rust协议唯一来源，声明类型和响应在实施时派生Schema/TS，不手写第二套模型；不调用CU SDK或引入新依赖。

## 状态与契约

按[AD-AG-04](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-04-AGENT-STEP-DECLARATION.md)：Gateway1.4 declare/get，声明只能来自归属Agent，created新声明不改变任务status；task_id+step_id幂等，label完全相同才返回既有事实。当前步骤来自SQLite声明表，get在同一读取事务取得任务/声明。

SQLite6任务序号、声明、同状态事件与Outbox同事务；事件可选step_declaration在旧会话投影移除，不能过滤掉声明事件。按有界事件页关联声明；每任务1024/全库10000上限。Application不引用Adapter，UI不拥有当前步骤或状态；控件不得生成步骤。

## 失败与验证

方法能力默认不支持，新版握手且Adapter声明支持才开放；未握手/过期/身份/版本拒绝不查幂等。越权同不存在；CAS、三表/声明故障回滚；未知副作用不涉及本纯登记用例，后续派发仍必须AD-TM-08。

备份后明文schema5→6，旧2/3/安全4沿既有迁移；旧加密库拒绝步骤能力且保持格式，未知/危险/DDL失败不覆写。验证对应STEP-01～06，使用真实SQLite与本地预绑定stdio Agent；不手工插正式任务，不把核心合约当原生双平台通过。
