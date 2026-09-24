# EN-S2 发布冻结 macOS 验证

验证时间：2026-09-23

## 范围

本记录只验证 macOS 本机 release 构建与版本冻结清单，不宣称签名、公证、升级备份、安装包或 Windows 完成。

## 命令与结果

- `python3 -m unittest discover -s scripts -p 'test_*.py' -v`：21 项通过，含 Workspace 版本不一致负例与 `.app` 递归哈希内容敏感性负例。
- `/Users/archersado/.cargo/bin/cargo test -p yonder-desktop release_contract --locked --offline`：4 项通过，覆盖版本、协议与 SQLite schema 不一致时的启动拒绝。
- `/Users/archersado/.cargo/bin/cargo build --release --locked --offline`：通过。
- `python3 apps/desktop/package-macos-preview.py --release --allow-adhoc --channel dev`：生成 `target/release/Yonda.app`，写入 `Resources/channel.json`，并由脚本自动执行 `codesign --verify --deep --strict`。
- `python3 scripts/release.py --artifacts --output target/release/yonder-release-manifest.json`：生成清单并通过 JSON 解析，包含 `Yonda.app` 的递归文件哈希。
- `/Users/archersado/.cargo/bin/cargo run -p yonder-protocol --example generate --locked --offline -- --check`：通过。
- `/Users/archersado/.cargo/bin/cargo test -p yonder-adapters name_migration_backs_up_preserves_legacy_and_rolls_back_failed_ddl --locked --offline`：1 项通过，覆盖升级前 SQLite 快照备份、旧数据保留和迁移失败 DDL 回滚。
- `openspec validate en-s2-release-signing --strict`：通过。
- `PATH=/Users/archersado/.cargo/bin:$PATH python3 scripts/check_architecture.py`：通过。
- `python3 apps/desktop/package-macos-preview.py --help`：确认 `--channel` 与 `--notary-profile` 参数存在。
- `/usr/bin/codesign --verify --deep --strict --verbose=2 target/release/Yonda.app`：通过。
- `/usr/sbin/spctl --assess --type execute --verbose=4 target/release/Yonda.app`：返回 `rejected`，证明当前为临时签名，尚未通过公证。

## 冻结结果

- 版本：`0.1.0`
- 提交：`a4441513fda70ed49c79dac8dbf0956b042248f9`
- 协议基线：`1.20`
- SQLite schema：`18`
- Driver：`trycua`（CUA）、`ego-lite`（BUA），均为 macOS。
- `target/release/yonder-desktop` SHA-256：`7dbeb77d9e73b9209108fb653113db53f6130b084da08327aebc5b3995d868d5`
- `target/release/yonder` SHA-256：`6cc71829409a0d9b8cc4bf29d2bc35968ba9f3e68568e57ec8f62ba7755da917`
- `target/release/Yonda.app` recursive-files-sha256：`b9a6e0b449c1a3615c5d2000748061216f6c4472f21d1716b977dd3b5bbf3bec`

## 未完成边界

已执行临时 `codesign`、本地签名校验和 dev 通道打包；未执行正式身份签名、`notarytool` 公证、安装/升级/回退样本、真实用户库备份验证和 Windows 验证。EN-S2 保持 `implementing`。
