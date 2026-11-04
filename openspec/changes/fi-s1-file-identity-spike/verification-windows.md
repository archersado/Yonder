# 独立 Verification Goal：Windows 文件身份与锁

日期：2026-10-23
结论：FAIL（统一样本未全部通过）

- 卷序列号与文件索引能够识别硬链接为同一文件。
- 同文件第二把排他锁被拒绝；授权根逃逸被拒绝；同目录临时文件提交通过。
- Microsoft Word 真实打开 DOCX 后，排他写锁被拒绝。
- 符号链接创建被 Windows 拒绝，错误为 `Administrator privilege required for this operation.`；因此符号链接身份断言未执行，统一样本总结果为失败。
- 结构化证据：[`result.json`](../../../spikes/file-identity/evidence/windows-20261023/result.json)。

不接受 AD-FI-01，不创建依赖该路线的产品文件写入 Proposal。后续应在启用开发者模式或具备创建符号链接权限的 Windows 验证机复测。
