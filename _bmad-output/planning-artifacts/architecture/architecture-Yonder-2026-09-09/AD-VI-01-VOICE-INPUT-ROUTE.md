# AD-VI-01 单轮语音输入技术路线

状态：Proposed  
日期：2026-09-17  
关联：VI-S1、CX-S2、VI-S2

## 决策问题

Yonder需要在Windows/macOS上提供用户显式启动的单轮语音转写，同时保持Application状态所有权、原始音频有界、停止可验证且不引入硬编码云密钥。参考插件只证明交互可行，其Electron、讯飞账号和Node状态机不能直接成为产品架构。

## 候选路线

1. 平台原生音频与语音框架：macOS `AVAudioEngine + Speech`，Windows音频捕获与`Windows.Media.SpeechRecognition`。依赖最少，但语言包、部分结果、离线性和桌面应用可用性必须实测。
2. Rust统一音频采集加单一本地ASR：候选仅在原生语音不能满足统一契约时评估；模型体积、签名、启动和内存必须有界。
3. WebView `getUserMedia`只作淘汰对照；若权限、后台生命周期或UI状态所有权不稳定则淘汰，不让React成为采集事实源。

## 不变围栏

- 没有用户显式开始时不得请求权限或开启麦克风。
- 采集与ASR通过Application Port驱动；平台Adapter不创建任务、不访问SQLite、不调用Gateway。
- 原始PCM默认不落盘；普通日志、事件和Outbox不记录音频或完整转写。
- 不复制`learn/avatar-orb-pet/config`中的凭据，不把讯飞或任一供应商写死为系统边界。
- Windows/macOS统一样本全部通过前保持Proposed，不生成产品实施Proposal。
- Spike交互必须复用Yonda桌宠进程：入口位于小龙，收音卡与小龙同Space并随其定位。语音采集使用仅影响显示的`voice_listening`临时态，复用抬头侧耳素材但显示麦克风提示；不得复用任务“收到请求”的`listening`确认标记或改变任务状态。允许在debug预览中接入候选Adapter，但不得把独立测试App作为产品入口。

## Spike淘汰门槛

候选必须支持显式开始、部分/最终结果、取消、停止后资源释放、权限拒绝/撤销、设备切换和稳定错误；任一平台需要常驻第二App、隐藏云凭据、无法确认停止或无法满足中文输入时淘汰。

## 2026-09-21 macOS 圈选组合定稿

用户已明确最终语音无需二次确认。普通语音继续在点击小龙麦克风时授权一次直接投递；圈选确认卡已经显示当前截图与发送范围，因此点击卡片内麦克风同样授权最终非空转写携带当前PreviewSession附件直接提交。部分转写只投影到当前卡片，不交给Agent。

Desktop在开始采集前以可信窗口入口固定本轮目标为`direct`或`region`；平台Adapter仍只产生语音事件，不访问Gateway。`direct`复用既有voice输入，`region`复用CX-S2现有selection提交与附件清理，不新增协议、SQLite、任务状态或第二份截图所有权。取消、关闭、重新圈选、超时及进程退出都结束region采集；最终事件只能消费一次，不能同时产生voice与selection两条输入。

已有macOS显式采集、中文转写、自动投递、停止释放与正式桌宠同进程证据允许该受限组合增量；Windows继续按用户决定暂缓，完整VI-S1及双平台结论仍保持未完成。
