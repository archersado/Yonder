# VI-S1 macOS 会话收口独立 Verification Goal

日期：2026-09-25
结论：PASS（实现与确定性状态机子范围）；真实麦克风声学样本未通过

## 目标

独立验证 macOS 候选实现满足以下实现契约：初始静音不触发静音收尾；检测到持续有效语音后，短暂停顿只请求一次收尾；手动停止、静音和总时限竞争时只有一个终结者；同一会话最终转写只领取一次，已结束会话的迟到回调不能进入下一会话；界面明确提示停顿后发送整段。证据不得记录 PCM 或转写正文。

## 验证结果

- Objective-C 纯状态样本验证：低于门槛的初始静音持续 1.25 秒不触发；幅度高于候选门槛并持续 0.24 秒后，累计 0.90 秒静音在预期边界触发。
- 四个模拟结束信号依次竞争同一个原生门闩，只有第一个被接受。
- Rust 会话样本验证旧 `session_id` 的 final 被拒绝；当前会话的同一 final 第一次被领取、第二次被拒绝。
- `session_id` 由 Rust 每轮生成并穿过 Objective-C 回调；取消或新会话开始后，旧 ASR 回调不能复用当前投递目标。
- 直接语音卡和圈选语音均显示“正在聆听，说完短暂停顿后自动发送整段”，不再以 20 秒总时限冒充静音结束；圈选入口仍只有一个 final 提交分支。
- 当前分支桌面二进制完成构建；以编译期隔离 identifier 生成的临时签名包 `com.yonder.voice-session.fixture` 通过 `codesign --verify --deep --strict` 并由 LaunchServices 成功启动，只创建专属 Application Support 目录。验证后仅终止夹具 PID，专属数据可恢复地移至 `/private/tmp/yonder-voice-session-fixture-data-20260925`；正在运行的正式 Yonder 未停止或改写。

结构化证据：`apps/desktop/evidence/vi-s1-session-finalization-macos-20260925/result.json`。

## 执行命令

```text
cargo test -p yonder-desktop voice_input -- --nocapture
node apps/desktop/check-voice-session.mjs
openspec validate vi-s1-voice-input-spike --strict
cargo build -p yonder-desktop
python3 apps/desktop/package-macos-observation-fixture.py voice-session
codesign --verify --deep --strict "apps/desktop/target/preview/Yonda Voice Session Fixture.app"
```

## 范围限制

本轮自动化能确认隔离正式宿主加载新二进制，但系统合成指针没有触发透明 WebView 中仅悬停显示的语音按钮，故未产生新的真实麦克风声学样本。不得用纯状态样本代替“真实说话→停顿→自动结束→整段单条投递”验收；该项继续保持未勾选。首次权限拒绝、运行中撤权、真实设备切换、Windows 统一样本和 AD-VI-01 终局结论同样未覆盖，完整 VI-S1 不得标记 Done 或 Archive。
