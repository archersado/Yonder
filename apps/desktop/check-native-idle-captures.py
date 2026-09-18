"""验证当前200点桌宠原生截图的透明边界与完整眨眼；不代表九态全部通过。"""
import json
from pathlib import Path
import sys
from PIL import Image

folder=Path(sys.argv[1])
report=json.loads((folder/'native-result.json').read_text())
assert report['single_host'] and report['pet_on_visible_desktop'] and report['no_task_mutation_by_probe']
assert report['bounds']['Width']==200 and report['bounds']['Height']==200
counts=[];bounds=[]
for i in range(report['captures']):
    im=Image.open(folder/f'native-{i}.png').convert('RGBA');w,h=im.size
    assert all(im.getpixel(p)[3]==0 for p in [(0,0),(w-1,0),(0,h-1),(w-1,h-1)])
    box=im.getchannel('A').getbbox();assert box and 0<box[0]<box[2]<w and 0<box[1]<box[3]<h
    bounds.append(box)
    eye=im.crop((int(w*.3),int(h*.3125),int(w*.7),int(h*.5625)))
    counts.append(sum(a>180 and r<120 and g<145 and b>r+20 and b>65 for r,g,b,a in eye.getdata()))
closed=min(range(1,len(counts)-1),key=counts.__getitem__)
assert counts[closed]<min(counts[closed-1],counts[closed+1])*.6,'未观察到完整闭眼再睁眼'
assert len(set(bounds))>2,'原生角色边界没有变化，需检查动态计时'
result={'result':'PASS','scope':'原生待命透明边界、动态轮廓和完整眨眼','closed_capture':closed,'eye_counts':counts,'distinct_bounds':len(set(bounds))}
(folder/'idle-analysis.json').write_text(json.dumps(result,ensure_ascii=False,indent=2))
print(result['scope']+'：PASS')
