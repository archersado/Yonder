# AG-S2 本地Agent任务登记联调

关联AG-S2三份设计、docs/specs/epic-AG/story-AG-S2/及Accepted AD-AG-01/02、AD-ST-01。实现Gateway 1.1 task.create登记、SQLite原子幂等、预绑定身份私有stdio测试Agent；不提供人工创建或伪造running，正式桌面Socket接线与生产认证保留门禁。

Architecture Impact：architecture-change，先有AD-AG-02。Rust协议新增task.create/TaskCreate，SQLite2→3同事务迁移。无新依赖，无跨层互调/HTTP/Planner。描述不是可执行指令。完整AG-S2不因本子范围通过而Done。

## Agent命名增量（2026-09-14）

关联AG-S2、TM-S1、DS-S2三份设计与Accepted AD-TM-07名称技术定稿。Architecture Impact：architecture-change（协议1.3、SQLite schema5）。只落实名称，不授权接管目标或Recording。

要求：1.3 Agent创建必须提供合法name；旧版不接收name且响应不增加name字段；新版本创建/get/list/cancel返回同一名称。名称1–256 UTF-8字节、非空、无控制字符，原样保存。相同owner/key仅在description和name完全相同才返回既有最新快照，否则-32009，不产生记录。取消后重试不恢复任务。

迁移前唯一SQLite备份，事务新增tasks.name与task_creations.name；旧NULL不伪造名称，保留所有业务记录。未知格式、已有加密旧版本、备份或DDL失败拒绝升级。卡片与详情用名称/历史缺名反馈，ID仍可查看。验证名称边界、旧新版门禁、原子回滚、备份与数据保留、正式嵌入UI和macOS原生显示；Windows暂缓、完整Story不归档。
