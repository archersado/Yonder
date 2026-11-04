# 任务

- [x] 完成CX-S2三份设计与需求来源映射
- [x] 建立Proposed AD-CX-01与Spike OpenSpec
- [x] 实现macOS无敏感检查窗口与原生区域捕获探针
- [x] 验证macOS已授权单显示器的坐标/像素换算、色块和进程清理
- [x] 验证macOS全新临时应用身份的未授权预检失败关闭，不请求权限或截图
- [x] 验证macOS非激活临时选择层的合成原生Esc与超时清场，不安装全局监听
- [x] 验证macOS局部选择层的CGHID合成原生拖动、选区尺寸、鼠标恢复和不抢前台
- [x] 验证macOS主显示器模式变化后确认卡清场并恢复分辨率
- [ ] 验证macOS副屏/负坐标、运行中撤权与物理断开清场
- [ ] 实现并验证Windows完整同一样本
  - [x] Windows Graphics Capture 单显示器四色/DPI/DisplayConfig核心样本连续三次通过（`verification-windows-20261101.md`）
  - [x] Windows 96 DPI负坐标副屏连续三次PASS；192 DPI主屏按批准边界安全降级
  - [x] Windows 192 DPI（200%）主屏WGC核心样本连续三次通过（`verification-windows-20261104.md`）
  - [ ] Windows 120/144/168 DPI、物理选择/Esc、权限拒绝与运行中撤销
- [ ] 完成全量双平台统一样本后接受或拒绝完整ADR（当前只接受各自有证据的受限子范围）
- [x] ADR接受后生成Windows受限产品实施Proposal
