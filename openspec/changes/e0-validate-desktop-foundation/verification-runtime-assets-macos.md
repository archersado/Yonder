# DS-S1 运行时素材独立 Verification Goal

Story：DS-S1；OpenSpec：e0-validate-desktop-foundation；日期：2026-09-13。

目标：保留原图，用系统工具生成较小运行时副本，检查形象和既有动画，并复测持续展开资源占用。

状态：本轮 macOS 素材优化验证通过；不代表 DS-S1 整体通过，不 Archive。

## 实施与检查

依据 AD-E0-01「2026-09-13 运行时素材尺寸验证」及用户明确授权。四张原图未修改，副本在 `spikes/desktop-foundation/ui/runtime/`，生成命令见该目录 README。主体和闭眼遮罩由1254×1254缩为600×600，探头由1774×887缩为336×168，支持现有逻辑尺寸的3倍缩放。仅 index.html 的五个图片引用切换到副本，没有调整动画代码、频率或遮罩。

四张源文件共5,682,476字节，副本共997,044字节。尺寸、PNG颜色类型和 SHA-256 留存在 `spikes/desktop-foundation/evidence/runtime-assets-20260913.json`。主体/睁眼探头保持 RGBA，闭眼图保持原有 RGB，通过既有 CSS 遮罩使用；不把原本无 Alpha 的闭眼图误报为透明图。

Release 离线锁定构建成功，已打包并启动 PID 15225。二进制 SHA-256：`d7b4d19302a3668420e051860e00693687dd425998938856414f7db4ab1d0ba1`。

ego-browser TaskSpace 25 验证五张图片均解码到副本尺寸；checkMotion 返回 moving/wagging/stopped 全为 true，恢复可见后6秒内捕获自然 blinking 状态。任务页面已关闭。该检查使用模拟 IPC，仅证明页面行为，不替代原生验证。

原生独立探针取得400×400像素透明窗口截图，人工查看小龙完整；采样中的截图保存在 `spikes/desktop-foundation/evidence/app-resized-budget-20260913-external.frames/native.png`。本轮没有重新覆盖点击、全部躲藏方位及拖动原生交互。

## 五分钟资源对照

命令：

```sh
python3 spikes/desktop-foundation/measure-app-macos.py 15225 /Users/archersado/workspace/Yonder/spikes/desktop-foundation/evidence/app-resized-budget-20260913-external.json --awake-probe /private/tmp/yonda-awake-probe --external-frames
```

采样器自检通过。61个样本、300.012秒；同一 resource coalition 的宿主与三个 WebKit 进程启动身份稳定。同一窗口每次为200×200且应用未隐藏，约120/240秒托盘找回保持展开；未修改三分钟闲置规则。

| 指标 | 原图持续展开基线 | 小尺寸副本 |
|---|---|---|
| CPU（单核） | 0.1747% | 0.1550% |
| 四进程 footprint 求和均值 | 163.99 MiB | 117.22 MiB |
| 同时点求和峰值 | 182.72 MiB | 122.85 MiB |

均值下降约28.5%，峰值下降约32.8%；本轮读数低于1% CPU /150 MB目标（峰值约128.81 MB）。这是分进程 footprint 求和，不是系统去重内存，也未覆盖 WindowServer 开销。

前两次采样在首张窗口截图处失败，未取得样本，分别保留 `app-resized-budget-20260913.json` 和 `app-resized-budget-20260913-retry.json`，不计性能结果。独立执行原生探针截图成功，采样器内调用报 could not create image from window，原因未定；本轮新增 --external-frames 将截图分开，不绕过窗口状态检查。

对照沿用相同进程归属、资源统计、持续展开与托盘找回口径；截图次数/调用方式以及进程重启后的缓存状态不同，因此不声称严格单变量因果证明。保留历史失败基线，不将其改为通过。本轮尚无 Windows、长时间运行、多屏或任务负载证据，AD-E0-01仍待评审，DS-S1继续 design-review。
