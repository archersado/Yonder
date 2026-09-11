# E0-S4 OOXML Adapter 对照验证

## Story

作为 Yonder 开发团队，我们需要使用同一组 DOCX/XLSX/PPTX 样本，对照 Rust 进程内实现与按需 Node Worker，实现唯一的 OOXML Adapter 技术选型。

## 验收条件

1. 两候选只通过 Document Port 语义执行读取和局部修改，不向上暴露 XML。
2. 原文件保持不变；输出先写临时文件，结构校验通过后再原子提交。
3. 写入前验证 `expected_hash`，冲突必须 fail closed。
4. 未修改的 OOXML part 保持逐字节一致；关系、内容类型及 ZIP 结构有效。
5. DOCX 段落、XLSX 单元格、PPTX 文本各完成读取、替换、重新打开验证。
6. 记录冷启动、批量耗时、峰值内存、分发体积、依赖和许可证。
7. 只输出 Rust、Node 或两者均淘汰，不保留长期双实现。

OpenSpec：`openspec/changes/e0-compare-ooxml-adapters/`
