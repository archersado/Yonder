# VI-S2 架构设计

## 边界与依赖

复用VI-S1的Application语音会话与ASR Port，新增Windows/macOS系统音频来源Adapter；双路音频在Application按来源合并为转写时间线。Adapter不写文件、不调Agent、不识别会议语义。

导出经FI/Document Port，纪要请求经未来可信Gateway用户请求契约；任务由Agent创建。Desktop只展示会话快照，不持有转写事实。

## 状态与契约

会话状态候选为`idle → starting → listening → stopping → review|failed`，每个音源独立拥有`starting|active|muted|silent|failed|stopped`。原始PCM只在有界队列内短暂存在；转写时间线的持久化、保留和删除策略必须在实施ADR中明确。

## 双平台技术路线门禁

统一Spike覆盖macOS 14.4+ Process Tap候选、Windows WASAPI loopback候选、麦克风并用、蓝牙通话模式、设备切换、权限撤销、双路时钟对齐、背压和进程退出。ASR服务与凭据接线单独决策，不使用参考仓库内密钥。

两平台必须产出相同的来源、时间和错误分类；平台差异由Adapter吸收。若系统版本不支持系统音频，能力明确降级为仅麦克风，不能用屏幕录制或虚拟声卡偷偷替代。

## 失败与验证

开始前明确列出将采集的音源；系统安全界面与用户排除应用不采集。任一路失败不得拖死另一条，但完整性缺口必须进入会话结果。未知外部提交不自动重试。

## 架构影响

会议音频是产品简报之外的新采集类别，必须先更新Architecture Spine与权限能力表，再创建OpenSpec。VI-S1、FI-S1和Agent用户请求协议是前置门禁。
