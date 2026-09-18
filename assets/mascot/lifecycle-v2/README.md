# Yonda 九状态素材 v2

依据：[互动动画生命周期-v1](../互动动画生命周期-v1.md)的九种状态动画；角色参考[既有小龙](../yonda-dragon-concept-v1.png)。使用内置 imagegen 生成，用户明确授权 Python/Pillow 去背景、拆帧与统一画布。全部原始图集保留，v3 为局部修订，未覆盖 v1/v2。

输出：`frames/` 中九状态各四张400×400 RGBA PNG，适配200×200逻辑窗口。帧序由[asset-plan.json](asset-plan.json)声明；原始图集顺序为左上、右上、左下、右下。录制红点独立叠加，不能随角色动画消失。素材包仅使用 PNG 与声明式 JSON；prepare_frames.py 是仓库研发工具，不随运行时素材发布。

| 状态 | 节奏 | 原始图集 |
| --- | --- | --- |
| idle | 呼吸3.6秒；眨眼另按6–10秒触发 | [待命](idle-v2.png) |
| listening | 350毫秒入场后保持 | [收到请求](listening-v3.png) |
| thinking | 2.4秒轻歪头循环 | [等待响应](thinking-v3.png) |
| executing | 1.4秒左右摆爪循环 | [执行](executing-v3.png) |
| waiting_for_user | 500毫秒入场后保持，弱提示最多每8秒一次 | [等待用户](waiting_for_user-v2.png) |
| paused | 300毫秒过渡后保持，慢呼吸独立播放 | [暂停](paused-v2.png) |
| recording | 专注姿态、尾巴轻摆；红点稳定，外围可2秒变化 | [录制](recording-v2.png) |
| success | 1.2秒单次，小跳不超过8逻辑像素 | [成功](success-v2.png) |
| failed | 1.4秒单次，不自动重试 | [失败](failed-v2.png) |

生成提示词：[初版](prompts.json)、[修订](edit-prompts.json)。思考图集后两帧存在角度/朝向偏差，未直接采用；基于第一帧生成±3°轻倾候选，保留相同角色朝向。该简化仍需动画目测确认，不宣称独立头部局部动作已通过。收到请求使用前两帧后保持，避免后续大角度歪头。

独立验收：[Verification Goal](../../../openspec/changes/e0-validate-desktop-foundation/verification-state-assets-v2.md)、[逐帧结构检查](frame-check.json)。本批不修改运行时引用和任务状态接线，不将素材静态检查称为双平台原生动画通过；Windows按用户要求暂缓。

2026-09-14用户真实桌面回归：本批背景格子残留，静态验收结论已拒绝，不再进入运行时。合格透明副本改用../lifecycle-v4/；本目录原图仅保留追溯。
