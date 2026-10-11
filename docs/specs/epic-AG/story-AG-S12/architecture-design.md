# AG-S12 架构设计

## 边界与依赖

变更仅位于 Desktop WebView 与 Tauri 窗口能力配置。`safe-markdown.js` 继续使用 DOM 白名单；`region-preview.js` 复用桌宠已经验证的 Tauri `start_dragging` 模式触发原生拖动。Gateway、Application、Protocol 与持久化不变。

## 状态与契约

- Markdown 渲染器在块解析阶段把同类型列表及项目间空行归并为一个列表，不把空行渲染成虚假项目。
- CSS 显式声明 `disc`、`decimal`、marker 颜色和列表缩进，避免宿主默认样式差异。
- 确认卡增加只包含卡片身份与标题的拖动区；主指针移动超过 4 点后先释放 WebView pointer capture，再调用 `plugin:window|start_dragging`，避免 WebView 截获后续原生拖动事件。
- `region-preview` capability 只增加 `core:window:allow-start-dragging`，不开放移动坐标、缩放或其他窗口命令。

## 失败与验证

拖动命令失败时保持卡片原位，不改变回复或提交状态。测试覆盖松散列表 DOM、安全渲染、拖动权限最小化、拖动命令触发与交互控件排除；macOS 正式宿主验证窗口确实移动且内容仍可操作。
