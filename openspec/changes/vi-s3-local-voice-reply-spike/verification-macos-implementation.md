# VI-S3 macOS 本地语音回复 Spike 实施侧验证

关联 Story：VI-S3  
日期：2026-10-14  
环境：macOS 26.0，arm64，Apple Swift 6.3.3  
结论：核心候选继续保留；完整 Verification Goal 仍为 PENDING

## 范围

本记录只覆盖隔离顺序夹具和静音音量的系统原生 TTS 控制面，不接 Agent、不修改正式 Desktop、不保存或播放用户正文。它不是非实施者独立验证，也不覆盖真实有声停止、断网、输出设备切换或 Windows。

## 命令

```bash
python3 openspec/changes/vi-s3-local-voice-reply-spike/fixtures/verify_spike.py
```

脚本在临时目录编译 Rust 顺序夹具和 Swift 原生探针，运行三轮后清理二进制。

## 结果

```json
{"contains_text_or_voice_name":false,"installed_chinese_voice_count":18,"kind":"vi-s3-macos-spike-verification","native_candidate":"pass","offline_verified":false,"ordered_contract":"pass","output_device_switch_verified":false,"queue_elapsed_ms_range":[968,1058],"requires_generation_guard":true,"runs":3,"stop_elapsed_ms_range":[0,119]}
```

- 乱序完成、失败序号跳过、旧 generation 拒绝和固定容量均通过。
- 三个原生 utterance 的开始与完成回调严格按 0、1、2 排列。
- `stopSpeaking(.immediate)` 三轮均被接受，`isSpeaking` 在 0～119ms 内变为 false，终态回调唯一，探针释放 synthesizer 引用。
- 本机停止后的原生终态是 `didFinish`，不是 `didCancel`。因此正式实现不得根据回调名称声明业务完成；停止时必须先令 generation 失效，随后只把原生回调作为资源释放事实。
- 输出不包含样本文字、声音名称、合成音频或完整错误 Payload。

## 未通过门禁

- 探针使用静音音量，尚未证明真实扬声器没有残余声音。
- 未在真实断网状态运行，不能声明完全离线。
- 当前未执行输出设备切换样本。
- 没有 Windows 环境与同样本证据。
- 尚未由非实施者执行独立 Verification Goal。

上述缺口关闭前，AD-VI-03 保持 Proposed，VI-S3 不得接入产品组合根或宣称完成。
