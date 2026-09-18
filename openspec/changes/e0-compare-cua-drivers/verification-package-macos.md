# macOS独立构件核验 Verification Goal

当前状态：withdrawn（2026-09-14用户改为SDK-only，Accepted AD-CU-01）。裸构件Gatekeeper失败保留；完整App包下载完成但未解包/核验/运行，不继续此分支、不Archive或称PASS。下述verifying文字仅历史过程。

2026-09-14；状态verifying。实现后独立建立，关联CU-S1 STOP-PKG01–03、TAKEOVER-STOP-PLAN、AD-E0-02及e0-compare-cua-drivers增量规格。固定官方cua-driver-rs-v0.25.0 darwin-universal-binary构件；核对发布SHA-256、归档路径/类型/配额、Mach-O架构、codesign签名身份及Gatekeeper结果。任一拒绝不运行构件、不重签、不删除隔离属性。

实现为check-macos-package.py；--self-check检查合法相对文件与绝对/父级/符号链接拒绝。证据位于忽略目录evidence/stop-macos-package-20260914。下载不安装服务；Goal不覆盖真实输入停止、Worker退出、权限或Recording；Windows暂停，完整Story不Done/Archive。

首轮独立核验失败：裸二进制发布哈希、架构、Developer ID签名通过（Cua AI, Inc.，YCK386LBJ7），Gatekeeper退出3/source=Unnotarized Developer ID，未运行。按流程返回实施：静态审查同版本官方辅助安装脚本，确认目录包内App是macOS正式载体，修订探针支持固定哈希的目录包并核验完整App/主程序，原失败result.json保留。修订后再次独立验证App载体，结果待核实；通过也不追认裸二进制通过。
