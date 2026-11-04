# VI-S1 Windows 显式语音候选验证（2026-11-02）

结论：**FAIL（显式授权与隐私闸已通过；真实麦克风可打开，但统一样本没有识别结果且优雅停止超时）**。

## 执行范围

- 用户在当前Agent会话中明确授权本轮语音测试；探针新增等价的`--authorized-start`入口，以解决当前宿主无法把交互式`START`转发给原生控制台的问题。默认入口仍保持未输入`START`即退出且不打开麦克风。
- 探针使用`Windows.Media.SpeechRecognition`与系统TTS，计划最多采集12秒，只输出计数、布尔状态和HRESULT，不打印或保存PCM、转写正文、设备名。
- 未执行CUA，未提权，未修改Windows隐私设置。

## 实际结果

- 编译PASS，构建阶段未打开麦克风。
- 显式启动到达Windows语音API后立即返回`0x80045509`；微软将该值定义为语音隐私声明未接受/联机语音识别关闭。
- `opened_microphone=false`、公开测试句未合成或播放、部分/最终结果均为0；因此本轮没有真实采集，也没有需要证明的停止后资源释放。
- 当前用户与机器级麦克风ConsentStore均为`Allow`，`Audiosrv`与`AudioEndpointBuilder`均在运行，用户语言为`zh-Hans-CN`。语音Capability枚举需要管理员权限，本轮未提权。
- 两次交互式标准输入转发尝试均被探针的拒绝闸安全拦截，均为`opened_microphone=false`；它们不计作真实采集尝试。

结构化证据：`spikes/voice-input/evidence/windows-20261102/explicit-capture-result.json`。

微软说明：<https://learn.microsoft.com/windows/uwp/ui-input/speech-recognition>；最新Windows App SDK说明还要求`Windows.Media.SpeechRecognition`调用方具有MSIX包身份，故开启联机语音识别后仍须验证正式打包身份，不能以未打包探针直接接受Windows路线。

## 通过条件与下一步

1. 用户已在同日开启“联机语音识别”，只读状态为`HasAccepted=1`，并再次明确授权。
2. 复测成功构造识别器、编译约束、启动会话、打开麦克风、合成并播放公开句；12秒内`partial_count=0`且`final_count=0`。随后`StopAsync`超过10秒，探针的有限时版本返回`0x800705B4`。
3. 首次未加超时的复测曾超过设计上限，已精确终止PID 61028并确认无残留探针进程。ConsentStore没有记录这个未打包探针的本次起止时间，因此仅凭进程退出不能把系统级释放门禁判为PASS。
4. 初版回退在尚未结束的`StopAsync`上并发调用`CancelAsync`，返回`0x80131509`（对象状态不允许）；这是探针次生错误。回退已改为只取消原停止操作、有限等待并失败退出，之后只重编译和复测拒绝闸，不再重复打开麦克风。
5. 复测证据见`spikes/voice-input/evidence/windows-20261102/explicit-capture-retest-result.json`。当前未打包候选不满足统一音频与停止释放淘汰门槛；下一步应创建最小MSIX身份Spike，使用同一公开样本比较识别与停止行为，不得直接进入产品实现。
6. 仍需另行覆盖权限拒绝、运行中撤权、设备切换和停止后系统录音指示消失，完整任务保持未完成。
