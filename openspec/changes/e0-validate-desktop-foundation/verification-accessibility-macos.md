# DS-S1 无障碍语义激活独立 Verification Goal

Story：DS-S1；OpenSpec：e0-validate-desktop-foundation；日期：2026-09-13。

依据：Story无障碍按钮要求及增量规格「辅助功能按钮激活」。状态：页面合约及本机原生AXPress唤醒通过；Windows/实体键盘等独立门禁保留。

## 失败证据与根因

旧PID92356原生辅助功能检查：trusted=true、app_active=false，WebArea内存在AXButton，名称为“点击趴在边缘的 Yonda 唤醒它”，有AXPress动作。直接执行该动作返回0，等待1秒后窗口仍112×56，未恢复。用户此前手动鼠标唤醒确认保持有效，两者是不同输入路径。

页面只有pointerdown/pointerup与keydown处理，缺少无指针的语义click。新增check-accessibility.mjs先在旧实现运行，断言失败：语义click后awake=false，后续Enter才调用wake。没有将AX动作返回0误作功能通过。

## 最小修复

pet.js新增detail=0的click入口，并与Enter/Space共用activate分流。普通鼠标click(detail>0)继续忽略，保留pointer事件原有点击/拖动处理；不重复唤醒。未增加命令、协议、任务状态、线程、动画计时或外部依赖。

ego-browser TaskSpace31同一页面回归结果：awake=true、wakes=1、noDuplicate=true、keyboardFeedback=true。测试加速闲置等待且模拟IPC，只用于页面合约，不能替代原生结果。任务已finish关闭，未升级浏览器。

## 原生复验

Release离线锁定构建成功，通过正常.app入口启动PID9322。二进制SHA-256：`fa06f9f3c8e6159c116438d1eb59e4b409f0c313097d05f725644807f40b5e3f`。

check-accessibility-macos.swift仅检查该bundle的目标窗口，等待真实自然休眠按钮，不缩短产品计时，不发送全局按键，也不预先激活应用。然后执行一次AXPress，最多3秒查找清醒名称及200至201原生窗口尺寸，失败返回非0。

命令：`/private/tmp/yonda-check-accessibility 9322 /Users/archersado/workspace/Yonder/spikes/desktop-foundation/evidence/accessibility-wake-20260913.jsonl`。

Swift编译及原生运行退出0。实际记录：休眠前置AXButton名称匹配、app_active=false；AXPress返回0；约107.845ms后出现“轻点 Yonda 小龙，按住移动可拖动”名称，窗口27386为200×201。日志中的100秒等待是观察器启动后的等待，不作为180秒闲置计时证据。原生JSONL保留在上述路径。

出现标签及几何恢复证明语义激活进入唤醒流程；标签在waking阶段即可更新，所以108ms不是完整出现动画播放完毕时间。无同期截图，按结构化日志证明此输入路径，不扩展为画面保真验收。

## 边界

实体Enter/空格、快速重复激活、减少动态效果仍有各自原生证据缺口。Windows用户暂缓；本次未重跑不受影响的五分钟资源基线，旧基线仍注明其二进制版本。原生合成坐标点击失败不在本次修复范围，也不据此否定用户手测鼠标唤醒。DS-S1总Goal未通过，不Archive。
