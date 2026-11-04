# DS-S4 Windows 原生资源包拒绝验证

日期：2026-11-01
平台：Windows 11
结论：**PASS**（PACK-01、PACK-03、PACK-05 的 Windows 原生拒绝路径）

## 验证范围

- 从 Yonda 原生托盘第 4 项进入“导入桌宠资源包”。
- 使用 Windows 系统文件选择器选择受控的伪 ZIP `invalid-pet-pack.zip`。
- 验证宿主将结果分类为 `invalid`，桌宠显示“资源包不符合规范”。
- 比较 `%APPDATA%\com.yonder.desktop\pet-packs` 导入前后的完整文件树摘要，确认失败不会激活或覆盖资源包。
- 本次未执行 CUA。

## 结果

原生文件选择器成功打开并关闭，结构化宿主日志仅记录结果分类 `桌宠资源包导入：result=invalid`，不记录输入路径或包内容。截图显示桌宠窗口中的“资源包不符合规范”反馈。导入前后资源树摘要均为 `absent`，因此 `active_pack_unchanged=true`。

Windows 调试构建 `cargo build --offline --locked -q -p yonder-desktop` 通过。OpenSSL 的 `LNK4099` 是静态库缺少调试 PDB 的链接器警告，不影响可执行文件或本项运行结果。

## 证据

- `apps/desktop/evidence/ds-s4-mascot-pack-import-windows-20261101/native-invalid-import-result.json`
- `apps/desktop/evidence/ds-s4-mascot-pack-import-windows-20261101/native-invalid-feedback.png`
- `apps/desktop/evidence/ds-s4-mascot-pack-import-windows-20261101/verify-native-invalid-import.ps1`
- `apps/desktop/evidence/ds-s4-mascot-pack-import-windows-20261101/invalid-pet-pack.zip`

首次夹具曾错误依赖透明 WebView 的根级 UI Automation `role=status` 节点，尽管系统文件选择器已完成，Windows 没有在该根句柄暴露状态文本，导致自动化超时。修复后以真实的 300×300 进程可见窗口截图、最小化结果日志和资源树摘要三方交叉验证，不再把 22×22 宿主句柄当成桌宠窗口。

## 完成边界

本报告补齐资源包导入子范围的 Windows 原生拒绝证据。Hatch Pet 式生成委托仍按用户决定只保留设计，不在本 Change 实施；DS-S4 因完整生成链路未进入实现，继续保持 `verifying`，不 Archive。
