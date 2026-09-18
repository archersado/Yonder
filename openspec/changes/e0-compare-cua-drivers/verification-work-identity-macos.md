# 独立Verification Goal：当前工作引用身份复核

状态：PASS（macOS隔离工作身份子范围）；关联CU-S2 EXEC-ID01/FOCUS-05、TM-S3、AD-CU-03、Proposed AD-TM-08。技术Spike，非产品接管实施。

依据spikes/cua-driver-comparison/WORK-IDENTITY-PLAN.md，首次执行日期2026-09-14，期限2个工作日。只操作隔离夹具，不录制、不输入、不截图用户工作，不改正式任务或系统权限。

验收：原目标正常/最小化可定位且frame保持；关闭、同名同frame重建、进程退出/重启的旧引用拒绝；同名重叠映射不唯一拒绝；权限拒绝与同PID/不同启动时间以明确合约负样本验证，不能冒充真实权限切换或PID复用。必须保留原AX对象并检查当前AX列表成员身份、WindowServer唯一映射和系统启动时间；只按标题/几何重查不算通过。

证据为结构化逐样本结果和原生夹具key/可见状态，SDK无截图Observe沿用选定0.25.0 SDK。验证失败返回Spike实施，保留首次失败；Windows、正式Yonder权限、多Space/显示器、步骤停止和Recording不在本Goal通过范围。

## 实测结果

fresh-mapping/result.json全部七组样本PASS：原对象正常定位、最小化恢复与frame保持；同名窗口完全重叠时mapping_not_unique且诱饵保持key；关闭/同名同frame重建后retained_object_missing、无定位副作用，重新SDK Observe的新引用有效；进程退出重启后process_identity_changed，旧引用拒绝，新进程Observe及引用有效。系统proc_pidinfo取得启动秒/微秒，不能以PID单独作为进程身份。

changed-start与deny-permission只是使用同一校验函数的合约负样本，未制造真实PID复用或关闭系统权限；不外推正式宿主权限通过。测试未录制、未输入、未请求截图、未修改任务，夹具与探针自动终止。Windows和多Space/显示器仍暂缓/待验。

失败证据first、diagnostic、readiness、observed均保留。WindowServer先可见但AX未就绪、初次CG几何过早及最小化动画期间数据不一致导致拒绝；修复为新引用前有效SDK Observe、只读匹配获取新鲜CG几何、最小化后有界只读复核，不重发定位、不放宽身份。最终通过不抹去失败记录。

可运行：swiftc编译input-fixture-macos.swift及focus-target-macos.swift；node work-identity-macos-probe.mjs <fixture> <helper> <全新证据目录>。编译及Node语法检查通过。AD-TM-08字段/Port/事务仍Proposed，完整CU-S2/TM-S3不Done或Archive。
