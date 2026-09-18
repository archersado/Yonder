# 独立 Verification Goal：九态连续局部动作 v7

关联 DS-S1 三份设计的“专属动作可辨认”和“需要流畅的动画”用户变更，2026-09-14。

## 结论

九态预览的连续局部变换与基础呼吸/眨眼检查 PASS；不代表原生桌宠、五态真实事件接线或主 Story Done。Windows仍暂缓。用户尚未认可最终自然程度，不以本次采样宣称“完美”。

## 失败与修复

v6只有整体轻晃，用户否决专属动作。v7初次四帧失败头部混合截图出现重影，返回实施，改为连续头部层。连续分层首次检查发现 listening maxMix=0.999996：相同姿态索引仍计算混合权重，头部身体补片与整图重叠。修正为相同索引mix=0。

保留全部失败目录，最终证据位于 apps/desktop/evidence/state-assets-v4-20260914/nine-state-continuous-v7-retry，result.json、top.png、bottom.png、light.png。深浅背景截图未见此前失败状态的双头重影。原生运动的所有肩/颈接缝未完整取证，不将静态截图当作流畅度实机结论。

## 可复现检查

生成 `python3 apps/desktop/build-animation-preview.py`；同一 ego-browser space 21/p1，导入 apps/desktop/check-animation-preview.mjs 调用 checkAnimationPreview。连续上方观察17秒，下方11秒，各自至少两轮。

九态均检查呼吸变化、完整闭眼采样、maxMix=0。收到请求/等待响应/失败额外检查头部矩阵变化；等待用户/暂停/成功检查爪或翼的局部矩阵变化；执行检查左右爪角度变化。验证使用真实时钟，不注入进度、任务或录制事件。完成最后补充余弦起落以保证落地速度归零，其结构检查随后再次执行。

Node语法检查及git diff --check通过。产品完整交互回归和原生 v7 编译/重启仍待执行。
