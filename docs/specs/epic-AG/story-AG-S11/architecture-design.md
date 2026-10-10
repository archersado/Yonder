# AG-S11 架构设计

## 边界与依赖

不改变协议或状态所有者。Connector、AgentInputHub 和 Tauri 继续传递有界纯文本 delta；`region-preview.js` 在 WebView 内累计原文并投影安全 DOM。Desktop 不新增 npm 运行依赖或网络加载。

## 状态与契约

- 当前卡片持有最多 64 KiB 的 `replyMarkdown`，严格跟随 AG-S9 的 input/session/sequence 门禁。
- 每个 delta 追加到累计字符串后重新渲染，保证跨 delta 的 Markdown 语法能在补齐后生效。
- 渲染器只使用 `document.createElement` 与 `textContent` 创建白名单节点，不使用 `innerHTML`。
- 链接仅接受 `http:`、`https:` 和 `mailto:`，显示标签与规范化目标；首批不在 WebView 内导航。图片语法只显示替代文本和安全占位，不发起资源请求。

## 失败与验证

解析异常必须回退为完整纯文本，不得影响回复序号或终态。测试覆盖块级/行内语法、跨 delta、HTML/脚本/图片/危险 URL、防 `innerHTML` 合约和既有回复终态；正式 macOS bundle 验证可读布局与复制选择。
