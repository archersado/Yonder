# OOXML Adapter 对照结果

## 固定依赖

| 候选 | ZIP | XML | 许可证 |
|---|---|---|---|
| Rust 进程内 | `zip 6.0.0`，仅启用 deflate | `quick-xml 0.42.0` | MIT |
| Node Worker | `fflate 0.8.3` | `@xmldom/xmldom 0.9.12` | MIT |

## 合成样本结果

两候选均通过 DOCX 段落、XLSX inline string 单元格、PPTX 文本的单点替换；均验证原文件不变、错误 `expected_hash` 返回 `document_conflict`、未修改 part 解压后 SHA-256 不变、输出 ZIP 可重开。

| 指标（三文件单进程） | Rust | Node |
|---|---:|---:|
| Harness 内耗时 | 1 ms | 46 ms |
| 进程墙钟 | <10 ms | 130 ms |
| 峰值 RSS | 3,220 KiB | 73,128 KiB |
| 最小交付体积 | 964 KiB Release binary | 约 1.3 MiB node_modules，另需 Node Runtime |

## 当前结论

选择 Rust 进程内 OOXML Adapter，淘汰 Node Worker。Rust 在合成和 Microsoft 官方 Transitional 复杂样本上均保持未修改 part 的解压后字节一致，并显著降低延迟、内存及运行时分发成本。

## Microsoft 官方复杂样本

固定使用 Open XML SDK 仓库中的图表 DOCX、超链接/图形 XLSX 和含媒体 PPTX，下载脚本校验 SHA-256。两候选均完成目标文本替换、未修改 part 保真和输出重开：Rust 15ms，Node 197ms。

首次 `O14ISOStrict` 样本因 WPS 多实例/格式兼容问题作废，不纳入结论。替换为 Microsoft 官方 Transitional 复杂样本后，Rust 用时 51ms，Node 用时 1647ms；Rust 输出的 DOCX、XLSX、PPTX 均通过 WPS 直接打开与用户目视确认，无损坏或修复提示。
