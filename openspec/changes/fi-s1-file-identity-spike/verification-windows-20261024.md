# 独立 Verification Goal：Windows 文件身份与锁复测

日期：2026-10-24
结论：PASS

- 统一真实 DOCX 样本通过卷序列号与文件索引识别硬链接为同一文件。
- 当前普通用户令牌无创建文件符号链接权限；探针记录原始错误后，按 AD-FI-01 允许的 Windows 等价重解析路径改用目录 Junction，别名路径仍解析为同一文件身份。
- 同文件第二把排他锁被拒绝；授权根逃逸被拒绝；同目录临时文件原子提交通过。
- Microsoft Word 真实打开 DOCX 后，排他写锁被拒绝。
- 结构化证据：[`result.json`](../../../spikes/file-identity/evidence/windows-20261024/result.json)。

本次复测解除 2026-10-23 Windows 符号链接权限造成的技术阻塞；旧失败证据保留用于追溯。
