# Verification Goal：Agent Computer Gateway

状态：PASS（macOS，2026-09-16）。Windows按用户决定暂缓。

验证范围：协议门禁、通用SDK动作桥接、身份隔离、当前桌面焦点窗口解析、固定trycua SDK真实动作与动作后Observe、真实硬件输入中断、显式完成及桌面资源释放。

证据：

- `apps/desktop/evidence/computer-sdk-bridge-20260916/result.json`：协议1.11经通用`tool_name + arguments`调用SDK `type_text`；隔离原生窗口收到固定文本，attempt为`observed`，任务完成且Recording未启动。
- `apps/desktop/evidence/computer-sdk-bridge-press-key-20260916/result.json`：同一通用桥接调用第二个SDK工具`press_key`并完成，证明Rust/Gateway未维护动作枚举或第二份参数Schema。
- `apps/desktop/evidence/computer-gateway-user-input-20260916/result.json`：2秒原生动作期间真实移动鼠标，Worker终止，attempt为`unknown/user-input`，任务转为`interrupted`，最终sequence 5，Recording未启动且未自动重试。
- `apps/desktop/evidence/computer-continuous-session-final-retry-20260916/result.json`：正式UDS任务连续执行`type_text`与`press_key`，两步均Observe成功，Node/trycua Worker PID保持不变，任务完成后Worker退出；证明正常动作间未重建Driver或切换执行器，且没有长期常驻。首次最终验证被真实用户输入中断并保守记为unknown，Observe后以新任务复验，未重试原动作。
- `apps/desktop/evidence/computer-step-observe-20260916/result.json`：协议1.12通过单次`computer.step`完成声明、真实`type_text`、动作后Observe和步骤推进；连续两步复用同一Worker，Observe返回67个有界UI元素，完成后Worker退出。当前系统未向trycua返回截图，因此未使用外部截图工具，截图字段保持为空；清理由单元测试覆盖。
- `cargo test --workspace --locked`共45项通过，覆盖连续Worker会话、SDK参数信封与受保护身份递归拒绝、Gateway、SQLite attempt结果事务、桌面单租约、完成释放与用户输入中断合约；生成协议检查和架构围栏检查通过。

实现修正：Worker运行时读取SDK `listToolsJson()`并按其Schema注入Yonder持有的目标字段，再原样调用`callTool`；最前方目标优先使用AX焦点应用/窗口，再映射WindowServer ID，避免跨Space全局窗口顺序误选。CGEventTap只计来源PID为0的真实硬件事件，忽略SDK合成输入。预览包采用稳定`com.yonder.desktop`指定要求签名，使辅助功能授权在重编译后保持有效。

连续动作修正：Node Worker与trycua Driver改为宿主内惰性创建并跨正常动作复用；每一步仍单独执行前后Observe。用户输入、超时、非法响应或Worker异常会终止会话，不自动重试。新增会话夹具与真实产品双动作证据。

协议1.12修正：Agent默认只调用`computer.step`，Gateway内部复用步骤声明、执行、Observe与推进用例；底层`computer.execute`保留线协议兼容但不再由MCP发现。Observe只返回元素数量和可选的SDK截图路径，截图限制为4 MiB并置于Yonder私有目录，任务完成或异常时清理。
