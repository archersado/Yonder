# 设计

Spike位于`spikes/macos-context-native-messaging/`。Swift原生探针只记录Bundle ID/PID有效性、窗口属性可用性、事件计数和资源释放；Accessibility未授权时返回稳定能力错误，不主动拉起授权或采用轮询。应用切换时旧AXObserver必须先清理，再绑定新PID。

Native Host保持独立可执行文件，完整读取32位本机字节序长度头与有界JSON后才响应。stdout只写Native Messaging帧；stderr只写固定诊断码。manifest使用绝对路径与精确扩展来源。复用Windows Spike的Manifest V3扩展协议边界，但macOS安装、实连和证据独立；不读取History数据库，`tab.incognito`在发送前拒绝。

统一验证脚本只生成布尔值、计数、版本和稳定结果码。真实浏览器步骤由验证者显式加载扩展并开始/停止；Google Chrome与Microsoft Edge分别计结论。当前环境未安装Edge，因此不得宣称双浏览器完成。
