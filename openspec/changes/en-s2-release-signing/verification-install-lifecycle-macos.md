# EN-S2 macOS 安装生命周期独立 Verification Goal

日期：2026-09-25  
结论：PASS（macOS隔离安装生命周期子范围）

## 目标

使用两个不同的真实临时签名`.app`，在与真实系统和用户数据隔离的临时根目录验证新装、升级、候选落位失败回退及卸载保留用户数据。验证不得操作真实`/Applications`、HOME、任务库或附件。

## 结果

- 上一版包与当前提交候选包均通过`codesign --verify --deep --strict`，应用树哈希不同，构成真实升级样本。
- 新装先复制到暂存路径并校验签名，再原子落位；安装后应用树哈希与候选一致。
- 升级先完整复制并验证候选，再把旧应用原子移动为回退副本；成功后候选与回退副本哈希分别匹配新旧包。
- 分别在旧应用已移动且候选尚未落位、候选已落位但尚未完成复核的位置注入失败；两条路径都自动恢复旧应用且应用树哈希不变。
- 合成SQLite主库、WAL/SHM sidecar与附件在升级、失败回退和卸载前后树哈希保持一致；结构化证据不记录其正文。
- 卸载只把应用移动至隔离Trash，安装位置清空，用户数据继续存在。
- 33个脚本测试、OpenSpec严格校验与架构检查通过；完整Workspace Rust回归沿用同一`dev`基线的107项PASS。

结构化证据：`apps/desktop/evidence/en-s2-install-lifecycle-macos-20260925/result.json`。运行时完整哈希结果输出到隔离验证清单，不写入任务库。

## 验证命令

```text
python3 scripts/verify_macos_install_lifecycle.py \
  --previous-app <previous.app> \
  --candidate-app <candidate.app> \
  --expected-commit <当前40位HEAD> \
  --channel dev \
  --output <result.json>
python3 -m unittest discover -s scripts -p 'test_*.py' -v
openspec validate --all
python3 scripts/check_architecture.py
```

## 范围限制

本Goal验证`.app`目录安装语义与数据保留，不引入安装器或自动更新服务，也不启动GUI、不申请系统权限。包仍为临时签名，因此不代替Developer ID、notarytool、Gatekeeper或Windows验证；完整EN-S2保持`implementing`，不得Archive。
