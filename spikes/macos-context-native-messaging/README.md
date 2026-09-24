# macOS上下文与Native Messaging Spike

本目录只验证CX-S1的macOS技术路线，不是产品Context Adapter。所有输出只包含能力布尔值、事件计数和稳定结果码，不包含窗口标题、URL、页面标题、键值、坐标或完整Payload。

## 原生上下文探针

能力预检不会主动弹出系统授权：

```bash
./run-macos.sh --check
```

有界观察默认运行8秒。只有当前进程已获得Accessibility权限时才注册AXObserver；否则结果为`capability_unavailable`：

```bash
./run-macos.sh --observe 8
```

可重复的原生事件样本依次激活TextEdit、Calculator和Finder，验证应用激活、AXObserver重绑定尝试和资源释放；不读取用户窗口：

```bash
./check-native-events.sh
```

## Native Messaging协议自检

```bash
./check-protocol.py
```

覆盖UTF-8、分片输入、连续消息、32位本机字节序长度头、1 MiB上限、非法JSON和stdout隔离。

扩展权限、隐私守卫和精确来源静态检查：

```bash
./check-extension.py
```

## Google Chrome实连

构建并注册当前用户级Host：

```bash
./register-chrome-host.sh install
```

在`chrome://extensions`启用开发者模式，加载本目录的`extension/`。扩展ID固定为`mofdddjaniddgalgegfdjegiegpneokc`。打开扩展的`control.html`后显式开始和停止；状态页只显示连接与计数，不显示浏览内容。验证结束必须撤销注册：

```bash
./register-chrome-host.sh uninstall
```

固定扩展自检页`test.html`会发送开始、标签激活和停止三条无正文消息。Host只把消息类型、计数和隐私模式拒绝布尔值写入临时`browser-evidence.json`，不记录URL、标题或Payload；验证者确认后只提交脱敏结构化证据。

真实Google Chrome的隔离自动样本使用临时用户数据目录，运行测试扩展页并在退出时撤销Host注册：

```bash
./check-chrome.sh
```

扩展不申请`history`权限；隔离profile通过`isAllowedIncognitoAccess()`确认未授权隐私模式，并在发送前再次拒绝`tab.incognito`。当前验证环境未安装Microsoft Edge，Chrome通过不能代表Edge通过。
