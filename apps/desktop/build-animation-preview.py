"""生成独立九态预览，内联当前pet渲染器与PNG；不连接或写入任务。"""
import base64
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[2]
ui = root/'apps/desktop/ui'
pet = (ui/'pet.html').read_text()
config_match = re.search(r'(<script id="state-animations" type="application/json">)(.*?)(</script>)', pet)
config = json.loads(config_match[2])
plan = json.loads((root/'assets/mascot/lifecycle-v4/asset-plan.json').read_text())
actual = set(config)
for item in plan['items']:
    state = item['id']
    if state not in config:
        # 五个未接线状态仅模拟视觉，不生成产品事件。
        index = 3 if state in ['listening', 'failed'] else 0
        config[state] = {'duration_ms':3600, 'playback':'loop', 'static_frame':index,
                         'body_frames':[index]*4,
                         'files':[str(root/f'assets/mascot/lifecycle-v4/frames/{state}-{i:02}.png') for i in range(4)],
                         'blink_files':[str(root/f'assets/mascot/lifecycle-v6/blinks/{state}-{i:02}.png') for i in range(4)]}
config['listening'].update(static_frame=3,body_frames=[3]*4)
config['thinking'].update(static_frame=0,body_frames=[0]*4)
config['failed'].update(static_frame=3,body_frames=[3]*4)
config['success'].update(static_frame=2,body_frames=[2]*4)
config['recording'].update(body_frames=[0]*4,prop_file=str(root/'assets/mascot/lifecycle-v8/camera.png'))
pet = pet[:config_match.start(2)]+json.dumps(config)+pet[config_match.end(2):]
pet = pet.replace('<link rel="stylesheet" href="pet.css" />', '<style>'+(ui/'pet.css').read_text()+'</style>')
pet = pet.replace('<script src="pet.js" defer></script>', '').replace('</body>', '<script>'+(ui/'pet.js').read_text()+'</script></body>')
for path in sorted(set(re.findall(r'(?:runtime/|/Users/)[A-Za-z0-9_./-]+\.png', pet)), key=len, reverse=True):
    file = Path(path) if path.startswith('/') else ui/path
    pet = pet.replace(path, 'data:image/png;base64,'+base64.b64encode(file.read_bytes()).decode())
states = [[item['id'], item['label'], '当前播放版本' if item['id'] in actual else '循环动画样本 · 事件未接线'] for item in plan['items']]
fixture = '''<script>window.__TAURI_INTERNALS__={invoke:async c=>{if(c==='pet_is_visible')return true;if(c==='pet_task_state')return [true,STATE];if(c==='pet_dock')throw Error('预览保持展开');return false;}};</script>'''
frames = []
embedded = re.search(r'(<script id="state-animations" type="application/json">)(.*?)(</script>)',pet)
embedded_config = json.loads(embedded[2])
for state,_,_ in states:
    subset = {'idle':embedded_config['idle'], state:embedded_config[state]}
    frame = pet[:embedded.start(2)]+json.dumps(subset)+pet[embedded.end(2):]
    frame = frame.replace('<head>', '<head>'+fixture.replace('STATE',json.dumps(state)))
    monitor = '''<script>setInterval(()=>{const p=document.querySelector('#pet');parent.postMessage({type:'yonda-preview',state:STATE,ready:p.classList.contains('frames-ready'),assetError:p.dataset.assetError??'',cue:getComputedStyle(document.querySelector('.state-cue')).display,record:getComputedStyle(document.querySelector('.record-badge')).display,dots:[...document.querySelectorAll('.wait-dots i')].map(x=>x.style.opacity).join(','),breath:document.querySelector('.breath').style.transform,blink:Number(document.querySelector('.state-eyelids').style.opacity),prop:document.querySelector('.state-prop').style.transform,flash:Number(document.querySelector('.camera-flash').style.opacity),mix:Number(document.querySelector('.state-frame-next').style.opacity),gesture:document.querySelector('.state-gesture').style.transform,pose:p.dataset.pose,action:document.querySelector('.response').style.transform+document.querySelector('.state-tail').style.transform,paws:p.style.getPropertyValue('--state-paw-right')+p.style.getPropertyValue('--state-paw-left')},'*')},40);</script>'''
    frames.append(frame.replace('</body>',monitor.replace('STATE',json.dumps(state))+'</body>'))
