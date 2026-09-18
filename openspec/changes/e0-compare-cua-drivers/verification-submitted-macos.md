# 已提交只读调用独立 Verification Goal

日期2026-09-14；状态PASS（仅已提交只读调用范围）。CU-S1 STOP-SUB01–04对应三份设计、TAKEOVER-STOP-PLAN和增量规格。实现后建立，验证固定trycua0.25.0真实macOS提交后Abort及并发shutdown，使用stop-macos-probe.mjs结构化结果和退出码；Windows暂缓。

检查：Abort竞争时序与真实返回被记录；取消后新读成功；并发shutdown和读调用完成后关闭态拒绝新调用；结果无正文且未触发键鼠、截图、Recording。任何子进程超时或语义断言失败返回实施阶段。该Goal不验证原生输入中断或Worker退出，不授权CU-S2/接管记录，不Archive完整Story。

## 原生证据与结论

命令：`node spikes/cua-driver-comparison/stop-macos-probe.mjs spikes/cua-driver-comparison/evidence/stop-macos-submitted-20260914/result.json`。退出0，readonly_lifecycle_passed=true，submitted_lifecycle_passed=true；此前生命周期语义回归同时通过。

| 验收 | 实际结果 | 判断 |
| --- | --- | --- |
| STOP-SUB01 | abort_before_settlement=true，submitted_read拒绝 | JS提交后的取消可观测；native_admission_observed=false，不能称原生动作中断 |
| STOP-SUB02 | read_after_abort成功且is_error=false | 取消后新调用可用 |
| STOP-SUB03 | 并发shutdown成功，shutdown_read拒绝，关闭后读拒绝 | 准入竞争结果真实；没有观测到已准入长动作排空 |
| STOP-SUB04 | input_dispatched/recording_started/native_input_stop_verified均false | 未输入/采集，停止门禁保留 |

SDK安装包README声明shutdown关闭准入并等待已准入操作结束；本次不能独立证明该等待覆盖执行中原生输入。独立Worker构造接口要求binaryPath，npm包不分发可执行文件。本机已有Spike/临时文件检索未找到可复用cua-driver可执行文件；不以同进程SDK或假Worker替代。下一真实验证必须先取得版本与签名可核实的macOS构件并定义隔离输入/动作后Observe样本；不重启Qwen双栈。
