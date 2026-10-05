# CU-S4 应用级 Computer Use Driver 收敛

Story: CU-S4
Epic: CU
Status: verifying
OpenSpec: cu-s4-trycua-foreground-delivery-spike、cu-s4-foreground-text-apply、cu-s4-ax-visual-fallback、cu-s4-sky-product-driver

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

2026-09-30正式Yonder企业微信样本证明：固定`@trycua/cua-driver@0.25.0`可以启动并Observe精确窗口，但企业微信后台快捷键、像素点击和引用文本输入均不能取得确认后置事实；把`unverifiable`聚焦当作焦点凭据会把后续敏感文本暴露给错误输入框，已禁止。

本Story仅授权限时macOS升级Spike，对比0.25.0与固定候选0.30.4。Spike通过和AD-CU-07接受前，不修改正式依赖、不改变Gateway协议、不恢复不可核实文本续跑。Windows继续按用户决定暂缓。

## OpenSpec 与验证

[OpenSpec Change](../../../../openspec/changes/cu-s4-trycua-foreground-delivery-spike/)只产生隔离证据和ADR结论。期限为2026-10-02；届时必须接受、缩小或淘汰候选路线。

2026-09-30 macOS独立Goal PASS：0.25.0与0.30.4统一样本均通过，升级无新增收益，AD-CU-07接受“保留0.25.0并接入既有受限前台坐标文本能力”。产品Apply与正式Yonder企业微信复验仍待独立Change；Windows暂缓，因此Story不Archive。

同日正式 QQ 音乐样本补充证明：应用启动成功后，空/不稳定 AX 树与后台动作拒绝没有产出截图，TM-S9 又丢弃 `UnknownObserved` 视觉证据，使慢脑无法按“元素失败才视觉”继续。用户明确要求修复该差距。Accepted AD-CU-08 与 `cu-s4-ax-visual-fallback` 在不引入第二执行栈的前提下补齐同窗口视觉降级；正式 QQ 音乐正向复验前 Story 继续保持 verifying。

同日追加产品要求：通用桌面任务必须由慢脑一次提交计划片段，并在顶部浮窗显示计划总数和当前附近步骤，不能退化为“慢脑已提交单步执行”。协议1.40已增加封闭通用桌面动作语义；待正式macOS产品复验后更新Verification Goal。

同日正式QQ音乐补充样本确认0.25.0的快捷键输入没有动作级foreground能力，而0.30.4具备该独占字段；AD-E0-02与AD-CU-07已修订为固定唯一0.30.4，不保留双版本执行栈。正式包回归通过前Story继续verifying。

同日进一步同实例对照确认：trycua 0.30.4仍无法返回QQ音乐应用内元素，而Codex Computer Use / `@oai/sky`可读取完整应用级AX transcript。主人决定不再扩展trycua，Accepted AD-CU-09与`cu-s4-sky-product-driver`把macOS产品切换为Sky单栈；历史trycua Change保留证据但不再决定产品依赖。

2026-10-01产品接线验证返回Apply：Sky-only包和隔离Worker已通过，但正式Yonder不属于ChatGPT/Codex可信宿主，native-pipe握手被服务关闭；六次Gateway样本均在首步安全交回且已取消。

2026-10-04在固定Sky发行物内确认官方签名`SkyComputerUseClient mcp`及其父进程Team ID要求。同发行物OpenAI签名Node派生Client后，`list_apps/get_app_state`隔离探针已取得QQ音乐41个AX元素和截图；当前改为正式桥接实施，Story继续verifying，仍不得在Gateway与独立Goal通过前Archive。

同日正式Gateway复验时图形会话已锁屏；官方Codex CUA与独立原生窗口夹具均同步返回`cgWindowNotFound`，Yonder正确分类为`target-window-unavailable`并交回，验证任务已取消。该外部环境阻塞不授权跳过Goal；解锁后重跑正式正向和视觉交回样本。

2026-10-06 macOS独立Verification Goal PASS：正式Yonder完成QQ音乐关键词输入、受限候选下移、结果页进入、视觉双击播放与`task.complete`终结；播放器显示`One Last Kiss - 宇多田光`并处于播放态，最终任务`task_343827d8b795ca1bf50528181e6b9cc0`为`completed@23`。验证同时修复协议语义`ARROWDOWN`到Sky xdotool键名`Down`的Adapter映射，以及计划片段任务被全局内存Runtime误判为不存在的终结路由。Windows对等验证仍按主人决定暂缓，Story保持verifying且不Archive。
