# EN-S2 macOS 发布包内容审计独立 Verification Goal

日期：2026-09-25
结论：PASS（发布包内容与通道审计子范围）

## 目标

验证发布冻结不会只记录产物哈希，还会在输出可发布结论前检查macOS包的装配边界、敏感文件、自有文本敏感值和发布通道。审计失败只报告路径或错误分类，不回显检测到的正文或凭据值。

## 结果

- 发布冻结测试的协议与SQLite期望已同步到当前正式契约`1.25`/`19`，消除旧`1.20`/`18`导致的门禁失败。
- 允许路径限定为desktop、CLI、固定CUA/Jev Worker及其锁定依赖、三份发布元数据、`Info.plist`和系统签名目录；包内符号链接与其他路径失败关闭。
- 数据库、日志、环境文件、私钥、证书容器和常见凭据文件名被拒绝；Yonder自有JSON、plist和Worker文本检查私钥/AWS/OpenAI样式的真实令牌形态。
- 单元负样本分别注入`tasks.db`和私钥正文，审计均拒绝，错误信息没有回显用户数据或私钥正文。
- 使用`package-macos-preview.py --release --allow-adhoc --channel dev`装配的真实包包含191个文件，内容审计通过并记录`channel=dev`、`sensitive_values_recorded=false`。
- 对同一`dev`包请求`--channel stable`时，冻结脚本退出1并报告通道不一致，未生成错误的stable清单。

结构化证据：`apps/desktop/evidence/en-s2-bundle-content-macos-20260925/result.json`。

## 验证命令

```text
python3 scripts/test_release.py
python3 scripts/release.py --root <repo> --artifacts --channel dev --output <manifest>
python3 scripts/release.py --root <repo> --artifacts --channel stable --output <manifest>
openspec validate en-s2-release-signing --strict
python3 scripts/check_architecture.py
```

## 范围限制

真实包使用本机现存的release二进制完成装配审计，没有声称这些二进制由当前提交重新构建；签名仍是临时签名。本Goal不覆盖Developer ID正式签名、notarytool公证、Gatekeeper、安装/升级/回退/卸载或Windows。完整EN-S2继续保持`implementing`，不得Archive。
