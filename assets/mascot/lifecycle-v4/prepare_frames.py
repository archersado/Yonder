"""纯绿色分离底提取；保留原图，复用既有画布与边界检查。"""
import importlib.util
from pathlib import Path
from PIL import Image, ImageChops

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('prepare_v2', ROOT.parent/'lifecycle-v2/prepare_frames.py')
previous = importlib.util.module_from_spec(spec)
spec.loader.exec_module(previous)
clean_component = previous.extract


def key(image):
    image = image.convert('RGBA')
    pixels = []
    for r, g, b, original_alpha in image.getdata():
        # 角色奶白/蓝/金/粉色不含绿色；分离背景并消除边缘绿色混合。
        alpha = 1 - max(0, g-max(r, b))/255
        if alpha < .08 or original_alpha == 0:
            pixels.append((0, 0, 0, 0))
        else:
            pixels.append((min(255, round(r/alpha)),
                           min(255, max(0, round((g-255*(1-alpha))/alpha))),
                           min(255, round(b/alpha)), round(alpha*original_alpha)))
    result = Image.new('RGBA', image.size)
    result.putdata(pixels)
    return result


def extract(image):
    keyed = key(image)
    cleaned = clean_component(keyed)
    # 连通区域只允许收窄原始透明度，不能把绿色空隙填成黑色像素。
    cleaned.putalpha(ImageChops.multiply(cleaned.getchannel('A'), keyed.getchannel('A')))
    return cleaned


def despill_frames():
    # 缩小后的插值也可能带出绿色；只改副本，不改生成原图。
    for path in (ROOT/'frames').glob('*.png'):
        image = Image.open(path).convert('RGBA')
        image.putdata([(r, min(g, max(r, b)), b, a) for r, g, b, a in image.getdata()])
        image.save(path)


if __name__ == '__main__':
    test = key(Image.new('RGB', (1, 1), (0, 255, 0)))
    assert test.getpixel((0, 0)) == (0, 0, 0, 0)
    assert key(Image.new('RGB', (1, 1), (245, 235, 220))).getpixel((0, 0)) == (245, 235, 220, 255)
    previous.ROOT = ROOT
    previous.extract = extract
    previous.main()
    despill_frames()
