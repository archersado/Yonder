# 独立 Verification Goal：九状态小龙素材

日期：2026-09-14。关联：DS-S1 ASSET-01～04；e0-validate-desktop-foundation；AD-E0-01。范围仅图片与声明式播放数据，不是UI状态接线验证。

## 验收与证据

- ASSET-01：沿用既有奶白浅蓝小龙、Y形龙角、蓝金双眼、卷尾与软3D质感；原始v1/v2保留，三种v3修订另存。
- ASSET-02：九状态图集已生成。执行v3第四帧改为图像左爪举起，第二帧为右爪；listening选择前两帧入场保持。thinking生成器后续帧大角度/镜像被拒绝，使用首帧±3°倾斜候选；尚未证明独立头部动作与尾尖动画满足完整设计。
- ASSET-03：本地用户明确授权Python/Pillow处理。36帧400×400 RGBA，透明/不透明像素并存，阈值可见边界位于安全区；见`assets/mascot/lifecycle-v2/frame-check.json`。首轮高光误抠除失败已返回处理阶段，不采用失败帧。深浅背景目测检查见`preview-light-dark.jpg`。
- ASSET-04：九状态清单与生成/修订提示词已保存。暂停过渡后保持，成功/失败单次，录制标记独立。工具脚本不属于发布素材包。

## 结论与限制

结构透明检查通过。静态角色候选已提供；边缘细节、动画平滑度与完整动作语义仍需后续验收，不把alpha存在等同精细抠图通过。未接入运行时、未重启桌宠，未产生或修改任务；不声明真实执行动画已修复。Windows原生验证按用户要求暂缓。整体Story不Done/Archive。

复现处理：安装或复用Pillow后执行`python3 assets/mascot/lifecycle-v2/prepare_frames.py`，内置断言检查36帧alpha、尺寸与安全边距。处理需人工复查，不能将本工具的形态学分割用于任意新背景。
