# CU-S4 Sky 产品链路 macOS 独立 Verification Goal

Result: PASS

日期：2026-10-06  
平台：macOS 26.0.1，正式调试包 `Yonda.app`  
结论：**PASS（仅 macOS；Windows 暂缓）**

## 验证边界

- 只通过安装包内 `yonder mcp`、Agent Gateway、计划片段与正式 Sky Worker 执行任务；没有直连 SQLite、Local Socket、Driver 或 Office/命令旁路。
- 固定 Driver 为外部签名 Sky `0.7.1`，包检查确认无 trycua、Qwen 与第二执行栈。
- 目标为 QQ 音乐搜索并播放 `One Last Kiss - 宇多田光`；每次动作后由 Yonder Observe，unknown 不自动重放。

## 正向证据

1. 任务 `task_eca281933c5094f0024617df0aa54eed` 在同一正式Gateway链路完成：
   - `launch → focus → input` 后于 `sequence=16` 携带截图交回；
   - 新鲜读取 `task.get/task.events` 后提交 `rebind → ARROWDOWN → ENTER → ENTER`，候选导航和结果页进入均为Observed；
   - `ENTER` 不能证明播放时于 `sequence=36` 交回，不误报成功；
   - 根据结果页截图提交 `rebind → click(x=670,y=780,click_count=2)`，于 `sequence=47` 返回 `fragment-complete`；后置截图显示底部播放器曲目为 `One Last Kiss - 宇多田光`、时长 `04:12` 且中央为暂停图标。
2. 上述样本暴露全局内存Runtime存在时，未登记到该Runtime的计划片段任务无法 `task.complete`。修复只在 `RuntimeError::NotFound` 时回到计划片段既有的Observed/步骤边界/桌面租约三重门禁；冲突、背压和不可用继续失败关闭。
3. 重建正式包后，任务 `task_343827d8b795ca1bf50528181e6b9cc0` 经正式Gateway执行视觉播放片段，后置截图再次显示准确曲目处于播放态；随后 `task_complete(expected_sequence=22)` 返回 `completed@23`。

有界结构化摘要保存在 [result.json](../../../apps/desktop/evidence/cu-s4-sky-product-20261006/result.json)。任务终结会按产品策略清理临时Observation截图；本Goal不把临时截图复制为长期正文或日志。

## 回归结果

- `python3 apps/desktop/check-sky-cua-worker.py`：PASS；覆盖已安装应用解析、Worker进程重建后的外置绑定恢复、`click_count=2`、`ARROWDOWN→Down`与敏感参数剥离。
- `cargo test --offline --locked -p yonder-protocol`：17 PASS。
- `cargo test --offline --locked -p yonder-application`：58 PASS。
- `cargo test --offline --locked -p yonder-adapters`：77 PASS；覆盖Adapter绑定与Worker进程生命周期分离、同任务激活及任务会话清理。
- `cargo test --offline --locked -p yonder-desktop`：lib 25 PASS、main 11 PASS；发布契约负例已对齐协议1.41。
- `python3 apps/desktop/check-cua-package.py`：PASS；Sky `0.7.1`、签名Node与Client Team `2DC432GLL2`，trycua/Qwen缺失符合单栈边界。

## 2026-10-06窗口绑定外置复验

- 从提交`a0f57c9`构建正式调试`Yonda.app`，仅通过安装包内`yonder mcp`与生产UDS创建任务`task_dff3884691d4291f23a66f47dfe35d40`。
- 同一三槽位片段依次完成应用启动、窗口内文本输入和结果激活；三个attempt均为`action_succeeded=true`、`observe_valid=true`，没有`target-window-unavailable`，最终`task_complete`返回`completed@18`。
- 该样本证明成功启动形成的bundle绑定由Rust Adapter跨步骤提供；后续动作不依赖Node Worker模块全局`launchedTarget`。独立Worker回归额外在启动后销毁并重建进程，仍以外置绑定继续下一动作。
- 输入正文、AX transcript、截图、PID和窗口号未写入本Goal、结构化证据、任务事件或日志。

## 2026-10-06企业微信企业入口负样本

- 从提交`713e38c`构建的正式调试`Yonda.app`经安装包内`yonder mcp`执行任务`task_080739ed85eebc668da4b97ffcf6bc96`，三槽位返回`fragment-complete@17`并终结为`completed@18`。
- 随后读取正式Yonder保存的动作后Observation，证明企业切换菜单并未打开；此前坐标还错误指向底部图标。以正确顶部头像坐标执行的任务`task_2b366150d400bcd80c2b95ea54d7ef88`同样未打开菜单，却仍被截图指纹变化误判为完成。
- 因此上述任务均为负样本，不计入PASS。协议1.42必须让坐标超时无条件交回，并以动作前新鲜AX元素索引完成正确入口复验后，才能关闭该增量任务。
- 协议1.42正式包随后以任务`task_91481a211402807f2d038ccee16b18f6`验证新鲜元素索引：顶部图像动作没有可验证transcript变化，Yonder正确以`handback@16`携带截图交回，未误报完成。截图物理坐标兜底仍只产生光晕/悬停；最终失败关闭任务`task_1e18ae977a3560f236a1d9fc9de6da8e`返回`handback@16`并取消为`cancelled@17`，证明最新包不再凭截图像素差异推进。
- 2026-10-07对照发现Codex CUA对同一主窗口坐标点击也返回`noWindowsAvailable`，但当前AX transcript的元素18可稳定打开账号/企业入口面板。这证明正确循环必须坚持AX元素优先，只在元素不可用时降级到同窗口视觉坐标。
- 重建正式`Yonda.app`后，Gateway任务`task_83ab460064455b4f0fc0bb49b5cfc2bb`先以坐标候选两次安全交回，再由慢脑读取`task.get + task.events`提交`observed_element_index=18`的新片段。该次事件为`action_succeeded=true`、`observe_valid=true`，Observation由260个主窗口元素变为68个面板元素，最终`fragment-complete@22 → completed@23`。
- 企业入口激活回归通过；本轮没有选择“狼顾科技”、没有搜索会话、没有填写或发送消息，不把入口PASS扩大为完整多企业切换流程PASS。
- 全仓`check_architecture.py`另被基线`TM-S9/README.md`缺少标准`Story/Epic/Status/OpenSpec`字段阻断；该既有问题与本负样本分别记录，均不能伪装为本项通过。

## 未计入 PASS 的负样本

- 干净重跑任务 `task_25cff1f3e9e5c6ba85236125f17851bd` 已证明重新输入和候选导航，但最后视觉动作发生`target-window-unavailable`且无后置截图，于`cancelled@42`停止；未重放unknown动作，也未计入正向结论。
- Windows原生E2E按主人决定暂缓；本结论不得外推到Windows，Change与Story不Archive。
