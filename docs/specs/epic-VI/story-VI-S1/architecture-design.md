# VI-S1 架构设计

## 边界与依赖

Application定义语音会话用例与`VoiceInputPort`；Windows/macOS平台音频和ASR实现位于Adapters。Desktop组合根只负责权限入口和视图投影。Adapters不得互调，语音Adapter不得直接访问任务SQLite、Agent连接或桌宠UI。

提交转写给Agent会改变Gateway双向协议边界，按AG-S5与AD-VI-02建立Agent输入通道，并由Rust协议类型生成Schema和TypeScript。该请求不是`task.create`；Agent是否创建任务由Agent决定。

## 状态与契约

候选内存状态为`idle → requesting_permission → listening → submitting → submitted|failed|cancelled`，由Application拥有；不进入编辑确认态。单设备同一时间最多一个语音采集会话，不与RC Recording混用状态或存储。

PCM进入有界环形缓冲区并流向ASR，停止后立即清理。最终转写只驻留至Agent确认接收或本次失败提示关闭；若后续需要历史或草稿持久化，必须另建Story与隐私决策。

## 双平台技术路线门禁

统一样本必须同时验证macOS和Windows的原生麦克风采集：启动/停止、设备切换、静音、部分/最终结果、超时、断网、资源释放和打包。参考插件的讯飞WebSocket与`sherpa-onnx-node`不自动成为Yonder依赖，禁止复制硬编码密钥。

候选优先使用系统原生音频API；ASR本地/远端路线通过Spike选出唯一实现。任何一端缺少等价能力时协议返回`capability_unavailable`，不得用另一套长期执行栈顶替。

## 失败与验证

权限、设备和网络变化通过事件驱动，不轮询数据库。Agent断连或交付超时不得自动重试；界面保留本次文字并提供“重试”或“取消”。日志只记录会话ID、`input_id`、阶段、耗时和错误码，不记录正文。

Windows与macOS分别使用同一公开测试音频验证允许、拒绝、撤权、设备切换和停止释放；证据不保存PCM或转写正文。

## 架构影响

新增语音技术模块与未来Gateway的用户请求方向。双平台结论前不得生成产品实施Proposal；Spike允许在debug桌宠预览中接入候选Adapter，以验证真实入口、同Space贴边卡片和`listening`状态投影，但不得形成第二App、持久化或Gateway协议。
