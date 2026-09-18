"""生成去除粉色掌心圆圈的等待用户运行时副本；保留 v4 原图。"""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parents[3]
CENTERS = {1: (256, 275), 2: (262, 240), 3: (257, 237)}

for index in range(4):
    source = ROOT / f"assets/mascot/lifecycle-v4/frames/waiting_for_user-{index:02}.png"
    image = Image.open(source).convert("RGBA")
    alpha = image.getchannel("A").tobytes()
    if index in CENTERS:
        cx, cy = CENTERS[index]
        left, right = image.getpixel((cx - 17, cy))[:3], image.getpixel((cx + 17, cy))[:3]
        fill = Image.new("RGBA", image.size)
        pixels = fill.load()
        for y in range(cy - 20, cy + 21):
            for x in range(cx - 20, cx + 21):
                ratio = max(0, min(1, (x - cx + 17) / 34))
                shade = (y - cy) * -0.25
                pixels[x, y] = (*[max(0, min(255, round(a * (1 - ratio) + b * ratio + shade))) for a, b in zip(left, right)], image.getpixel((x, y))[3])
        mask = Image.new("L", image.size)
        ImageDraw.Draw(mask).ellipse((cx - 16, cy - 17, cx + 16, cy + 17), fill=255)
        image = Image.composite(fill, image, mask.filter(ImageFilter.GaussianBlur(3)))
        image.putalpha(Image.frombytes("L", image.size, alpha))
    assert image.getchannel("A").tobytes() == alpha
    for folder in (ROOT / "assets/mascot/lifecycle-v10", ROOT / "apps/desktop/ui/runtime/lifecycle-v10"):
        folder.mkdir(parents=True, exist_ok=True)
        image.save(folder / f"waiting_for_user-{index:02}.png")
