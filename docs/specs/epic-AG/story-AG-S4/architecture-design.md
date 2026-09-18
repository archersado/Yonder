# AG-S4 架构设计

## 边界与依赖

Skill 是 Agent 侧使用说明和编排规则，不是新的执行层。调用链保持 `Agent → Yonder CLI/MCP 或未来同源入口 → Agent Gateway → Application 用例 → 能力 Adapter`。所有任务先进入 Yonder；Skill 不读 UDS/Named Pipe、不访问 SQLite、不启动桌宠或 Worker。

依赖方向不变。协议、能力名、请求和响应以 Rust 类型及其派生 Schema 为唯一来源；Skill 只解释如何组合已发布工具，不手写第二份协议模型。未来协议变化先更新所属 Architecture Decision、Story 和 OpenSpec，再更新 Skill。

## BUA 与 ego-browser Skill 组合

Yonder 创建任务并通过 Browser Bridge取得 ego-lite Task Space，持久化受限外部引用。嵌入的 ego-browser 子模块只允许在该引用对应的空间中执行页面动作，并继承以下围栏：一个用户目标使用一个 Task Space；失败后恢复同一空间；复用已有 Page；用户接管立即停止；完成时只保留用户明确要求的页面。

“嵌入”采用受控的 Skill 参考模块，而不是复制 ego-lite Runtime。生成 Skill 时记录所基于的 ego-browser Skill 版本及来源，摘取执行所需规则并增加 Yonder 任务包装；上游更新通过显式差异审阅。若当前 Browser Gateway 只能管理生命周期而不能承接所需动作，Skill 不得通过独立 Task Space 绕过，须等待 BU Story 补齐受监管调用契约。

## CUA、Document 与 Command 路由

CUA 使用 Yonder 粗粒度组合步骤入口；Yonder负责步骤声明、资源准入、SDK调用、动作后Observe和边界推进。Skill从运行时能力读取动作 Schema，不列出固定动作全集，不携带Node代码。

Document 与 Command 在各自产品 Story 完成后按能力发现启用。Document Port 不暴露 OOXML；Command 默认结构化调用，Shell 单独授权。Skill 只组合已实现用例，不通过本机通用 shell、Office 自动化或直接文件修改填补能力缺口。

## 状态与契约

Skill 保存于发布仓库的独立目录，包内包含主入口和按需读取的 BUA/CUA/Document/Command 参考；不包含运行时凭据和用户数据。调用方始终使用 Yonder 返回的 `task_id` 与最新 `sequence`。副作用请求超时、断连或结果不明时读取任务/事件并停止，不能自动重放。

首次实现前需确定 Skill 包的安装位置、版本与 Yonder 协议兼容范围；这些属于实施设计，不在当前规格中猜测。Document、Command 或 BUA 受监管动作协议尚未齐备时，AG-S4保持draft，不创建实现 Proposal。

## 失败与验证

独立验证必须使用正式 Yonder Runtime，覆盖能力发现、单任务四类操作、BUA同空间恢复、CUA动作后Observe、Document锁/另存、Command结构化参数、用户接管、缺能力拒绝、unknown不重试和完成清理。不得只校验 Markdown 文本或用Mock工具冒充端到端通过。
