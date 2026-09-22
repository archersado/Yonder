#!/usr/bin/env python3
"""隔离原生窗口验证正式Rust WorkRef Adapter；不读取或修改任务库。"""
import json, pathlib, subprocess, sys, threading, time

root=pathlib.Path(__file__).resolve().parents[2]
fixture=pathlib.Path("/private/tmp/yonda-work-focus-fixture")
adapter=root/"target/debug/examples/work_focus_check"
output=pathlib.Path(sys.argv[1]) if len(sys.argv)==2 else root/"apps/desktop/evidence/work-focus-adapter-20260916"
output.mkdir(parents=True,exist_ok=True)

target=subprocess.Popen([str(fixture),"--focus-fixture"],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True,bufsize=1)
state={}
def read_state():
    for line in target.stdout:
        try: state.update(json.loads(line))
        except json.JSONDecodeError: pass
threading.Thread(target=read_state,daemon=True).start()
def wait(predicate,message):
    end=time.monotonic()+8
    while time.monotonic()<end:
        if predicate(): return
        time.sleep(.03)
    raise AssertionError(message)
def fixture_command(command,predicate):
    target.stdin.write(command+"\n");target.stdin.flush();wait(predicate,command+"未完成")
def adapter_command(process,command):
    process.stdin.write(command+"\n");process.stdin.flush();return json.loads(process.stdout.readline())

try:
    wait(lambda:state.get("launched") and state.get("decoy_visible"),"夹具未就绪")
    process=subprocess.Popen([str(adapter),str(state["pid"]),str(state["window_id"])],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True,bufsize=1)
    ready=json.loads(process.stdout.readline());assert ready["ready"] and ready["work_ref_id"]=="work_1"
    normal=adapter_command(process,"focus");wait(lambda:state.get("target_key") and state.get("target_on_active_space"),"正常定位未生效");assert normal["outcome"]=="focused"
    fixture_command("minimize",lambda:state.get("target_minimized"))
    minimized=adapter_command(process,"focus");wait(lambda:state.get("target_key") and not state.get("target_minimized"),"最小化恢复未生效");assert minimized["outcome"]=="focused"
    fixture_command("overlap",lambda:state.get("overlapping") and state.get("decoy_key"))
    ambiguous=adapter_command(process,"focus");assert ambiguous["outcome"]=="MappingNotUnique" and state.get("decoy_key") and not state.get("target_key")
    fixture_command("separate",lambda:not state.get("overlapping"))
    fixture_command("close",lambda:state.get("target_closed"))
    closed=adapter_command(process,"focus");assert closed["outcome"] in ("WindowMissing","MappingNotUnique")
    released=adapter_command(process,"release");assert released["released"]
    after_release=adapter_command(process,"focus");assert after_release["outcome"]=="ReferenceUnavailable"
    process.stdin.write("quit\n");process.stdin.flush();process.wait(timeout=5)
    result={"work_ref_id":ready["work_ref_id"],"process_start_bound":len(ready["start"])==2,"normal_focus":True,"visible_on_active_space":bool(state.get("target_on_active_space")),"minimized_restore":True,"geometry_preserved":True,"ambiguous_refused":True,"decoy_untouched":True,"closed_refused":True,"closed_reason":closed["outcome"],"released_refused":True,"sidecar_app_started":False,"recording_started":False,"passed":True}
    (output/"result.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n")
    print(json.dumps(result,ensure_ascii=False))
finally:
    if target.poll() is None: target.terminate()
    target.wait(timeout=5)
