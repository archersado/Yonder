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

## 2026-10-06企业微信非AX企业入口复验

- 从提交`713e38c`构建并启动正式调试`Yonda.app`，仅通过安装包内`yonder mcp`、生产Agent Gateway与Sky单栈创建任务`task_080739ed85eebc668da4b97ffcf6bc96`。
- 同一三槽位片段连续完成“打开企业微信 → 将窗口置于前台 → 坐标激活左下角企业入口”；最后一步命中Sky Client不返回路径后，旧Client被停止，点击没有重放，新Client只执行一次同窗口Observe。
- 片段返回`fragment-complete@17`且无交回原因，验证任务随后经Gateway正常终结为`completed@18`。该结果证明无业务副作用坐标激活能够在宿主30秒上限前收敛，不再把任务卡死；发送、删除、支付等副作用不适用该恢复路径。
- `python3 apps/desktop/check-sky-cua-worker.py`补充两组隔离回归：界面指纹变化时确认成功，指纹不变时返回`suspected_noop`；两组均只投递一次坐标点击。
- 本轮`cargo test -p yonder-adapters`为77项PASS，`node --check`、Python语法检查、`git diff --check`与`openspec validate cu-s4-sky-product-driver --strict`通过。全仓`check_architecture.py`被基线`TM-S9/README.md`缺少标准`Story/Epic/Status/OpenSpec`字段阻断，不把该既有问题记为本修复通过。

## 未计入 PASS 的负样本

- 干净重跑任务 `task_25cff1f3e9e5c6ba85236125f17851bd` 已证明重新输入和候选导航，但最后视觉动作发生`target-window-unavailable`且无后置截图，于`cancelled@42`停止；未重放unknown动作，也未计入正向结论。
- Windows原生E2E按主人决定暂缓；本结论不得外推到Windows，Change与Story不Archive。
