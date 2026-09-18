# AD-DEV-01 按技术模块组织 Epic 与 Story 文档门禁

状态：Accepted。来源：用户于 2026-09-11 明确要求以技术模块建立 Epic 文件夹，每个 Story 文件夹含产品需求、架构设计和视觉交互设计，再转为 OpenSpec Proposal 实施。

规划唯一入口为 docs/specs/README.md。Epic 位于 docs/specs/epic-<模块>/README.md，Story 位于其下 story-<ID>/。每个 Story 必须有 README.md、product-requirements.md、architecture-design.md、visual-interaction-design.md。无独立 UI 的 Story 也必须说明调用方、可观察状态和错误反馈；不允许省略视觉交互文档或仅写“不适用”。

先完成三份设计及可验证验收条件、明确跨模块依赖和未决事项，再创建关联该 Story 的 OpenSpec proposal/design/tasks/delta spec。实现不得先行，OpenSpec 不代替产品与设计文档。变更系统边界等仍先更新 ADR；原 E0 技术路线和双平台证据要求不变。

Story README 使用唯一 Story、Epic、Status、OpenSpec 字段。状态为 draft、design-review、ready、implementing、verifying、done、deferred。ready 表示设计与前置门禁已明确；设计审阅是工程审阅，不隐含每一步都要向用户请求批准。用户明确暂缓的事项仍保持 deferred。新增设计文档不追认历史实现合规或产品完成。

迁移既有七个 Story 到所属技术模块，使用新模块 ID，旧文档保留跳转和历史 ID；原正文只在新目录 legacy-record.md 保留，不再维护两份当前状态。既有跨模块 oct-s1-task-status Change 暂作为 TM-S1 历史承接，Gateway、桌面、资源准入的后续工作分别在模块 Story 创建新 Change，禁止继续扩大历史 Change。月度 Epic 文档降为历史里程碑，E0 降为跨模块技术门禁索引。

CI 检查模块目录、唯一 ID、四份非空文档、必需章节、双向关联和 PR 的 Story 设计状态；PR 不得以旧平铺 Story 文件替代新目录。CI 验证结构不能代替设计质量审阅或原生验收。迁移中的历史 Proposal 可引用 design-review Story，但不得据此继续实现；新 PR 的 Story 必须达到 ready 或更后阶段。

本次调整研发规划与门禁，不改变运行时架构、依赖、协议或数据库。无密钥、桌宠或任务代码变更。
