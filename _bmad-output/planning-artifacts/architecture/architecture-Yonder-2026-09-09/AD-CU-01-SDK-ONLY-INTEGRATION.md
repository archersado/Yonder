# AD-CU-01 CUA仅SDK集成

状态：Accepted（集成边界）；日期：2026-09-14。关联CU-S1/CU-S2、TM-S3、RC-S1及e0-compare-cua-drivers。Architecture Impact：architecture-change（Driver交付和进程宿主）。

## 来源与决定

原产品简报“两条执行路径”、架构CUA模型无关Driver/受监管按需Worker；用户本次明确“不希望再包一个app，只想使用他们的sdk”。采用已选定@trycua/cua-driver@0.25.0 SDK及匹配原生库，不附带、安装或启动上游CuaDriver.app，不以SDK createPrivateWorker接口要求的上游可执行文件作为产品依赖。

沿用Yonder按需Worker边界：Yonder Supervisor直接启动自己管理的Node Worker，由Worker导入SDK并创建同进程CuaDriver运行时。Worker无独立产品窗口、托盘或服务；Yonder是唯一用户入口和权限责任宿主。通信仅继承私有stdio，不开放HTTP/TCP或可重连Daemon。生产Node打包、宿主责任链、系统权限及两平台原生输入仍须Spike证明，不据本决定宣称完成。

## 状态和停止

TM仍拥有当前任务状态/attempt与唯一前台租约，Supervisor拥有Worker进程生命周期，CU Adapter翻译SDK结果，UI不成为状态所有者。SDK shutdown关闭准入并等待排空，不是强制中止；Supervisor终止Worker并确认退出只能证明进程停止，已产生副作用仍须unknown/新鲜Observe，不自动重试或立即释放未知租约。Worker退出不等于用户输入来源/隐私过滤已验证，不自动启用Recording。

## 证据与迁移

现有macOS SDK只读生命周期/已提交Abort验证继续有效。独立上游构件STOP-PKG分支停止：裸二进制哈希/签名通过但Gatekeeper拒绝的原始证据保留；完整App包虽已下载，不再解包、评估或运行。该外部App拒绝不作为SDK方案失败或通过依据，也不绕过Yonder最终签名/公证要求。无产品协议、数据库或实际任务数据迁移。

下一样本为Yonder监督的SDK子进程：只读就绪、宿主停止并确认子进程退出、新SDK实例只读恢复、宿主断开时子进程退出。结果仅布尔/分类，不采集输入、截图、剪贴板内容或Recording。真实原生动作停止、动作后Observe、宿主崩溃残留与权限归属分别保留门禁。

补充原生证据：STOP-SDK01–04只读生命周期已通过，结构化结果为spikes/cua-driver-comparison/evidence/stop-macos-sdk-worker-20260914/result.json，独立Goal为openspec/changes/e0-compare-cua-drivers/verification-sdk-worker-macos.md。测试Node宿主监管SDK子进程的停止和断连退出得到确认；不宣称正式Yonder宿主权限或真实输入停止完成。

首批原生输入证据：STOP-IN01–03独立Goal通过，SDK隔离原生输入后Observe与目标匹配、动作间shutdown拒绝新输入且750ms目标状态稳定，见openspec/changes/e0-compare-cua-drivers/verification-input-macos.md。原生执行中动作中断、SDK交付来源/隐私过滤、正式Yonder宿主权限归属和Windows证据仍待验证；不据此启用接管或Recording。
