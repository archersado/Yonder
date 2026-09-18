"""保留原帧，提取头部与身体；仅补齐头颈转动时露出的窄连接区。"""
from pathlib import Path
from PIL import Image, ImageDraw, ImageChops, ImageFilter
root=Path(__file__).resolve().parents[3]
for state in ['listening','thinking','failed']:
    index=3 if state=='failed' else 0
    original=Image.open(root/f'assets/mascot/lifecycle-v4/frames/{state}-{index:02}.png').convert('RGBA')
    mask=Image.new('L',original.size);d=ImageDraw.Draw(mask)
    d.polygon([(0,0),(400,0),(400,220),(310,220),(310,245),(265,253),(250,265),(175,265),(155,253),(112,245),(112,220),(0,220)],fill=255)
    # 侧鳞片位于主要轮廓之外，属于头部而不是身体。
    d.rectangle((98,217,115,230),fill=255);d.rectangle((127,245,145,256),fill=255)
    mask=mask.filter(ImageFilter.GaussianBlur(.6))
    head=original.copy();head.putalpha(ImageChops.multiply(original.getchannel('A'),mask))
    body=original.copy();body.putalpha(ImageChops.multiply(original.getchannel('A'),ImageChops.invert(mask)))
    neck=original.crop((207,270,231,292)).resize((68,43))
    neckmask=Image.new('L',neck.size);ImageDraw.Draw(neckmask).ellipse((0,0,67,42),fill=255)
    neck.putalpha(ImageChops.multiply(neck.getchannel('A'),neckmask.filter(ImageFilter.GaussianBlur(2))))
    body.alpha_composite(neck,(181,242))
    for name,im in [('head',head),('body',body)]:
        im.save(root/f'assets/mascot/lifecycle-v7/{state}-{name}.png')
        im.save(root/f'apps/desktop/ui/runtime/lifecycle-v7/{state}-{name}.png')
    assembled=body.copy();assembled.alpha_composite(head);assembled.save(f'/private/tmp/{state}-rig.png')
