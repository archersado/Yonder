"""把现有透明 PNG 状态本地描摹为多色 SVG；不上传素材。"""
from pathlib import Path
import json
import cv2
import numpy as np

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
SOURCES = {
    "idle": ("待命", "apps/desktop/ui/runtime/lifecycle-v5/idle-00.png"),
    "listening": ("收到请求", "apps/desktop/ui/runtime/lifecycle-v4/listening-03.png"),
    "thinking": ("等待外部响应", "assets/mascot/lifecycle-v4/frames/thinking-00.png"),
    "executing": ("执行中", "apps/desktop/ui/runtime/lifecycle-v4/executing-00.png"),
    "waiting_for_user": ("等待用户", "apps/desktop/ui/runtime/lifecycle-v10/waiting_for_user-03.png"),
    "paused": ("暂停", "apps/desktop/ui/runtime/lifecycle-v4/paused-03.png"),
    "recording": ("手动录制", "assets/mascot/lifecycle-v4/frames/recording-00.png"),
    "success": ("成功", "assets/mascot/lifecycle-v4/frames/success-02.png"),
    "failed": ("失败", "assets/mascot/lifecycle-v4/frames/failed-03.png"),
}


def trace(source: Path, colors=128):
    rgba = cv2.imread(str(source), cv2.IMREAD_UNCHANGED)
    assert rgba is not None and rgba.shape == (400, 400, 4), source
    rgba = cv2.cvtColor(rgba, cv2.COLOR_BGRA2RGBA)
    alpha = rgba[:, :, 3]
    ys, xs = np.where(alpha > 3)
    rgb = rgba[ys, xs, :3]
    lab = cv2.cvtColor(rgb.reshape(-1, 1, 3), cv2.COLOR_RGB2LAB).reshape(-1, 3).astype(np.float32)
    features = np.column_stack((lab, alpha[ys, xs].astype(np.float32) * .7))
    criteria = (cv2.TERM_CRITERIA_EPS + cv2.TERM_CRITERIA_MAX_ITER, 35, .35)
    cv2.setRNGSeed(7)
    _, labels, centers = cv2.kmeans(features, colors, None, criteria, 3, cv2.KMEANS_PP_CENTERS)
    labels = labels.ravel()
    canvas = np.full(alpha.shape, -1, np.int16)
    canvas[ys, xs] = labels
    layers = []
    for label in range(len(centers)):
        mask = np.uint8(canvas == label) * 255
        contours, _ = cv2.findContours(mask, cv2.RETR_LIST, cv2.CHAIN_APPROX_SIMPLE)
        paths = []
        for contour in contours:
            if cv2.contourArea(contour) < .4:
                continue
            contour = cv2.approxPolyDP(contour, .38, True).reshape(-1, 2)
            if len(contour) >= 3:
                paths.append("M" + "L".join(f"{x} {y}" for x, y in contour) + "Z")
        if not paths:
            continue
        picked = rgba[ys[labels == label], xs[labels == label]]
        r, g, b, a = np.median(picked, axis=0).astype(int)
        color = f"#{r:02x}{g:02x}{b:02x}"
        opacity = a / 255
        layers.append((a, f'<path d="{"".join(paths)}" fill="{color}" stroke="{color}" stroke-width="1.2" stroke-linejoin="round" fill-opacity="{opacity:.3f}" stroke-opacity="{opacity:.3f}" fill-rule="evenodd"/>'))
    layers.sort(key=lambda item: item[0], reverse=True)
    return "\n".join(layer for _, layer in layers)


def main():
    states = {}
    for name, (label, relative_source) in SOURCES.items():
        artwork = trace(REPO / relative_source)
        svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 400" role="img" aria-labelledby="title desc">
  <title id="title">Yonda · {label}</title>
  <desc id="desc">由现有透明 PNG 在本机进行 128 色高保真矢量描摹。</desc>
  {artwork}
</svg>
'''
        (ROOT / f"{name}.svg").write_text(svg)
        states[name] = {
            "paths": {},
            "orphanSVG": artwork,
            "idle": "bob" if name in {"executing", "success"} else "sway" if name in {"idle", "thinking", "recording"} else "breathe-y",
        }
    config = {
        "title": "Yonda High-Fidelity Vector Animator",
        "subtitle": "现有 PNG 本地 128 色矢量描摹 · 状态间淡入淡出",
        "viewBox": "0 0 400 400",
        "size": {"width": 400, "height": 400},
        "exportName": "YondaHighFidelityAnimator",
    }
    (ROOT / "yonda-vector.states.js").write_text(
        "window.PREVIEW_CONFIG = " + json.dumps(config, ensure_ascii=False, indent=2) + ";\n"
        "window.STATES_DATA = " + json.dumps(states, ensure_ascii=False, separators=(",", ":")) + ";\n"
    )
    assert len(states) == 9
    assert all((ROOT / f"{name}.svg").stat().st_size > 100_000 for name in states)
    print(f"已生成 {len(states)} 个高保真 SVG。")


if __name__ == "__main__":
    main()
