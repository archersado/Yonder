# 设计

Application的`reviewing`允许附件为空；从`selecting`直接进入无附件review，或在`capturing`失败后进入无附件review。`begin_submission`原子返回可选图片并进入`submitting`，结束路径统一`clear`。

Desktop根据可选图片选择现有Hub路径：有图片调用附件提交，无图片调用`agent.input`且`source=selection`。无图片路径只要求会话声明`user_input`，不检查`user_input_attachment`。确认卡隐藏空图片并显示“本次不包含截图”；accepted关闭，rejected/unknown清除问题且不自动重试。

原生验证使用固定非敏感问题与受控本地Agent，只记录帧数、附件帧数、source、结果和清理布尔值。验证直接点击与权限缺失均不发送附件，任务/事件/Outbox不变化，应用数据不出现正文。
