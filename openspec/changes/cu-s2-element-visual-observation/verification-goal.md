# CU-S2 元素优先与窗口视觉证据 Verification Goal

状态：核心与隔离Worker检查PASS；正式macOS Yonder Runtime双样本待完成，Windows暂缓，不Archive。

## 已完成检查

- `cargo test -p yonder-protocol -p yonder-application`：67项PASS。
- `cargo check -p yonder-desktop`：PASS。
- `python3 apps/desktop/check-cua-visual-fallback.py`：证明歧义元素不执行副作用、只采集同一后台可信窗口截图，输出`element_first=true`、`visual_fallback=true`、`side_effect_executed=false`。

## 待完成原生证据

- 正式Yonder GUI与安装包内MCP：唯一元素路径不产生视觉定位截图。
- 同一窗口制造元素歧义：原动作未执行，协议1.34交回临时窗口Observation。
- 任务终结后临时截图清理，SQLite/事件/Outbox不含路径或图像数据。
