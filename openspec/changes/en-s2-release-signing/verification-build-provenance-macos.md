# EN-S2 macOS 构建来源绑定独立 Verification Goal

日期：2026-09-25  
结论：PASS（macOS release 构建来源子范围）

## 目标

验证发布冻结记录的提交来自实际交付的 desktop 与 CLI，而不是冻结脚本运行时为既有旧二进制补写当前 HEAD。两个二进制必须独立报告同一 release 构建身份；冻结阶段重新执行最终包内身份入口、校验签名、记录最终二进制 SHA-256 并与当前干净源码树 HEAD 比较。

## 结果

- 未提供 40 位 `YONDER_BUILD_COMMIT` 的 release 构建在 build script 阶段失败关闭；debug 构建只报告 `development`，不能进入 release 包。
- 当前提交分别构建 `yonder-desktop` 与 `yonder-cli`，两者结构化身份均为版本 `0.1.0`、profile `release`，提交与构建时 HEAD 完全一致。
- 临时签名 `.app` 生成 `build-provenance.json`；最终包内 desktop/CLI 身份与来源声明一致，`codesign --verify --deep --strict`通过。
- `release.py --artifacts --channel dev` 在干净源码树上通过，并确认冻结提交、包内构建提交一致，同时记录两份最终签名二进制哈希。
- 单元负样本覆盖旧提交、debug profile、缺失/无效提交、desktop/CLI提交不一致及包内二进制身份损坏，均在生成发布结论前拒绝；签名后修改由冻结阶段签名校验拒绝。
- 30个发布/架构脚本测试、107个 Workspace Rust 测试、74项OpenSpec校验与架构检查通过。

结构化证据：`apps/desktop/evidence/en-s2-build-provenance-macos-20260925/result.json`。

## 验证命令

```text
YONDER_BUILD_COMMIT=<当前40位HEAD> cargo build --release --locked -p yonder-desktop -p yonder-cli
target/release/yonder build-info
target/release/yonder-desktop --release-build-info
python3 apps/desktop/package-macos-preview.py --release --allow-adhoc --channel dev
python3 scripts/release.py --artifacts --channel dev --output <manifest>
python3 -m unittest discover -s scripts -p 'test_*.py' -v
cargo test --workspace --locked
openspec validate --all
python3 scripts/check_architecture.py
```

## 范围限制

本Goal使用临时签名，只证明当前提交的构建来源、包内二进制完整性与冻结绑定，不代替 Developer ID 正式签名、notarytool公证、Gatekeeper、安装/升级/回退/卸载或Windows验证。完整 EN-S2 继续保持 `implementing`，不得 Archive。
