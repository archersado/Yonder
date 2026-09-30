# Verification Goal：Sky单一产品CUA Driver

- 状态：待执行
- 日期：2026-09-30
- 平台：macOS；Windows按主人决定暂缓

独立复核正式包不含trycua且没有Driver开关/回退；固定Sky身份校验、应用绑定、transcript元素计数、语义动作映射、动作后Observe、unknown交回和清理测试全部通过。通过正式Yonder Gateway提交QQ音乐多步骤片段，确认至少能从应用级AX定位搜索框并推进输入/搜索；不保存transcript、正文或完整Payload。失败则返回Apply，不Archive。
