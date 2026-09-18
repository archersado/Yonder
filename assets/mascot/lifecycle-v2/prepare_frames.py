"""用户授权的素材处理工具；运行时素材包不包含此脚本。需要 Pillow。"""
from collections import deque
from pathlib import Path
import json
from PIL import Image, ImageFilter

ROOT = Path(__file__).resolve().parent


def extract(image):
    has_alpha = 'A' in image.getbands()
    image = image.convert('RGBA')
    w, h = image.size
    pixels = list(image.getdata())
    # ponytail: 仅适用于本批中性棋盘背景；其他背景需重新分割而非调大阈值。
    mask = bytearray(255 if (p[3] > 128 if has_alpha else max(p[:3]) - min(p[:3]) > 3) else 0 for p in pixels)
    mask = bytearray(Image.frombytes('L', (w, h), bytes(mask)).filter(ImageFilter.MaxFilter(7)).filter(ImageFilter.MinFilter(7)).tobytes())
    seen = bytearray(w * h)
    components = []
    for start in range(w * h):
        if not mask[start] or seen[start]:
            continue
        seen[start] = 1
        queue = deque([start])
        component = []
        while queue:
            i = queue.popleft()
            component.append(i)
            for j in (i - w, i + w, i - 1 if i % w else -1,
                      i + 1 if i % w < w - 1 else -1):
                if 0 <= j < w * h and mask[j] and not seen[j]:
                    seen[j] = 1
                    queue.append(j)
        components.append(component)
    assert components, '未找到角色'
    foreground = bytearray(w * h)
    for i in max(components, key=len):
        foreground[i] = 255
    # 恢复角色内部小块中性高光；较大棋盘空隙仍透明。
    seen = bytearray(w * h)
    for start in range(w * h):
        if foreground[start] or seen[start]:
            continue
        seen[start] = 1
        queue = deque([start])
        hole = []
        border = False
        while queue:
            i = queue.popleft()
            hole.append(i)
            border |= i < w or i >= w * (h - 1) or i % w in (0, w - 1)
            for j in (i - w, i + w, i - 1 if i % w else -1,
                      i + 1 if i % w < w - 1 else -1):
                if 0 <= j < w * h and not foreground[j] and not seen[j]:
                    seen[j] = 1
                    queue.append(j)
        transitions = sum(1 for i in hole if i % w < w-1 and not foreground[i+1]
                          and abs(pixels[i][0]-pixels[i+1][0]) > 35)
        if not border and (len(hole) < 200 or transitions < len(hole)*0.02):
            for i in hole:
                foreground[i] = 255
    alpha = Image.frombytes('L', (w, h), bytes(foreground)).filter(ImageFilter.MinFilter(3))
    image.putalpha(alpha)
    return image


def main():
    plan = json.loads((ROOT / 'asset-plan.json').read_text())
    extracted = {}
    for item in plan['items']:
        sheet = Image.open(ROOT / item['source_sheet'])
        w, h = sheet.size
        frames = [extract(sheet.crop((x*w//2, y*h//2,
                                                    (x+1)*w//2, (y+1)*h//2)))
                                 for y in range(2) for x in range(2)]
        extracted[item['id']] = [frames[i] for i in item.get('source_frame_order', range(4))]
    boxes = [im.getbbox() for frames in extracted.values() for im in frames]
    scale = min(344 / max(b[2]-b[0] for b in boxes),
                344 / max(b[3]-b[1] for b in boxes))
    output = ROOT / 'frames'
    output.mkdir(exist_ok=True)
    evidence = []
    for item in plan['items']:
        state = item['id']
        item['files'] = []
        for i, im in enumerate(extracted[state]):
            box = im.getbbox()
            feet = im.getchannel('A').crop((0, box[3]-24, im.width, box[3])).getbbox()
            anchor_x = (feet[0]+feet[2])/2
            resized = im.resize((round(im.width*scale), round(im.height*scale)), Image.Resampling.LANCZOS)
            canvas = Image.new('RGBA', (400, 400))
            lift = [0, 16, 8, 0][i] if state == 'success' else 0
            canvas.alpha_composite(resized, (round(200-anchor_x*scale), round(376-box[3]*scale-lift)))
            angle = item.get('normalization_rotation_deg', [0, 0, 0, 0])[i]
            if angle:
                canvas = canvas.rotate(angle, Image.Resampling.BICUBIC, center=(200, 376))
            filename = f'frames/{state}-{i:02d}.png'
            canvas.save(ROOT / filename)
            item['files'].append(filename)
            bounds = canvas.getchannel('A').point(lambda p: 255 if p > 16 else 0).getbbox()
            assert bounds and min(bounds[:2]) >= 8 and max(bounds[2:]) <= 392, (state, i, bounds)
            assert canvas.getchannel('A').getextrema() == (0, 255)
            evidence.append({'file': filename, 'size': [400, 400], 'mode': 'RGBA', 'bounds': bounds})
        item['static_frame'] = 3 if item['playback'] == 'hold' else 0
    plan['canvas_px'] = [400, 400]
    plan['logical_px'] = [200, 200]
    plan['status'] = '待人工边缘与姿态验收'
    (ROOT/'asset-plan.json').write_text(json.dumps(plan, ensure_ascii=False, indent=2)+'\n')
    (ROOT/'frame-check.json').write_text(json.dumps(evidence, ensure_ascii=False, indent=2)+'\n')
    print(f'已生成并检查 {len(evidence)} 张透明帧')


if __name__ == '__main__':
    main()
