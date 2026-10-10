# Design

## 决策

新增无依赖、DOM 白名单 Markdown 渲染模块。模块先解析块结构，再以有限行内标记构造节点；所有普通内容最终通过 `textContent` 写入。回复卡保存累计 Markdown，每个严格序号 delta 后重新渲染，因此跨帧标记可恢复。

链接只做安全文本投影，不触发 WebView 导航；图片不创建 `img`。渲染器抛错时清空当前 DOM 并用 `textContent` 显示完整累计原文。

## 非目标

不引入通用 HTML sanitizer、远程资源、语法高亮器、Mermaid、数学公式、完整 CommonMark 或回复持久化。
