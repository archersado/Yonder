# 运行时素材

原图保留在上一级目录，本目录仅存系统 `sips` 生成的小尺寸副本。200×200 主体及112×56探头按3倍显示尺寸准备；页面引用副本，动画及遮罩保持原样。

在仓库根目录重新生成：

```sh
sips -Z 600 spikes/desktop-foundation/ui/yonda-dragon-v1.png --out spikes/desktop-foundation/ui/runtime/yonda-dragon-v1.png
sips -Z 600 spikes/desktop-foundation/ui/yonda-eyes-closed-v1.png --out spikes/desktop-foundation/ui/runtime/yonda-eyes-closed-v1.png
sips -Z 336 spikes/desktop-foundation/ui/yonda-peek-v1.png --out spikes/desktop-foundation/ui/runtime/yonda-peek-v1.png
sips -Z 336 spikes/desktop-foundation/ui/yonda-peek-closed-v1.png --out spikes/desktop-foundation/ui/runtime/yonda-peek-closed-v1.png
```

主体、睁眼探头保留 Alpha；闭眼素材原本为 RGB，继续通过既有 CSS 遮罩使用。更大显示尺寸或超过3倍缩放时应从原图重新生成。原图仍随现有静态资源目录保留，优化针对运行时加载，不宣称安装包变小。
