"""沿用已授权Pillow：同角色闭眼参考仅提取眼区，身体/肢体不进入眨眼层。"""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFilter, ImageChops
import json

ROOT = Path(__file__).resolve().parent
ASSETS = ROOT.parent
closed = Image.open(ASSETS / 'lifecycle-v5/frames/idle-02.png').convert('RGBA')
reference = Image.open(ASSETS / 'lifecycle-v5/frames/idle-00.png').convert('RGBA')


def eye(image, left):
    # 只在脸内找最大深蓝瞳孔连通域，避免把尾巴/蓝色爪尖当眼睛。
    x0, x1 = (115, 205) if left else (205, 290)
    pixels = image.load()
    remaining = set()
    for y in range(110, 245):
        for x in range(x0, x1):
            r, g, b, a = pixels[x, y]
            if a > 200 and r < 100 and g < 140 and b > r + 20 and b > 60:
                remaining.add((x, y))
    groups = []
    while remaining:
        stack = [remaining.pop()]
        group = []
        while stack:
            x, y = stack.pop()
            group.append((x, y))
            for point in [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]:
                if point in remaining:
                    remaining.remove(point)
                    stack.append(point)
        groups.append(group)
    group = max(groups, key=len, default=[])
    if len(group) < 100:
        return None
    return (sum(x for x, _ in group)/len(group), sum(y for _, y in group)/len(group))


def patch(left):
    cx, cy = eye(reference, left)
    rx, ry = (31, 35) if left else (29, 34)
    mask = Image.new('L', closed.size)
    ImageDraw.Draw(mask).ellipse((cx-rx, cy-ry, cx+rx, cy+ry), fill=255)
    mask = mask.filter(ImageFilter.GaussianBlur(1.2))
    result = closed.copy()
    result.putalpha(ImageChops.multiply(mask, closed.getchannel('A')))
    return result, (cx, cy)


if __name__ == '__main__':
    out = ROOT / 'blinks'
    out.mkdir(exist_ok=True)
    report = []
    pieces = [patch(True), patch(False)]
    plan = json.loads((ASSETS/'lifecycle-v4/asset-plan.json').read_text())
    for item in plan['items']:
        state = item['id']
        source = ASSETS/('lifecycle-v5' if state == 'idle' else 'lifecycle-v4')/'frames'
        for i in range(4):
            original = Image.open(source/f'{state}-{i:02}.png').convert('RGBA')
            overlay = Image.new('RGBA', original.size)
            detected = []
            for left, (piece, origin) in zip([True, False], pieces):
                center = eye(original, left)
                detected.append(center is not None)
                # 已闭眼/不可识别帧不以蓝爪误定位；此帧保留原眼区，不叠加。
                if center is not None:
                    overlay.alpha_composite(piece, (round(center[0]-origin[0]), round(center[1]-origin[1])))
            overlay.save(out/f'{state}-{i:02}.png')
            report.append({'state':state, 'frame':i, 'eyes_detected':detected, 'rgba':True})
    (ROOT/'blink-check.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n')
    print('眼区透明层已生成；不含身体动作')