html = '''<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Yonda · 状态动画预览</title>
<style>*{box-sizing:border-box}body{margin:0;background:#101923;color:#edf4fa;font-family:-apple-system,BlinkMacSystemFont,sans-serif;padding:28px 5vw}header,.cards{max-width:1100px;margin:auto}h1{font-size:28px;margin:0 0 10px}p{color:#a9bdcf;line-height:1.6;margin:6px 0}.toolbar{display:flex;gap:12px;margin:16px 0 24px}button{padding:10px 18px;border:1px solid #415c72;border-radius:12px;background:#203346;color:#eff8ff;font-size:14px;cursor:pointer}button:hover{background:#2d455d}button:focus-visible{outline:2px solid #a1d8ff}.cards{display:grid;grid-template-columns:repeat(3,minmax(220px,1fr));gap:16px}.card{border:1px solid #30465a;border-radius:18px;overflow:hidden;background:#172635}.stage{display:flex;align-items:center;justify-content:center;height:220px;background:radial-gradient(ellipse at 50% 70%,#2f455a 0,#1c2d3d 60%)}.light .stage{background:#f0f4f7}iframe{width:200px;height:200px;border:0}.caption{padding:14px 18px;border-top:1px solid #30465a}.caption h2{margin:0 0 4px;font-size:16px}.caption p{font-size:12px;margin:0}@media(max-width:780px){.cards{grid-template-columns:repeat(2,minmax(200px,1fr))}}@media(max-width:490px){.cards{grid-template-columns:1fr}}</style>
<header><h1>Yonda 九种状态</h1><p>每个状态均保留呼吸与眨眼。执行短臂交替抬起、落下。</p><p>四态复用当前播放器；其余五态模拟专属循环动作。独立预览，不创建/执行任务或启动录制。</p><div class="toolbar"><button id="replay">重新播放动画</button><button id="theme" aria-pressed="false">切换浅色背景</button></div></header><main class="cards"></main><script>
const states=STATES,frames=FRAMES;const main=document.querySelector('main');
window.previewChecks={};window.addEventListener('message',e=>{const d=e.data;if(d?.type!=='yonda-preview'||!states.some(s=>s[0]===d.state)||!Array.from(document.querySelectorAll('iframe')).some(f=>f.contentWindow===e.source))return;const last=window.previewChecks[d.state]??{};window.previewChecks[d.state]={ready:d.ready,assetError:d.assetError,dotsChanged:last.dotsChanged||Boolean(last.dots&&last.dots!==d.dots),dots:d.dots,cue:d.cue,record:d.record,breathChanged:last.breathChanged||Boolean(last.breath&&last.breath!==d.breath),blinkSeen:last.blinkSeen||d.blink>.95,pawsChanged:last.pawsChanged||Boolean(last.paws&&last.paws!==d.paws),propChanged:last.propChanged||Boolean(last.prop&&last.prop!==d.prop),prop:d.prop,flashSeen:last.flashSeen||d.flash>.7,hint:d.hint,maxMix:Math.max(last.maxMix??0,d.mix),gestureChanged:last.gestureChanged||Boolean(last.gesture&&last.gesture!==d.gesture),gesture:d.gesture,poses:[...new Set([...(last.poses??[]),d.pose].filter(x=>x!==undefined))],actionChanged:last.actionChanged||Boolean(last.action&&last.action!==d.action),action:d.action,breath:d.breath,paws:d.paws};});
states.forEach(([state,title,note],i)=>{const card=document.createElement('section');card.className='card';card.innerHTML='<div class="stage"><iframe sandbox="allow-scripts" title="'+title+'动画"></iframe></div><div class="caption"><h2>'+title+'</h2><p>'+note+'</p></div>';main.append(card);card.querySelector('iframe').srcdoc=frames[i];});
document.querySelector('#replay').onclick=()=>document.querySelectorAll('iframe').forEach((f,i)=>f.srcdoc=frames[i]);
document.querySelector('#theme').onclick=()=>{const light=document.body.classList.toggle('light'),b=document.querySelector('#theme');b.setAttribute('aria-pressed',String(light));b.textContent=light?'切换深色背景':'切换浅色背景';};
</script></html>'''
html = html.replace('STATES',json.dumps(states,ensure_ascii=False)).replace('FRAMES',json.dumps(frames,ensure_ascii=False).replace('</script>', '<\\/script>'))
path = Path(sys.argv[1] if len(sys.argv)>1 else '/private/tmp/yonda-state-animation-preview.html')
path.write_text(html)
print('九态独立预览已生成：'+str(path))

# 独立单角色夹具：用于状态打断/减少动态效果测试，不连接任务库。
transition=pet.replace('<head>', '<head><script>window.previewState="recording";</script>'+fixture.replace('STATE','window.previewState'))
Path('/private/tmp/yonda-transition-check.html').write_text(transition)

# 隐藏态状态变化夹具：缩短闲置时间，只验证既有 dock/wake 调用次数。
dock_fixture = '''<script>window.previewState="idle";window.previewWakeCount=0;window.previewFail=false;window.__TAURI_INTERNALS__={invoke:async c=>{if(c==='pet_is_visible')return true;if(c==='pet_task_state'){if(window.previewFail)throw Error('状态不可用');return [true,window.previewState]}if(c==='pet_dock')return 'right';if(c==='pet_wake'){window.previewWakeCount++;return true}return false;}};</script>'''
docked = pet.replace('const IDLE_MS = 180000;', 'const IDLE_MS = 80;').replace('<head>', '<head>'+dock_fixture)
Path('/private/tmp/yonda-docked-state-change.html').write_text(docked)
