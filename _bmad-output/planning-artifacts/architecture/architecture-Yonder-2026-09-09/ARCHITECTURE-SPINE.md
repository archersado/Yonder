---
name: Yonder
type: architecture-spine
status: draft
updated: 2026-09-09
paradigm: 模块化单体、六边形端口/适配器、受监管执行进程
companions: [DEVELOPMENT-AND-CHANGE-MODE.md]
---

# Yonder 架构围栏

## 产品边界

Yonder 是 Windows/macOS 上轻量常驻的 Agent Tool，提供本地系统操作、用户操作感知和个人上下文采集。它可被桌面 Agent、Agent CLI 或外部云端 Agent 驱动，但不是 Agent、规划器或云端服务。BUA 直接复用 ego-lite Browser Task Space，Yonder 不复制该模型。

## 模块与依赖

```text
React 桌宠/UI ──> Tauri 组合根
                        │
CLI/MCP ──Local IPC──> Agent Gateway <──WSS── 外部云端平台
                        │
                   Application ──> Domain
                        ▲ ports
 Adapters: CUA / ego-lite BUA / Command / File / Document / Context
```

最小 Cargo Workspace：`apps/desktop`、`apps/yonder-cli`、`crates/domain`、`crates/application`、`crates/adapters`、`crates/protocol`。

依赖只能是 desktop→adapters/application，adapters→application/protocol，application→domain/protocol，CLI→protocol。Domain 无技术依赖；Application 不引用具体 Adapter；Adapter 不互调；React 不拥有核心状态。CI 检查 Cargo 图、前端 import 和协议生成。

## Agent Gateway

- 本地 Agent 经 MCP stdio 或 CLI，通过共享 Rust IPC Client 接入；IPC 使用 `interprocess 2.x`、Tokio、版本化 JSON-RPC，映射 Unix Domain Socket/Windows Named Pipe，不开放本地 TCP。
- Yonder 主动建立单一 WSS 连接云端，不开放公网入口、端口映射或 P2P。断线按 `last_sequence` 补传 Outbox，且不影响本地能力。
- 本地与云端请求统一为 `AgentRequest + AuthContext`。Rust 协议类型生成 JSON Schema 和 TypeScript；hello 协商版本与 capabilities。
- IPC 限当前 OS 用户，设备凭据进入系统 Credential Store。MVP 不做细粒度授权；支付、发送、删除、安装、提权仍需确认。

## 任务、状态与恢复

- Execution Session 是 Yonder 内部执行实体；ego-lite Task Space 是 BUA 外部实体；两者映射为统一 Task Status。
- SQLite 当前状态表是唯一当前事实源，配合追加事件日志与同事务 Outbox，不采用完整 Event Sourcing。每任务 sequence 严格递增。
- 持久化状态、步骤、结果、错误、等待原因、恢复信息和外部引用；句柄、连接、窗口对象及订阅仅驻内存。
- 重启后 running 转 interrupted；重新 Observe 后由外部 Agent 决定继续、跳过或局部 replan，禁止自动重试副作用未知动作。
- 提供 task.get、task.events(after_sequence)、CLI status/watch、MCP 查询/长轮询与云端事件推送。
- 每一步后增量 Observe；窗口切换、导航、目标丢失时完整 Observe。Yonder 校验预期条件，不负责语义规划。
- CUA 使用唯一前台租约，用户输入即暂停；BUA MVP 单并发。后台读取可并行，同一文件禁止并发写。

## 执行能力

### CUA 与 BUA

CUA 只含模型无关 Driver，不内置 Agent/Planner/Model。Qwen cua-driver 与 trycua cua-driver 使用相同 Windows/macOS 黑盒用例验证，只交付胜者；OSWorld 仅作基准。Driver 由 Supervisor 按需启动；崩溃或超时将动作标为 unknown，Observe 后交由 Agent 决策。

BUA Bridge 直接调用 ego-lite/ego-browser，保存 external_task_ref 并映射状态，不复制 Task Space。只有经 Yonder 调用才保证桌宠收到状态。由于 ego-lite 当前没有 Windows Runtime，Windows 首版 BUA 为 `capability_unavailable`；不得自动回退到 CUA 或另一套浏览器引擎。

### Command、File 与 Document

Command Executor 与 CUA 分离，默认结构化 program/args/cwd/env；Shell 必须显式启用。支持超时、取消、进程树终止、输出上限、最小环境及脱敏审计。

File Tools 提供打开、定位、元数据、列表、读、复制、移动、重命名、建目录、受控写入和移入回收站；路径绝对化、规范化并阻止穿越。

Document Port 不暴露 XML。OOXML Adapter 处理 docx/xlsx/pptx 的结构读取与局部修改，写请求携带 expected_hash，多操作原子提交。默认另存；覆盖须显式要求；临时写入并校验后原子替换。不得绕过 Office/WPS 文件锁；复杂排版、图表、宏、旧格式和宿主 UI 操作降级 CUA。Rust 进程内与按需 Node Worker 经 Spike 选出唯一实现，达到保真门槛时优先 Rust。

## Recording 与桌宠

Recording 仅手动开始，每设备最多一个。原始时间线不可变，派生版本化 CUA Trajectory；AX 目标优先、相对坐标兜底。输入标记 user、agent_cua 或 replay；只有 user 事件生成轨迹，避免递归录制。

桌宠导入声明式 ZIP，状态含 idle、listening、recording、thinking、executing、waiting_for_user、success、failed、paused。MVP 仅 PNG/WebP，禁止脚本和可执行内容；隔离解压并校验大小、尺寸、帧数和路径。

## 个人上下文

- 当前窗口：Windows WinEvent Hooks/UI Automation；macOS NSWorkspace/AXObserver。只采元数据并由事件驱动。
- 浏览：BUA 复用 ego-lite 事件；日常记录由 Chrome/Edge 扩展经 Native Messaging Host 上报，不读取 History DB；隐私窗口排除。
- 文档：只处理明确授权或任务选择的文件，由实际活动触发；不全盘扫描。OOXML 复用 Document Adapter，文本/PDF 使用只读 Extractor；MVP 无 OCR。
- 操作序列：仅手动 Recording 采集。密码框、安全界面与排除应用始终忽略。

SQLite 存元数据并使用 FTS5；大内容存加密附件。Agent 只能经 Context Port 查询，默认返回摘要与引用。SQLite 使用 SQLCipher，附件文件级加密，主密钥只存 Keychain/Credential Manager。MVP 无本地向量库、Embedding、知识图谱和跨设备密钥同步。

上下文默认保留 90 天，Recording 原始素材 30 天，任务附件 7 天；轨迹与固定内容长期保留。删除同步清理索引、附件和未发送 Outbox。

## 非功能与发布

- 空闲 CPU 平均低于 1%，内存目标低于 150MB，托盘至可用低于 3 秒，窗口变化至状态更新低于 300ms。
- 重型 Worker 按需启动、空闲退出；解析、加密和索引不得阻塞 UI。
- 日志不得记录正文、截图、输入、完整命令输出或 Agent Payload。
- 一次发布包含 desktop、CLI、MCP、IPC 协议、迁移和 Driver manifest；版本固定，不用 latest。
- 数据库仅向前迁移且升级前备份；更新签名，macOS notarize，Windows code-sign；只设 dev/stable 通道。

## MVP 明确不做

云端服务、内置 Agent/Planner、完整 Event Sourcing、通用 HTTP Server、全盘采集、本地语义知识库、Office COM/AppleScript/WPS 插件和多个 CUA 同时控制桌面。
