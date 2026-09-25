# DO-S2 macOS OOXML 文件 Runtime 证据

本目录记录 Document Port 与受控 File Port 组合后的无正文结构化验证结果。

- DOCX、XLSX、PPTX 均从文件快照读取语义文本并默认另存，新输出可重新解析且源文件不变。
- 另存提交绑定源身份与 SHA-256；源变化、expected hash 冲突、目标存在或无效 OOXML 均不产生正式输出。
- 覆盖只接受可信 LocalUser，复用 FI-S1 独占锁、同目录暂存、结构校验和原子替换。
- 外部 advisory lock 被稳定区分为 `HostLocked`，不绕过、不自动重试。

本增量没有 UI，也不申请系统权限，因此不生成截图。全部样本为仓库合成文件并在唯一临时目录内运行；Windows 按用户决定延期，产品路径保持 unavailable。
