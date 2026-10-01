# Verification Goal：Sky单一产品CUA Driver

- 状态：FAIL（返回Apply）
- 日期：2026-09-30
- 平台：macOS；Windows按主人决定暂缓

独立复核正式包不含trycua且没有Driver开关/回退；固定Sky身份校验、应用绑定、transcript元素计数、语义动作映射、动作后Observe、unknown交回和清理测试全部通过。通过正式Yonder Gateway提交QQ音乐多步骤片段，确认至少能从应用级AX定位搜索框并推进输入/搜索；不保存transcript、正文或完整Payload。失败则返回Apply，不Archive。

## 2026-10-01结果

- PASS：Sky-only正式包审计、固定`@oai/sky@0.7.1`身份、隔离应用绑定、AX transcript元素计数及`click/set_value/press_key`语义映射；桌面36项、Adapter75项和发布审计7项测试通过。
- FAIL：正式Yonder Gateway连续创建六个QQ音乐五槽位片段，均在首个`launch_app`的`list_apps`前返回`unknown/worker-failed`并安全交回；有界诊断最终收敛为`transport-closed`，未执行后续桌面动作。
- 归因：已安装Computer Use服务与SDK内服务版本、SHA-256一致，Socket存在；同一应用通过Codex CUA REPL可取得AX transcript，但Yonder直接native-pipe握手被关闭。Sky受ChatGPT/Codex可信宿主/授权边界保护，现有公开SDK路径不能作为Yonder独立产品Driver直接接线。
- 处置：所有验证任务均经Gateway取消；不Archive、不合入dev。下一步必须取得OpenAI支持的外部Broker/授权接口，或另立架构决定选择可独立分发的Driver；不得伪造可信RPC、复制签名能力或回退双栈。

证据：[结构化结果](../../../apps/desktop/evidence/cu-s4-sky-product-macos-20261001/result.json)。
