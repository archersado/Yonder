# lifecycle-v11 语音聆听素材

- `voice-listening-source.png`：基于既有Yonda原图生成的透明原图；右前爪贴近右耳，头部朝爪子侧倾，翅膀收拢。
- `voice-listening-blink-source.png`：同姿态闭眼原图，只用于提取眨眼覆盖层。
- 运行时副本位于`apps/desktop/ui/runtime/lifecycle-v11/`，统一400×400透明画布并保留安全边距。

生成方式：内置ImageGen `precise-object-edit`。约束为保持角色身份、配色、比例和透明背景，禁止✓、庆祝、挥手、道具、裁切、断肢与棋盘背景。
