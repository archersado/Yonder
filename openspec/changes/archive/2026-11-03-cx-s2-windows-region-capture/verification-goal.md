# 独立 Verification Goal：CX-S2 Windows 受限圈选截图

Story：CX-S2
日期：2026-11-03
结论：PASS（仅限批准的Windows 100% DPI受限Preview范围）

## 复核范围

- 受限范围仅为96 DPI（100%缩放）单显示器选区；非100%缩放必须进入无图确认。
- 产品实现与Spike共用同一WGC模块；像素只在内存中转换和编码，不保存截图。
- 完整CX-S2、高DPI、Windows应用切换、显示器变化、来源应用和物理断开不在本结论内。

## 实现者提供的验证输入

- `powershell -File spikes/region-capture/run-windows.ps1`：96 DPI负坐标副屏连续三次尺寸与四色校验PASS；192 DPI主屏由共享模块直接拒绝且不返回像素。
- `cargo test --offline --locked -p yonder-application`：20项全部通过。
- `cargo test --offline --locked -p yonder-desktop --bin yonder-desktop`：坐标、代次、状态、WGC像素/PNG与前端接线等10项全部通过。
- `node --test apps/desktop/ui/region-preview-feedback.test.mjs`：错误文案、后端无图转换与DOM无图确认投影通过。
- `cargo build --offline --locked -p yonder-desktop`：PASS。
- `node --check apps/desktop/ui/region-preview.js`：PASS。
- `python3 scripts/check_architecture.py`：PASS。

## 独立复核结论

2026-11-03由非实现者针对当前工作树逐项复核，三项均PASS：

- 正式产品路径执行viewport→work area→物理矩形换算，保留负坐标；换算单测与真实96 DPI负坐标副屏三连样本共同覆盖。
- 真实192 DPI显示器确实调用产品同一`capture_region`并返回`capture-unsupported-scale`，没有返回像素。
- 捕获失败进入无图`Reviewing`，已有图片被清除，DOM投影为“仅提问”，并可继续走无附件文字提交。

复核同时确认WGC事件、session、frame pool和帧由RAII/显式关闭释放，结构化证据没有截图且探针进程无残留。

## 结论边界与剩余工作

- 本结论不代表完整CX-S2 Done；高DPI扩围、Windows应用切换、显示器变化、来源应用和物理断开仍须后续门禁。
- 当前证据采用“产品接线 + 状态/换算测试 + 共享捕获模块真实双屏样本”的分层方式，没有完整Tauri鼠标自动化。
- 运行证据绑定了源码修订与工作树状态，尚未记录源码或探针二进制内容哈希。
- 密码框、隐私窗口、系统安全界面与用户排除应用需专用无敏感样本补充验证，已登记延期工作。
