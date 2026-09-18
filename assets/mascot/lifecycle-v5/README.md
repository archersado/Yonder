# 待命双爪修订 v5

用户反馈idle只见一只手；原始v4待命图本身缺少清晰可见的第二前爪，非尾巴剪裁。内置imagegen只修订待命四帧为两个短前爪清晰露于腹部前方，与双脚分开；保留奶白浅蓝角色、相机、透明处理及第三帧双眼闭合。v4原图保留，另外八状态不重新生成。

沿用用户授权Python/Pillow及v4纯绿色底透明处理，交付400×400 RGBA，runtime只替换idle四帧。源码图片保留用于追溯，不把纯绿色source当作运行时图片。通过双短爪、透明边缘与眨眼原生目测后再更新同一桌宠预览；完整Story门禁保持。

复验：构建及架构检查通过，单实例预览已重启。macOS原生窗口连续60张截图中双短爪清晰且与双脚分开，透明边缘未见格子；第16→17→18帧捕捉闭眼再睁开。证据：apps/desktop/evidence/state-assets-v4-20260914/idle-arms-v5-retry/open-closed-open.jpg、native-result.json、blink-check.json。仅待命素材修复通过，完整Story及Windows验证状态保持。
