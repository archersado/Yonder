# VI-S1 macOS 真实语音验证前置检查

日期：2026-09-26  
结论：BLOCKED（环境前置不足；不是能力通过或失败结论）

## 检查范围

以只读方式确认当前 macOS 是否能进行余下的真实“运行中撤权、输入设备切换、说话后停顿自动结束”样本。探针不请求系统权限、不打开麦克风、不保存 PCM、转写、设备名称或标识。

## 结构化结果

`swift spikes/voice-input/macos-readiness-probe.swift` 返回：

- `input_device_count: 1`，因此 `can_verify_real_device_switch: false`；缺少第二个真实输入设备，不能把配置变更通知或模拟事件记为设备切换通过。
- `microphone_permission: not_determined` 与 `speech_permission: not_determined`；尚未有用户显式授权会话，不能把只读探测当成允许、拒绝或运行中撤权证据。
- `requested_permission: false`、`opened_microphone: false`，并确认没有输出设备名称或标识。

## 已复核的实现子范围

- `cargo test -p yonder-desktop voice_input -- --nocapture`：4 项确定性语音状态/终态回归通过。
- `node apps/desktop/check-voice-session.mjs`：停顿后整段发送文案及单一 final 提交分支通过。
- `openspec validate vi-s1-voice-input-spike --strict`：通过。

## 恢复条件

1. 测试者在隔离的已签名 Yonder/夹具中显式点击开始，完成允许、拒绝和运行中撤权会话；不得以 `tccutil` 重置或模拟回调替代运行中撤权。
2. 接入第二个真实输入设备并在采集期间切换，保留无正文结构化结果。
3. 用公开统一声学样本验证“有效语音 → 短暂停顿 → 单次 final 投递”，同时覆盖初始静音和手停/静音竞态。

Windows 项按用户现行决定延后；本记录不改变 AD-VI-01 的 Proposed 状态，也不允许 Archive。
