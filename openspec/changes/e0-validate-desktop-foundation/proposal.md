当前归属 Story：DS-S1；规划：`docs/specs/epic-DS/story-DS-S1/README.md`。旧编号保留历史追溯。

# 提案：验证桌面基础栈

2026-09-14用户优先实施四状态素材接线：关联DS-S1 RUN-ASSET-01～04与AD-DS-01，Architecture Impact: conforming。仅修改正式apps/desktop内置UI与图片，原事实源/协议/持久化不变；完整生命周期与Windows门禁保留。

2026-09-15 收到请求子范围：关联 DS-S1 LISTEN-01～04 与 Accepted AD-DS-02。复用正式 Agent Gateway 成功 `task.create` 结果，在桌面组合根生成最长1.6秒的非持久化 listening 展示；不新增 wire/schema，不伪造其他生命周期状态。

## 为什么

Tauri、透明桌宠窗口与跨平台 Local Socket 是后续所有能力的底座，必须先用目标平台证据验证。

## 变更

创建一次性最小验证程序，验证窗口、托盘、IPC 和资源基线；不实现正式产品模块。

## 影响

- Story：E0-S1
- 架构影响：conforming
- 相关决策：[AD-E0-01](../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-E0-01-DESKTOP-FOUNDATION.md)（Proposed，待评审）
- 涉及模块：仅 Spike 代码和验证脚本
- 协议影响：仅验证性 `hello/echo`，不得视为正式协议
- 迁移影响：无

2026-09-14生命周期素材子范围：关联DS-S1三份设计ASSET-01～04、assets/mascot/互动动画生命周期-v1.md。Architecture Impact：none（只新增图片与声明式清单）；生成9种状态四帧图集并独立验证角色/alpha/边缘/清单。原图保留，不修改运行时状态/协议/持久化。完整UI生命周期与Windows仍未Done/Archive。

本批素材处理：已获用户明确Python/Pillow授权；原图保留，生成400×400透明帧和声明式播放清单，独立验证记录见verification-state-assets-v2.md；不修改运行时状态接线。
