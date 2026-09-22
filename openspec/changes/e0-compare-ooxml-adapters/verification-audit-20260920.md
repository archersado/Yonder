# DO-S1 独立 Verification Goal 复核（2026-09-20）

## 结论

**未通过，不得 Archive。**

自动化证据支持 Rust 进程内唯一选型，Node Worker 淘汰结论未漂移；但现有 WPS 证据只有结论性文字，没有可独立审计的截图、视频或结构化日志，未满足 Verification Goal 的完整性要求。

## 复核范围

- Story：`docs/specs/epic-DO/story-DO-S1/`
- OpenSpec：`e0-compare-ooxml-adapters`
- Architecture Decision：Accepted `AD-E0-04`
- 环境：macOS，分支 `story/do-s1-verification`，基线提交 `7b73851`

## 命令与结果

在 `spikes/ooxml-adapter-comparison/` 执行：

```sh
python3 generate_fixtures.py
sh fetch_real_fixtures.sh
npm ci
/Users/archersado/.cargo/bin/cargo run --release
node node-harness.mjs
```

结果：

- Microsoft Transitional DOCX/XLSX/PPTX 下载哈希与脚本固定值一致：
  - DOCX `e88a8f272e3c28baae8aa82875acf8d29e26cb8746b9436a2adf9a05bfc63fbe`
  - XLSX `0307c8a9bbfe7e82fa2271be7629084a13df1051a366e24cb7318c2d2fcf7a9f`
  - PPTX `02a3df49dd36c319d4c34cd05591b4e6e055d365a9b4e79045a0cc9fd2441fb2`
- Rust 与 Node 均完成 3 个合成样本和 3 个官方复杂样本的唯一替换，均报告 `conflict:true`。
- 12 个输出包全部通过 `unzip -t`；Rust 复杂输出分别包含预期文本 `Yonder Integration Test`、`Yonder file name`、`Yonder Graphics Usage`。
- Harness 内本次耗时：Rust 59ms，Node 1296ms；性能方向未反转。
- 仓库产品依赖采用固定 `quick-xml 0.42.0`、`sha2 0.10.9`、`zip 6.0.0`；`fflate` 与 `@xmldom/xmldom` 只存在于 Spike，Node 未进入产品依赖树。
- `openspec status --change e0-compare-ooxml-adapters --json` 报告 artifacts 全部 done，`tasks.md` 原记录 8/8。

## 阻塞缺口

现有 `verification-goal.md`、`RESULTS.md` 与 `AD-E0-04` 仅重复“WPS 成功打开并经用户目视确认”的结论。Change 与 Spike 中没有对应截图、视频或结构化日志，也没有可追溯的验证日期、平台、WPS 版本、输入/输出产物哈希及无修复提示记录。仓库虽有 `verify-wps-windows.mjs`，但没有其运行结果。

因此无法独立确认 DOCX/XLSX/PPTX 三种 Rust 输出的 WPS 实机打开证据真实、完整。补齐证据后需重新执行本 Goal；通过前保持 Story 为 `verifying`，不归档 Change。

## 后续最小补证

在同一 WPS 环境打开本次 Rust 的三个复杂输出，保存一份结构化结果（平台、WPS 版本、三个输出 SHA-256、三个窗口/成功打开状态、无修复或损坏提示）并附截图或视频引用。补证不得修改产品代码。
