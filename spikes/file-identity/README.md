# FI-S1 文件身份 Spike

```bash
/Users/archersado/.cargo/bin/rustc --edition 2024 macos-probe.rs -o /private/tmp/yonda-file-identity-macos
/private/tmp/yonda-file-identity-macos
```

macOS临时目录样本已通过，证据见`evidence/macos-20260917/result.json`。Office/WPS真实锁与Windows样本尚未执行。

WPS真实占用探针：

```bash
/Users/archersado/.cargo/bin/rustc --edition 2024 wps-lock-probe.rs -o /private/tmp/yonda-wps-lock-probe
/private/tmp/yonda-wps-lock-probe /absolute/path/to/document.docx open
/private/tmp/yonda-wps-lock-probe /absolute/path/to/document.docx closed
```
