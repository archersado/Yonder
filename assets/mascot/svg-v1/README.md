# Yonda 高保真 SVG 状态包 v1

本目录由现有透明 PNG 生命周期素材在本机进行 128 色分层矢量描摹，不上传素材，也不替换产品运行时素材。

- `idle.svg` 等九个文件：保留原图比例、体积光影和细节的独立 SVG 状态。
- `yonda-vector.states.js`：可直接拖入 `svg-character-animator` 的中文版 Animator Studio。
- `trace.py`：复用环境现有 OpenCV 与 NumPy，重新生成全部输出并执行最小检查。

运行：`python3 assets/mascot/svg-v1/trace.py`

还原度优先：每态由大量彩色路径组成，状态切换采用整组矢量淡入淡出；不把彼此没有语义对应关系的自动描摹路径强行 morph。
