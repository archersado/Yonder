# CX-S1 视觉交互设计

## 入口与流程

本Spike无产品UI。命令行先显示能力预检，再由验证者显式启动有界样本；不得主动弹出Accessibility授权提示或自动修改浏览器配置。Chrome扩展仍由验证者在开发者模式显式加载并点击开始/停止。

## 状态与错误反馈

结构化结果至少区分`ready`、`observing`、`stopped`、`capability_unavailable`、`protocol_rejected`和`browser_unavailable`。输出只含布尔值、计数和稳定错误码，不含应用/窗口标题、URL、页面标题、键值、坐标或完整Payload。

Native Host连接成功只表示协议通道可用，不表示产品Recording已开始。扩展停止后必须断开Host；Host退出与Observer释放分别可观察。权限拒绝时给出系统设置方向但不代替用户点击授权。

## 无障碍与平台验证

扩展按钮保持可访问名称并支持键盘命令；`REC`与`ERR`徽标不能作为唯一反馈，验证页面须有文字状态。原生交互保存结构化日志；若需截图，只截取扩展状态，不包含网页内容、地址栏或其他用户数据。

## 待决事项

Google Chrome可在当前macOS环境验证；Microsoft Edge未安装，保持明确未验证。产品权限页、采集开关、排除列表和配额反馈不在本Spike实现。
