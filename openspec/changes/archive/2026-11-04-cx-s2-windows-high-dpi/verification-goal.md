# 独立 Verification Goal：CX-S2 Windows 200% DPI圈选截图

Story：CX-S2
结论：PASS（仅限 Windows 96/192 DPI Preview 扩围）

## 已有实现证据

- 独立C++ WGC Spike：真实192 DPI主屏240×160逻辑窗口连续三次返回480×320物理像素，四色、DisplayConfig、释放和不落盘PASS。
- 产品共享Rust模块：192 DPI主屏480×320连续三次四色PASS；96 DPI负坐标副屏240×160连续三次回归PASS；结构化结果见`spikes/region-capture/evidence/windows/result.json`。
- Rust测试：Application 21/21、Desktop lib 9/9、Desktop bin 16/16 PASS；包含96/192 DPI门禁、异轴/超限拒绝、200%坐标单次换算和旧代次隔离。
- `cargo build --locked -p yonder-desktop`、前端Node测试、架构关联检查及`openspec validate cx-s2-windows-high-dpi --strict`均PASS。

## 独立复核矩阵

| 验收 | 证据 | 结论 |
|---|---|---|
| 192 DPI共享模块三连尺寸/四色/释放/不落盘 | `spikes/region-capture/evidence/windows/result.json` | PASS |
| 96 DPI负坐标回归 | 同上 | PASS |
| 未验证DPI和异轴DPI拒绝 | `region_capture_windows`单元测试 | PASS |
| 200% CSS矩形只换算一次 | `main.rs`单元测试 | PASS |
| 真实圈选后确认卡显示图片，重选/取消可用 | 2026-11-04用户在当前200% Windows主屏独立确认 | PASS |
| 无协议/持久化/状态所有者变化 | 架构检查与diff | PASS |

## 保留门禁

本Goal不得宣称125%/150%、跨屏、应用切换、来源应用、受保护内容、权限撤销、真实显示变化/拔屏、完整鼠标自动化或完整CX-S2通过。
