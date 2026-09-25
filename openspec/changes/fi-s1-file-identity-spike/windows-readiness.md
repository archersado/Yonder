# FI-S1 Windows运行包就绪记录

日期：2026-09-24
状态：运行包已准备；Windows实机未验证，不构成Verification Goal PASS

## 已准备内容

- `spikes/file-identity/windows-probe.rs`：使用Windows文件句柄的volume serial与file index归并原路径、硬链接和软链接；验证第二写锁拒绝、规范化父目录、目录软链接逃逸拒绝及同目录提交。
- `spikes/file-identity/windows-office-lock-probe.rs`：在隔离DOCX由Office或WPS打开/关闭时验证写打开或写锁是否按预期被拒绝。
- `spikes/file-identity/run-windows.ps1`与`run-windows-office-lock.ps1`：编译固定源码、校验单行JSON并以UTF-8无BOM写入指定证据路径，结束后删除临时可执行文件。

所有结构化结果只包含平台、固定宿主分类、布尔断言和错误类别，不包含文件路径或正文。软链接夹具需要Windows Developer Mode或创建符号链接权限；该条件只用于构造Spike样本，不是产品运行权限。

## 本机预检查

- 两个Rust文件已通过`rustfmt --edition 2024`。
- 两个文件在macOS使用非Windows入口编译成功，运行时均以退出码2明确拒绝生成Windows证据。
- 当前工具链只安装`aarch64-apple-darwin` target，且没有PowerShell；因此未对`cfg(windows)`分支做Windows类型检查，也未运行PowerShell脚本。

## Windows实机门禁

Windows环境须使用Rust 1.89或更高版本依次执行：

```powershell
powershell -ExecutionPolicy Bypass -File .\spikes\file-identity\run-windows.ps1
powershell -ExecutionPolicy Bypass -File .\spikes\file-identity\run-windows-office-lock.ps1 -DocumentPath C:\absolute\fixture.docx -Expected open -HostApplication wps
powershell -ExecutionPolicy Bypass -File .\spikes\file-identity\run-windows-office-lock.ps1 -DocumentPath C:\absolute\fixture.docx -Expected closed -HostApplication wps
```

还须复核结果来自同一隔离DOCX、打开与关闭顺序真实发生，并确认未记录路径或正文。上述证据未取得前，AD-FI-01保持Proposed，FI-S1与DO-S2不得开放产品文件写入口。
