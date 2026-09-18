# AG-S5 Agent连接态 macOS Verification Goal

日期：2026-09-17  
平台：macOS arm64  
状态：通过

## 目标

独立验证桌宠连接提示由Agent会话事件驱动，且不覆盖生命周期动画。

## 证据

- `cargo test -p yonder-desktop agent_input::tests::routes_to_the_registered_agent_connection -- --exact`通过：两个会话中断开一个仍保持连接，最后一个断开后变为未连接。
- `node --check apps/desktop/ui/pet.js`通过。
- 本机预览应用建立`agent-bridge`会话后，独立桌宠窗口截图中右下徽标中心像素为`rgba(105,183,117,255)`。
- 终止桥接后1秒内，同一窗口徽标中心像素变为`rgba(147,152,160,255)`；未查询任务数据库。
- 重新建立桥接后恢复已连接态；桌宠透明背景、呼吸与眨眼素材保持可见。

Windows产品验证按用户决定暂缓。
