# macOS Verification Goal：圈选语音直接提交

日期：2026-09-21
状态：PASS（独立复核）

## 实施者证据

- 正式`com.yonder.desktop` bundle在无截图确认卡中完成真实语音转写，只发送1条`agent.input`；`source=selection`、无附件帧、无普通voice帧、无额外帧。
- 同一bundle在带截图确认卡中完成真实语音转写，只发送1条`agent.input`与3条有界附件帧；附件字节数和SHA-256校验通过，输入引用同一临时附件，无普通voice帧、无额外帧。
- 收音中点击取消后窗口与PreviewSession清场，Agent收到0帧。
- direct/region最终转写的可信一次性路由由Rust定向测试覆盖；本Change未修改既有`deliver_voice`协议或Agent输入用例。普通语音原生回归未在人工窗口内取得转写，不计为原生PASS，需独立复核时再次发声验证。
- 三条结构化结果位于`apps/desktop/evidence/cx-s2-voice-submit-macos-20260921/`，不含转写正文、截图、base64、哈希或完整Agent Payload。

## 自动检查

- `cargo test --workspace --locked`：60项通过。
- `openspec validate cx-s2-voice-submit --strict`：通过。
- `python3 scripts/check_architecture.py`：通过。
- Swift、Python与JavaScript语法检查、`git diff --check`：通过。

## 独立复核要求

非实现者须从当前提交独立重建正式bundle，至少复跑带截图、无截图、取消和普通direct语音。前两者各只能产生一条selection输入，取消为0帧，direct只能产生一条voice输入；任一路径不得双投递。复核证据不得保存转写正文或截图。

Windows按用户决定暂缓；多显示器与云端产品WSS不在本Change范围，完整CX-S2继续保持`verifying`。

## 独立复核结论

非实现者在提交`7048c2a58ed1130f2ef6b7ca8cc448b2ba7f9670`上重新构建并签名正式`com.yonder.desktop` bundle；二进制SHA-256为`ec1b903391f7567d0bfb6928c60d5c96f45abb52dd7d2bc17cedae9ae2f1e5fe`。使用同一正式bundle独立复跑四条macOS本地UDS路径：

- 带截图语音：1条selection输入、0条voice输入、3条附件帧，附件长度与摘要校验通过，无额外帧。
- 无截图语音：1条selection输入、0条voice输入、0条附件帧，无额外帧。
- 收音中取消：Agent收到0帧，窗口与PreviewSession清场。
- 普通direct语音：1条voice输入、0条selection输入、0条附件帧，无额外帧。

四条路径均使用真实macOS语音转写，复核结果未保存转写正文、截图、base64、摘要或完整Agent Payload；应用数据目录、正式bundle日志与提交内证据均未发现固定测试转写正文。`cargo test --workspace --locked`共60项通过，严格OpenSpec、架构门禁及Swift/Python/JavaScript语法检查通过。结论为PASS，允许Archive本Change；Windows、多显示器和云端WSS仍按既定边界保留，完整CX-S2不因此转Done。
