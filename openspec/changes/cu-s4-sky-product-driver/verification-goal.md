# Verification Goal：Sky单一产品CUA Driver

- 状态：PASS（macOS；Windows暂缓）
- 日期：2026-10-12（汇总；原始失败证据保留）
- 平台：macOS；Windows按主人决定暂缓

## 2026-10-18 Sky 0.7.4 固定发行物复核

- 独立worktree重新构建正式`Yonda.app`，包审计结果为`sky-external-0.7.4`、Jev SDK `0.6.0`，签名Node、签名Client、App Group、Automation用途声明、Worker一致性以及trycua/Qwen缺失检查全部通过。证据见[结构化结果](../../../apps/desktop/evidence/cu-s4-sky-package-20261018/result.json)。
- 正式产品任务`task_5f4967b174b1c3b9587f68cd5d4703af`经安装包MCP/Gateway进入QQ音乐：启动与搜索框聚焦均形成有效Observe；后续`ENTER`在序号41形成`observed(action_succeeded=true, observe_valid=true)`，证明0.7.4签名Client的动作与Observe链路可用。
- 同次动作随后由EX-S2步骤核验判定当前目标未完成并在序号43交回。该结果是正确的负向业务判定，不是CU-S4 transport/版本失败；未把Driver成功回执提升为任务完成。
- 回归结果：Adapter 88/88、Application 92/92、Desktop 39/39；`scripts/check_architecture.py`、OpenSpec strict与`git diff --check`通过。

独立复核正式包不含trycua且没有Driver开关/回退；固定Sky身份校验、应用绑定、transcript元素计数、语义动作映射、动作后Observe、unknown交回和清理测试全部通过。正式Yonder Gateway 已完成QQ音乐多步骤正向样本和后续回归；不保存transcript、正文或完整Payload。早期FAIL/Apply章节作为技术路线演进证据保留，不再覆盖2026-10-06及之后的macOS PASS结论。Windows暂缓，因此Story仍不Archive。

## 2026-10-12 当前 dev 复核

- 当前产品只打包 Sky Worker 与固定签名桥，未恢复 trycua、Qwen、Driver 开关或第二执行栈。
- 正式任务 `task_78ac5b1e03f22dfec3356b8aafeae3b9` 经最新 `Yonda.app → MCP stdio → Gateway → Application → Sky Worker` 取得 QQ 音乐应用级 AX 句柄；目标按钮未变化时返回 `unknown/driver-action-unconfirmed@11 → handback@12`，任务在序号13取消，没有把无关 transcript 变化误报成功。
- 当前回归证据复用 CU-S6 对 Sky 临时句柄与目标确认的增量验证：`apps/desktop/evidence/cu-s6-qqmusic-button-target-confirmation-20261012/`。该证据进一步证明 CU-S4 的 Sky 单栈仍是实际产品链路，但不扩大为 Windows 结论。

## 2026-10-01结果

- PASS：Sky-only正式包审计、固定`@oai/sky@0.7.1`身份、隔离应用绑定、AX transcript元素计数及`click/set_value/press_key`语义映射；桌面36项、Adapter75项和发布审计7项测试通过。
- FAIL：正式Yonder Gateway连续创建六个QQ音乐五槽位片段，均在首个`launch_app`的`list_apps`前返回`unknown/worker-failed`并安全交回；有界诊断最终收敛为`transport-closed`，未执行后续桌面动作。
- 归因：已安装Computer Use服务与SDK内服务版本、SHA-256一致，Socket存在；同一应用通过Codex CUA REPL可取得AX transcript，但Yonder直接native-pipe握手被关闭。Sky受ChatGPT/Codex可信宿主/授权边界保护，现有公开SDK路径不能作为Yonder独立产品Driver直接接线。
- 处置：所有验证任务均经Gateway取消；不Archive、不合入dev。下一步必须取得OpenAI支持的外部Broker/授权接口，或另立架构决定选择可独立分发的Driver；不得伪造可信RPC、复制签名能力或回退双栈。

证据：[结构化结果](../../../apps/desktop/evidence/cu-s4-sky-product-macos-20261001/result.json)。

## 2026-10-04签名MCP桥复核

- PASS：正式包固定`@oai/sky@0.7.1`，OpenAI签名Node与Client的Team ID、Client identifier、App Group均通过；trycua/Qwen回退不存在。
- PASS：隔离Worker覆盖签名MCP握手、应用清单文本解析、调用范围内elicitation、范围外拒绝、应用绑定、`click/set_value/press_key/type_text`及`target-window-unavailable`分类；Adapter 75项、Desktop 36项通过。
- PASS：在会话进入锁屏前，签名Node派生Client的独立QQ音乐探针取得41个AX元素及JPEG截图，证明不再受原`transport-closed`阻断。
- BLOCKED：正式Gateway五槽位QQ音乐任务在`launch_app`交回；同刻官方Codex CUA与独立原生窗口夹具均返回`cgWindowNotFound`。系统CG窗口清单只含`loginwindow`、`SecurityAgent`等锁屏窗口，故归因为当前macOS锁屏，而非Yonder、目标App或签名transport。
- 处置：正式任务已经Gateway取消；Worker把该状态收敛为`target-window-unavailable`且未派发后续动作。独立Goal仍为FAIL/返回Apply，不Archive、不合入dev；主人解锁图形会话后重跑正式Gateway正向与视觉交回样本。

证据：[2026-10-04结构化结果](../../../apps/desktop/evidence/cu-s4-sky-product-macos-20261004/result.json)。
