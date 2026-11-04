# Proposal：VI-S1 单轮语音输入 Spike

关联Story：VI-S1；关联AD：Proposed AD-VI-01。

比较Windows/macOS单轮语音输入候选路线，验证权限、采集、转写、停止和资源释放。debug预览复用Yonda进程与既有桌宠状态投影，提供小龙入口和同Space贴边卡片；不修改协议、SQLite或Gateway，不把独立测试App作为入口，不采集真实用户语音作为证据。

2026-11-02 Windows未打包候选虽能打开麦克风，却没有识别结果且`StopAsync`超时。按微软当前接口门禁增加有期限的稀疏MSIX身份对照：同一控制台探针、同一公开样本和同一可执行路径，仅增加包身份，不新增第二套产品UI、协议或长期执行栈。身份包构建与清单匹配可自动完成；创建/信任开发证书和`Add-AppxPackage -ExternalLocation`注册属于安装状态变更，必须取得用户单独确认。
