# macOS 独立验证

日期：2026-09-17  
结论：PASS（仅临时目录合成样本）

## 验证命令

```bash
/Users/archersado/.cargo/bin/rustc --edition 2024 spikes/file-identity/macos-probe.rs -o /private/tmp/yonda-file-identity-macos
/private/tmp/yonda-file-identity-macos
```

## 结果

- 软链接和硬链接归并为相同 `st_dev + st_ino` 文件身份。
- 同一文件经硬链接再次申请写锁被拒绝。
- 相对路径及经符号链接越出授权根的输出路径被拒绝。
- 临时文件在目标目录写入、同步并原子替换成功。
- WPS真实打开DOCX时持有目标文件引用且`File::try_lock`被拒绝；关闭该标签后引用消失且锁可取得。
- 结构化结果见 `spikes/file-identity/evidence/macos-20260917/result.json`。

WPS对照结果见`spikes/file-identity/evidence/macos-20260917/wps-lock.json`。

Windows统一样本未验证；AD-FI-01保持Proposed，不接产品Gateway。
