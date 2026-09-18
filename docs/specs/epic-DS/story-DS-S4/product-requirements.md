# DS-S4 产品需求

## 问题与目标

用户需要上传自定义形象，生成完整且一致的 Yonder 状态动画，并导入可定制的小龙资源包。导入失败不能破坏正在运行的内置形象，也不能把压缩包携带的脚本或路径写入当作可信资源。

## 范围与非目标

本 Story 定义“自定义形象 → Hatch Pet 式生成 → Yonder 状态包 → 本地导入”链路。资源包不包含 JavaScript、二进制、字体、网络地址或任务逻辑；Yonder 不内置图像模型、密钥、Planner、在线市场、同步或多角色管理。

## 验收条件

| AC | 可观察结果 | 来源 |
|---|---|---|
| PACK-01 | 用户从本机选择一个 ZIP；仅完整且符合声明格式的 PNG/WebP 包可导入 | 产品简报「MVP 主干链路」步骤1、桌宠 |
| PACK-02 | 包含 `manifest.json`，声明格式版本和九个标准状态的静态帧或循环帧；缺失、重复、路径穿越、非图像文件或超限均拒绝 | 架构主干「Recording 与桌宠」 |
| PACK-03 | 校验在暂存目录完成；全部通过后才原子替换当前包，失败继续使用上一个可用包 | 架构主干「非功能与发布」的失败不伪报成功与磁盘保护 |
| PACK-04 | 导入后桌宠按现有真实生命周期选择对应状态；资源包不创建任务、不改变任务状态、不触发 Agent 行为 | 产品简报「桌宠」、架构主干任务状态唯一事实源 |
| PACK-05 | 入口、处理中、成功、拒绝和回退均有可见且可访问反馈；减少动态效果显示各状态静态帧 | 产品简报可见且可控、用户轻量桌宠变更 |
| GEN-01 | 用户可选择1～5张自定义形象参考图和可选性格描述；外发给生成 Agent/模型前必须明确确认 | 用户最新变更、产品简报默认不采集 |
| GEN-02 | Yonder 定义九个标准状态及动作语义；生成结果覆盖全部状态，不能由生成器自定义任务含义 | 用户最新变更、产品简报桌宠、AD-DS-04 |
| GEN-03 | 生成流程先固定角色身份，再逐状态生成与质量检查；单个不合格状态只定向重生成 | Hatch Pet 方法、AD-DS-04 |
| GEN-04 | 未明确外发、Agent/Skill不可用、取消或生成失败时，不上传参考图、不替换当前形象 | 产品简报默认不采集、架构 Agent 边界 |
| GEN-05 | 生成包只有通过 Yonder manifest、安全与运行时预览校验后才可激活 | DS-S4 PACK-01～05、AD-DS-04 |

## 资源包建议格式

根目录仅允许 `manifest.json` 与 `assets/`。manifest 声明 `format_version: 1` 和 `states`；每项为相对 `assets/` 的 PNG/WebP 帧列表。标准状态为 `idle`、`listening`、`recording`、`thinking`、`executing`、`waiting_for_user`、`success`、`failed`、`paused`。帧文件必须唯一、有限且无目录外引用；具体字节、尺寸、帧数和解压总量限额由 OpenSpec 定稿。Hatch Pet 图集与 `pet.json` 是生成中间格式，必须转换为此 manifest 才能导入。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)「MVP 主干链路」步骤1、「桌宠」。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)「Recording 与桌宠」「非功能与发布」。
- 后续用户变更：小龙为透明轻量桌面插件，状态动作以 `assets/mascot/互动动画生命周期-v1.md` 为准；此 Story 不改变状态来源。
- 2026-09-18 用户变更：参考 Hatch Pet，让用户上传自定义形象并生成 Yonder 定义的状态动画；生成委托、外发确认和产物归一化由 Proposed AD-DS-04 限定。
