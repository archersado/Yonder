"""从已授权透明原帧分离活动爪/翼，保持原图；身体补片仅限被分离区域。"""
from pathlib import Path
from PIL import Image, ImageDraw, ImageChops, ImageFilter
root=Path(__file__).resolve().parents[3]
clean=Image.open(root/'apps/desktop/ui/runtime/lifecycle-v6/executing-body.png').convert('RGBA')
for state,index,box in [('waiting_for_user',3,(228,243,284,313)),('success',2,(234,234,283,298))]:
    src=Image.open(root/f'assets/mascot/lifecycle-v4/frames/{state}-{index:02}.png').convert('RGBA')
    mask=Image.new('L',src.size);ImageDraw.Draw(mask).ellipse(box,fill=255);mask=mask.filter(ImageFilter.GaussianBlur(3 if state=='waiting_for_user' else 1))
    limb=src.copy();limb.putalpha(ImageChops.multiply(src.getchannel('A'),mask))
    bounds=limb.getchannel('A').getbbox()
    assert bounds and bounds[0]>=box[0]-10 and bounds[1]>=box[1]-10 and bounds[2]<=box[2]+11 and bounds[3]<=box[3]+11, (state,bounds)
    body=src.copy();body.putalpha(ImageChops.multiply(src.getchannel('A'),ImageChops.invert(mask)))
    patch=clean.copy();patch.putalpha(ImageChops.multiply(clean.getchannel('A'),mask));body.alpha_composite(patch)
    for name,img in [('gesture',limb),('body',body)]:
        img.save(root/f'assets/mascot/lifecycle-v7/{state}-{name}.png');img.save(root/f'apps/desktop/ui/runtime/lifecycle-v7/{state}-{name}.png')
src=Image.open(root/'apps/desktop/ui/runtime/lifecycle-v5/idle-00.png').convert('RGBA');mask=Image.new('L',src.size)
ImageDraw.Draw(mask).polygon([(262,246),(286,236),(323,251),(335,282),(312,270),(290,292),(263,288)],fill=255)
mask=mask.filter(ImageFilter.GaussianBlur(.8));src.putalpha(ImageChops.multiply(src.getchannel('A'),mask))
for folder in ['assets/mascot/lifecycle-v7','apps/desktop/ui/runtime/lifecycle-v7']:src.save(root/f'{folder}/paused-gesture.png')
