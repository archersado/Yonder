# Design：AG-S12 圈选回复卡呈现与拖拽

`safe-markdown.js` 在列表块中容忍项目之间的空行，但只有下一非空行仍匹配相同列表类型时才继续，防止跨段误合并。列表仍只由白名单节点构造。

`region-preview.html` 增加明确拖动区，`region-preview.js` 复用桌宠模式：主指针移动超过 4 点后先释放 pointer capture，再调用 Tauri 原生 `start_dragging`。关闭按钮不位于拖动区，输入和回复内容不绑定拖动事件。Capability 只向 `region-preview` 开放该单一命令。

测试先复现松散列表被拆分和卡片无拖动权限，再验证 DOM、权限、事件目标与正式窗口位置变化。
