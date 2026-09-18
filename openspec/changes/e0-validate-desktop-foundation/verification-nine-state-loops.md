# 独立 Verification Goal：九态专属循环 v6

日期：2026-09-14。关联 DS-S1 三份设计「用户变更：九态专属循环」及生命周期规格。

## 目标与结论

九态独立预览视觉采样 PASS；产品 Story 保持实施中，不 Archive/Done。Windows 按用户要求暂缓，原生 v6 未完成验证/重启，浏览器结论不覆盖原生桌面或五个尚未接线状态的真实事件来源。

## 可复现检查

先执行 `python3 apps/desktop/build-animation-preview.py`。在已有 ego-browser TaskSpace 中导入 `apps/desktop/check-animation-preview.mjs`，调用 `checkAnimationPreview(page, output)`；不得额外创建恢复用 TaskSpace。本次使用 space 21 / p1。

上方六态连续观察17秒，下方三态连续观察11秒，覆盖各自主循环至少两轮。每态采样确认 ready、呼吸变化、闭眼覆盖度超过0.95、专属动作或两爪角度变化。检查通过不等同于用户认可动作自然程度。

证据：`apps/desktop/evidence/state-assets-v4-20260914/nine-state-loops-v6/result.json`、`top.png`、`bottom.png`、`light.png`。人工查看深浅背景截图未见棋盘格、尾部裁切线或角色画布裁切。执行肩部动态连接仍需原生连续观察，静态截图不能证明每一时刻均无接缝。

## 边界与保留事项

等待保持举爪关键姿态后循环轻点头；暂停保持收翼关键姿态后循环缓慢放松回位，没有反复展翼动画。五态仅预览模拟，不创建任务、不启用录制。动画循环不重复任务事件、不自动恢复。

Node语法检查、预览Python语法检查、`git diff --check`通过。Python首次检查受默认缓存目录权限限制，获得升级执行后通过。尚未执行完整产品交互回归与原生 v6 验证，因此不宣称正式桌宠功能全部验收通过。

## 后续用户反馈：撤回视觉动作验收

用户确认除待命/执行外专属动作不可辨认。以上 PASS 仅证明轻微 transform 和闭眼采样，不满足专属动作要求；返回实施。v7 连续局部动作验证见 verification-nine-state-continuous.md。
