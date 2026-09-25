# CI 架构检查

关联 Story：OCT-S1；OpenSpec：`openspec/changes/oct-s1-task-status`。本增量不改变架构决策。

在仓库根目录执行：

```bash
python3 -m unittest discover -s scripts -p 'test_*.py' -v
python3 scripts/check_architecture.py
cargo test --workspace --locked
cargo run -p yonder-protocol --example generate --locked -- --check
```

发布冻结清单在提交后生成，默认打印 JSON；产物签名与公证由后续增量承接：

```bash
# <commit> 必须是当前干净源码树的40位HEAD；两个release二进制都要使用同一值构建。
YONDER_BUILD_COMMIT=<commit> cargo build --release --locked -p yonder-desktop -p yonder-cli
python3 apps/desktop/package-macos-preview.py --release --allow-adhoc --channel dev
python3 scripts/release.py --artifacts --channel dev
```

macOS 预览/发布共用 `apps/desktop/package-macos-preview.py`；`--release` 指向 release 产物并拒绝开发profile、旧提交、desktop/CLI身份不一致或有已跟踪改动的源码树，`--identity` 指定正式签名身份，`--allow-adhoc` 仅用于本机临时签名验证。打包生成的`build-provenance.json`声明两个构建身份；发布冻结会重新执行最终包内身份入口、校验签名、记录最终二进制哈希并与当前HEAD交叉核对。

`verify_macos_install_lifecycle.py`接收哈希不同的上一版与当前候选`.app`，只在内部临时根目录验证新装、原子升级、候选落位失败回退和卸载保留合成用户数据。它不读写真实`/Applications`或HOME，不启动GUI，也不代替正式签名、公证和Gatekeeper验证。

脚本从 `cargo metadata --no-deps --locked --offline` 读取正式 Workspace 清单，覆盖依赖别名、开发/构建依赖及平台条件；隔离 Spike 不属于正式 Workspace。Domain 不得增加外部依赖。新增模块名称必须经架构核对后登记。

PR 事件额外传 `--event "$GITHUB_EVENT_PATH"`，从 JSON 读取正文，不将正文作为 Shell 执行。PR 模板三行分别给出 Story ID、Change 名称和仓库相对验证记录路径，检查唯一性、文件存在、Story/Change 双向引用及验证记录引用 Story。此检查只验证关联，不判断证据真实性、规格是否覆盖全部代码或 Goal 是否通过。

工作流使用 runner 现有 Rust/Python，打印版本供追溯，Cargo.lock 锁定产品依赖。Windows/macOS 各自运行库层测试和生成检查，不安装桌面应用、不操作系统权限。后续应在实际原生功能交付时补齐前端 import、Adapter 内部互调和原生证据门禁；本次不宣称完成这些检查。仓库分支保护还需将工作流设为必需检查，本地配置不会自动启用远端策略。

实现依据：[Cargo metadata 官方字段说明](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)、[GitHub Actions 工作流语法](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)。
# 规划目录门禁补充

`check_architecture.py` 每次检查 `docs/specs/epic-*/story-*/` 的唯一标识、模块归属、三份设计和必需章节；存在 Change 时检查双向关联。PR 仅允许 ready/implementing/verifying/done 的 Story，旧平铺文件不作为实施入口。设计审阅质量与平台证据仍须独立审阅，结构检查不能替代。
