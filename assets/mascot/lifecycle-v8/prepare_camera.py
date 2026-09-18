"""保留相机生成原图，按轮廓去除外部光晕；分离抱相机前臂。"""
from pathlib import Path
from PIL import Image,ImageDraw,ImageChops,ImageFilter
root=Path(__file__).resolve().parents[3];folder=root/'assets/mascot/lifecycle-v8';runtime=root/'apps/desktop/ui/runtime/lifecycle-v8'
src=Image.open(folder/'camera-source.png').convert('RGBA');mask=Image.new('L',src.size);d=ImageDraw.Draw(mask)
d.rounded_rectangle((339,284,1265,810),radius=75,fill=255);d.rounded_rectangle((647,237,949,331),radius=48,fill=255);d.rounded_rectangle((1045,244,1178,303),radius=23,fill=255)
src.putalpha(ImageChops.multiply(src.getchannel('A'),mask.filter(ImageFilter.GaussianBlur(.7))))
prop=Image.new('RGBA',(400,400));prop.alpha_composite(src.crop(src.getchannel('A').getbbox()).resize((84,52)),(185,270));prop.save(folder/'camera.png');prop.save(runtime/'camera.png')
original=Image.open(root/'assets/mascot/lifecycle-v4/frames/recording-00.png').convert('RGBA');mask=Image.new('L',original.size);d=ImageDraw.Draw(mask)
d.ellipse((164,256,226,316),fill=255);d.ellipse((216,255,276,313),fill=255);mask=mask.filter(ImageFilter.GaussianBlur(1))
limb=original.copy();limb.putalpha(ImageChops.multiply(original.getchannel('A'),mask));body=original.copy();body.putalpha(ImageChops.multiply(original.getchannel('A'),ImageChops.invert(mask)))
patch=Image.open(root/'apps/desktop/ui/runtime/lifecycle-v6/executing-body.png');patch.putalpha(ImageChops.multiply(patch.getchannel('A'),mask));body.alpha_composite(patch)
for name,im in [('recording-body',body),('recording-gesture',limb)]:im.save(folder/f'{name}.png');im.save(runtime/f'{name}.png')
assert prop.getchannel('A').getbbox()==(185,270,269,322)
for im in [prop,body,limb]:assert im.mode=='RGBA' and all(im.getpixel(p)[3]==0 for p in [(0,0),(399,0),(0,399),(399,399)])
