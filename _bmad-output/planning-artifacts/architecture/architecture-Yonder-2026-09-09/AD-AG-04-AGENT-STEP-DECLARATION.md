# AD-AG-04 Agent步骤声明首批契约

状态：Accepted（created任务步骤声明设计；不授权真实动作、停止或Recording）。日期：2026-09-14。
Architecture Impact：architecture-change（Gateway1.4、SQLite6、步骤声明Port）；关联AG-S3、TM-S1/S2/S5、AD-AG-01、AD-ST-01、AD-CU-02及Proposed AD-TM-08。

## 来源与范围

原产品简报「MVP 主干链路」6～9、「Task Space 与权限模型」、补充材料「执行原则」要求Agent驱动、当前步骤、动作后观察与审计；架构「任务、状态与恢复」明确步骤/结果/历史及当前事实源。后续用户仅Agent创建、统一轻量任务菜单及人工接管变更保持。

本决定只接受**未开始任务的步骤声明**，为后续真实动作绑定step_id提供前置；标签不解释成指令，不派发SDK、不改变status、不制造attempt或WorkRef。running/暂停/恢复的步骤声明、动作提交、执行结果/观察仍归后续AG/TM/CU契约，不能以此删减完整原需求。

## 协议与调用方

Rust是唯一类型来源。协议1.4新增task.step.declare（capability=task.step.declare）和task.step.get（capability=task.read）；两方法都须已握手且存储支持步骤能力，旧会话返回-32010。AuthContext由宿主绑定，不从params.agent_id构造；declare仅Agent且只允许所属任务，LocalUser可经既有可信查询入口读取、不能写声明。

declare参数沿用JSON-RPC id、agent_id、deadline，新增task_id、expected_sequence、step_id、label。step_id复用1～128 ASCII标识限制，label复用1～256 UTF-8字节非空/无控制字符纯文本规则。拒绝未知字段，不接受action、program、args、PID、status或owner。request_id仅是调用身份，task_id+step_id构成声明去重身份。

声明结果包含当前任务快照及已接受声明（step_id、label、accepted_sequence）；get返回当前任务快照及最近接受声明或null。声明序号与任务sequence同一空间，accepted_sequence永久记录接受时版本，重试时当前任务sequence可以更大，不冒充新声明。名称兼容仍沿既有1.3规则。

## 幂等与事务

可信身份、方法版本、参数和deadline通过后，IMMEDIATE事务内验证任务归属；不存在/越权同为-32004。先查该任务step_id：同label逐字节匹配返回既有声明及当前快照，即使任务后来取消也不复活；不同label返回声明冲突-32013，不泄露他人声明。

首次声明仅created且expected_sequence等于当前sequence；非created返回-32012“任务已开始，需执行控制契约”，序号不符-32011，耗尽拒绝。事务同时递增tasks.sequence、插入不可变task_steps声明、插入原state→原state事件与Outbox。不是任务状态迁移；任何CAS、声明、事件或Outbox失败全部回滚，返回成功前落盘。

task.step.get在同一读取事务内取得快照及最近声明，不能拼出不同版本。当前步骤来自SQLite最近声明，不以UI缓存或事件重建任务状态。所有旧声明保留，可通过1.4 task.events读取；不支持改标签/删除声明、不回收step_id。

## 事件兼容与存储

SQLite6增加task_steps，主键(task_id,step_id)，声明accepted_sequence与events(task_id,sequence)绑定且唯一。声明与事件/Outbox同事务保证不存在孤立声明。原events状态字段保持合法值，新增协议TaskEvent可选step_declaration，由对应事件序号关联得到；状态迁移事件不带该字段。

1.4客户端看到明确step_declaration，表示Agent声明而非执行证明。1.0～1.3 task.events投影移除该字段，保留完整连续序号与状态未变事件，不能删事件导致游标缺口；现有get/list/cancel快照不加current_step字段，避免扩大严格客户端形状。查询按原事件分页上限取对应声明，最多100项，不全表扫描。

每任务最多1024声明、全库最多10000声明；达到上限返回-32014且不递增序号。已有声明幂等命中不占新配额；不自动删除历史/未同步数据。限额是工程选择，不称为产品要求，完整任务/附件配额仍由ST后续定稿。正常日志不含label、完整请求或输入。

现有明文schema5升级6前SQLite一致快照备份，DDL同事务；旧任务无声明，不能补造。既有2/3及无危险标记4先沿已接受名称/保留数据迁移至5，再同一迁移事务到6；未知格式、危险4、备份失败/DDL失败拒绝，业务数据不变。已有加密schema5保持原样且步骤能力不支持，不自动升级加密库；加密旧格式沿原门禁拒绝。本子范围不改密钥或加密要求。

## 评审与完成边界

设计审阅结论：上述纯登记与读取不依赖AD-TM-08尚未定稿的attempt/Worker/控制事务，也不依赖原生权限；只支持created新声明是显式首批边界，不是完整步骤模型完成。进入实现须先有AG-S3三份设计与关联OpenSpec。

验证必须覆盖未握手/旧版本/伪造身份/越权/LocalUser/过期、UTF-8及控制字符、重复/异内容、取消后重试、并发声明/取消、三表及声明原子回滚、配额、事件旧新版投影、备份和拒绝加密/危险格式。Windows原生接入继续暂缓，库合约不能冒充原生双平台通过。真实步骤执行及完整接管仍未授权。
