(() => {
  const invoke = (command, args) => window.__TAURI_INTERNALS__?.invoke(command, args) ?? Promise.reject(new Error('圈选能力未提供'));
  const selector = document.querySelector('#selector'), selection = document.querySelector('#selection'), review = document.querySelector('#review'), preview = document.querySelector('#preview'), status = document.querySelector('#status');
  let start, timer, openedAt = 0;
  const close = () => invoke('region_preview_close').catch(() => {});
  const clear = () => { start = undefined; selection.hidden = true; review.hidden = true; preview.removeAttribute('src'); clearTimeout(timer); };
  const armTimeout = () => { clearTimeout(timer); timer = setTimeout(close, 30000); };
  const draw = point => { const x=Math.min(start.x,point.x),y=Math.min(start.y,point.y),w=Math.abs(start.x-point.x),h=Math.abs(start.y-point.y); Object.assign(selection.style,{left:`${x}px`,top:`${y}px`,width:`${w}px`,height:`${h}px`}); selection.hidden=false; return {x,y,width:w,height:h,viewportWidth:innerWidth,viewportHeight:innerHeight}; };
  selector.addEventListener('pointerdown', event => { if (event.button !== 0 || Date.now() - openedAt < 350) return; start={x:event.clientX,y:event.clientY}; selector.setPointerCapture(event.pointerId); armTimeout(); });
  selector.addEventListener('pointermove', event => { if (start) draw({x:event.clientX,y:event.clientY}); });
  selector.addEventListener('pointerup', async event => { if (!start) return; const rect=draw({x:event.clientX,y:event.clientY}); start=undefined; if (rect.width<12||rect.height<12) return clear(); try { await invoke('region_preview_hide_for_capture'); await new Promise(resolve=>setTimeout(resolve,120)); const image=await invoke('region_preview_capture',{rect}); preview.src=`data:image/png;base64,${image}`; selector.hidden=true; review.hidden=false; await invoke('region_preview_show_review',{rect}); } catch(error) { selector.hidden=true; review.hidden=false; preview.removeAttribute('src'); status.textContent=error.message||'截图不可用'; await invoke('region_preview_show_review',{rect}).catch(() => {}); } });
  document.querySelector('#close').addEventListener('click', close); document.querySelector('#again').addEventListener('click', () => { clear(); invoke('region_preview_reselect').catch(() => close()); });
  addEventListener('keydown', event => { if (event.key==='Escape') { event.preventDefault(); close(); } });
  addEventListener('yonda-region-open', () => { clear(); openedAt=Date.now(); selector.hidden=false; status.textContent='仅在本机内存预览；发送给 Agent 的功能待接入。'; armTimeout(); });
})();
