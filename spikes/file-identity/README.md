# FI-S1 文件身份 Spike

```bash
/Users/archersado/.cargo/bin/rustc --edition 2024 macos-probe.rs -o /private/tmp/yonda-file-identity-macos
/private/tmp/yonda-file-identity-macos
```

macOS临时目录与WPS真实锁样本已通过，证据见`evidence/macos-20260917/`。Windows探针只用于后续实机取证；在Windows结果文件生成并独立复核前，AD-FI-01仍保持Proposed。

WPS真实占用探针：

```bash
/Users/archersado/.cargo/bin/rustc --edition 2024 wps-lock-probe.rs -o /private/tmp/yonda-wps-lock-probe
/private/tmp/yonda-wps-lock-probe /absolute/path/to/document.docx open
/private/tmp/yonda-wps-lock-probe /absolute/path/to/document.docx closed
```

Windows统一样本（需要Rust 1.89+；软链接夹具需要启用Developer Mode或以具备创建符号链接权限的终端运行）：

```powershell
powershell -ExecutionPolicy Bypass -File .\run-windows.ps1
```

Windows Office/WPS真实锁对照必须使用隔离DOCX，打开和关闭各执行一次：

```powershell
powershell -ExecutionPolicy Bypass -File .\run-windows-office-lock.ps1 -DocumentPath C:\absolute\fixture.docx -Expected open
powershell -ExecutionPolicy Bypass -File .\run-windows-office-lock.ps1 -DocumentPath C:\absolute\fixture.docx -Expected closed
```

脚本输出只包含结构化布尔值和错误分类，不记录文件路径或正文。macOS或非Windows编译只能检查解析与非平台入口，不能替代Windows实机证据。
