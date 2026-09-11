---
title: "Yonder 产品简报"
status: draft
created: 2026-09-09
updated: 2026-09-09
---

# Yonder 产品简报

## 执行摘要

Yonder 是面向 AI Agent 的本地感知与执行工具，连接用户的 Windows 或 macOS 电脑与第一方、第三方云端 Agent、桌面 Agent 及 Agent CLI。它不是另一个 AI 助手，而是向 Agent 提供用户授权的个人工作上下文，以及受控操作本地电脑的能力。Yonder 的研发范围仅包含桌面插件应用；个人上下文云服务与云端 Agent 平台属于外部系统。

桌宠是 Yonder 的可视化控制界面，通过可定制角色呈现录制、执行、等待用户介入和完成等状态。用户决定何时录制上下文、哪些 Agent 可以访问上下文，以及允许执行哪些动作。

## 问题

Agent 能够理解和规划工作，却缺少一条稳定、由用户控制的通道连接真实 PC。办公文档、桌面应用、浏览器会话、文件和用户示范过的流程彼此割裂。现有桌面助手通常将执行能力绑定到内置 Agent，自动化工具则缺乏统一的个人上下文、权限和用户接管机制。

依赖 PC 完成工作的白领因此仍需重复执行文档密集型流程，或向不同 Agent 反复解释上下文。云端 Agent 无法安全触达本地设备，本地 Agent 之间也不能共享用户工作方式的持久记录。

## 产品定义

Yonder 提供三项相互连接的能力：

1. **显式上下文录制：** 默认不采集。用户主动开始 Record 后，记录用户操作时间线、文档与截图引用、浏览器上下文，以及应用和窗口状态；授权记录同步到配套的个人上下文云产品。
2. **面向 Agent 的本地执行：** 本地或远程 Agent 创建 Task Space，并通过独立的 BUA 或 CUA 执行路径观察、操作设备；两条路径共享任务、权限、事件和审计模型。
3. **可见且可控：** 桌宠展示执行状态，请求确认，并支持暂停、取消、接管和结果检查。

## 目标用户与工作场景

首批用户是依赖 PC 完成日常工作的白领，高密度场景包括：

- 创建、阅读、编辑文档，并在文档之间搬运信息；
- 操作 Microsoft Word、Excel、PowerPoint 和 WPS；
- 通过 Finder 或 Windows 文件资源管理器管理文件；
- 操作已登录的网页应用，并在浏览器与桌面软件间传递信息。

## MVP 主干链路

1. 用户在 Windows 或 macOS 安装 Yonder、登录账号并导入符合规范的桌宠动画资源包。
2. 用户连接第一方或第三方 Agent，并授予明确权限。
3. 用户主动开始 Record，示范一段文档或浏览器工作流程。
4. Yonder 采集授权范围内的时间线与上下文，并通过外部系统提供的协议同步到个人上下文云服务。
5. 获得授权的 Agent 查询指定历史记录并创建 Task Space。
6. Agent 通过基于 ego-lite 的 BUA 路径，或选定本地驱动后的 CUA 路径执行任务。
7. 遇到身份验证、敏感数据、对外提交、破坏性操作或执行歧义时，Yonder 暂停并请求用户确认或接管。
8. Agent 恢复执行，或由用户完成受阻步骤。
9. Yonder 完成 Task Space，并保留时间线、产物、结果和审计记录。

## Task Space 与权限模型

Task Space 记录发起 Agent、授权范围、生命周期、资源引用、事件、产物和审计历史。状态包括 `created`、`running`、`waiting-for-user`、`paused`、`completed`、`failed` 和 `cancelled`。

## 2026 年 10 月人机协作里程碑

10 月优先交付 Windows CUA 的人机协作闭环：任务状态可见、暂停/取消、用户接管、明确归还、归还后重新观察与局部重规划、结果确认和协作时间线；同时完成一次用户手动开启的 Record → 审阅 → Replay。详细范围见 `planning-artifacts/epics/EPIC-2026-10-HUMAN-AGENT-COLLABORATION.md`。

首批权限包括：

- `context:read`
- `device:observe`
- `device:control`
- `browser:control`
- `command:execute`
- `record:manage`

历史上下文读取与实时设备操作必须分别授权。即使任务来自外部云端 Agent，权限校验和实际执行仍由本地 Runtime 负责；Yonder 不实现云端服务端。

## 两条执行路径

**BUA：** MVP 直接使用 ego-lite/`ego-browser`，不自研浏览器和浏览器自动化引擎。浏览器上下文隔离时可并行执行；Yonder 负责授权、云端连接、上下文同步、状态与审计。

**CUA：** 对外提供统一的 Windows/macOS 动作契约，优先使用 Accessibility Tree 感知，截图作为回退，并遵循“观察—操作—验证”循环。前台焦点、鼠标和键盘操作需要单独授权；MVP 同一时间仅允许一个前台 CUA 任务。

首批结构化桌面适配范围为 Microsoft Word、Excel、PowerPoint、WPS、Finder 和 Windows 文件资源管理器。其他应用可使用通用 CUA，但不承诺结构化上下文质量。

## 桌宠

用户可以导入实现标准状态的动画资源包：`idle`、`listening`、`recording`、`thinking`、`executing`、`waiting_for_user`、`success`、`failed` 和 `paused`。桌宠是任务状态与控制界面，不是独立的内置 AI 助手。

## MVP 成功标准

首个里程碑以跑通主干链路为成功：第三方 Agent 在 Windows 和 macOS 上连接 Yonder，读取用户显式录制并同步的上下文，启动一个 CUA 或 BUA Task Space，完成一次用户接管与恢复，最终完成任务并留下可检查的审计记录。

## MVP 明确不做

- 默认或持续后台监控。
- 自研浏览器或 BUA 引擎。
- 自动将录制泛化为参数化流程。
- 可视化流程编辑器。
- AI 自动生成角色或动画。
- 对指定 Office、WPS 和文件管理器之外的应用做广泛结构化适配。
- 在主干验证前优化规模、并发或成功率。

## 长期方向

主干成立后，Yonder 将成为开放、用户可控的个人 Agent PC 能力层：不同 Agent 可以理解用户选定的工作历史，安全操作本地软件，并复用用户曾经示范的工作过程。

## 待决策事项

- 使用同一套 Windows/macOS 黑盒用例，对比验证 Qwen `cua-driver` 与 trycua `cua-driver` 后选择最终 CUA 后端。
- 云端到设备的传输协议与第三方 Agent 身份认证方式。
- Record 事件结构、保留周期、加密、脱敏与删除机制。
- 支持的系统版本，以及安装包签名与分发方式。
- 动画资源包格式与渲染约束。
