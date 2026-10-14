# 任务

- [x] 完成 VI-S3 三份设计、来源映射与 Proposed AD-VI-03
- [x] 建立 Spike proposal/design/delta spec
- [x] 实现并自检 generation/sequence 有界顺序夹具
- [x] 实现 macOS 系统 TTS 无正文原生探针
- [x] 验证 macOS 中文声音发现与顺序回调
- [ ] 验证 macOS 播放中停止、迟到回调和资源释放
  - [x] 静音音量控制面三轮验证：停止为0～119ms、唯一终态回调、资源释放，并确认必须使用generation门禁
  - [ ] 真实有声输出停止与无残余声音验证
- [ ] 验证 macOS 离线运行与输出设备切换
- [ ] 建立 macOS 独立 Verification Goal
  - [x] 形成可重复的实施侧验证脚本与无正文证据
  - [ ] 由非实施者复核运行并给出独立结论
- [ ] 在 Windows 运行同一样本，覆盖声音发现、顺序、停止、离线、设备切换和释放
- [ ] 根据双平台证据接受、修订或拒绝 AD-VI-03
- [ ] ADR 接受后另建产品实施 Proposal
