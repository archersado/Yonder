# 延期工作

- source_plan: `_bmad-output/plan-cx-s2-windows-region-capture.md`
  summary: 验证Windows受保护内容、密码框、隐私窗口和用户排除应用的失败关闭证据
  evidence: WGC会屏蔽系统保护像素，但当前统一样本未覆盖普通密码控件、隐私窗口和用户排除应用；需以不含敏感内容的原生专用样本确认没有敏感像素可达。
- source_plan: `_bmad-output/plan-cx-s2-windows-region-capture.md`
  summary: 增加完整Tauri鼠标圈选自动化与证据内容哈希
  evidence: 当前由产品接线/换算单测、共享WGC真实双屏探针和前端行为测试组成分层证据；独立复核认为批准范围充分，但浏览器级鼠标流程以及源码/探针二进制哈希仍可增强证据绑定。
