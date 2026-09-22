# 提案：统一任务总览首批只读展示

GUI首批工程审阅通过（2026-09-14）：按AD-DS-01增量，正式桌面crate复用既有Tauri路线，托盘“任务总览”与独立轻量面板、真实app_data_dir任务源及本地窗口校验进入Apply。保持E0小龙原状，仅一次研发时间盒并行预览；不发布双应用、不复制执行栈。依赖复用既有Spike版本，无HTTP/执行器/新协议。原文早期入口未决状态为历史，当前入口以DS-S2视觉设计为准。

2026-09-14宿主核心子范围Apply审阅通过：按AD-DS-01/AD-ST-01，先实施apps/desktop组合根库的真实未加密SQLite、标准文件锁、恢复及可信本机只读查询。不创建UI或外部传输，入口未决不阻断独立核心子范围；下述GUI仍待入口与窗口权限审阅，不授权其Apply。

Story：DS-S2，关联docs/specs/epic-DS/story-DS-S2/三份设计及DS-S2-CONTRACT-REVIEW.md。决策：AD-DS-01、AD-OCT-03、AD-OCT-02。

问题：用户需要查看所有真实并行任务，现有桌宠没有总览入口。首批展示可信Application查询返回的任务ID、Agent、状态、sequence；进行中/全部分页、刷新及详情，区分能力未提供、读取失败、空列表与过期快照。保留名称/步骤/等待原因、外部浏览器引用、控制及历史的完整Story需求，未提供字段明确反馈。

Architecture Impact：conforming（遵循已更新的阶段依赖决策）。模块：DS展示、桌面组合根；依赖Application查询与既有Rust派生协议，不改schema、wire协议或任务事实源，无迁移。不将产品追加至E0 Spike，不开放HTTP，不复制ego-lite Task Space。

状态：macOS首批Apply与独立验证已完成；可信本机身份、真实未加密SQLite、恢复/准入与轻量菜单均有证据。依Accepted AD-ST-01，MVP显式使用未加密SQLite，不自动修改已有加密库。Windows依用户决定暂缓，完整Story及Archive仍受跨平台与关联控制能力门禁约束。加密与明文迁移延期MVP之后，归ST-S2。

2026-09-15 当前步骤展示增量：关联 DS-S2 STEP-UI01～03 与 Accepted AD-AG-04。详情只读已实施的 `task.step.get`，不新增协议、持久化或步骤编辑入口。

2026-09-23 右键入口增量：依用户变更将鼠标入口由悬停自动打开改为右键显式唤起；悬停仅保留状态表现。复用既有本地菜单命令、真实未结束任务查询和面板定位，删除悬停轮询/计时，不新增协议、持久化或第二状态所有者。macOS原生与前端回归已通过，Windows仍暂缓。
