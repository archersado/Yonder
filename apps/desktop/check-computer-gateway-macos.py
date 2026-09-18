#!/usr/bin/env python3
"""通过正式UDS Gateway和隔离原生窗口验证Agent CUA完整生命周期。"""
import json, pathlib, socket, subprocess, sys, threading, time

root=pathlib.Path(__file__).resolve().parents[2]; output=pathlib.Path(sys.argv[1]); output.mkdir(parents=True,exist_ok=False)
fixture=subprocess.Popen(["/private/tmp/yonda-cu-gateway-fixture"]+(["--delay-ax"] if "--interrupt" in sys.argv or "--slow" in sys.argv else []),stdout=subprocess.PIPE,text=True); state={}
def read_fixture():
    for line in fixture.stdout:
        try: state.update(json.loads(line))
        except json.JSONDecodeError: pass
threading.Thread(target=read_fixture,daemon=True).start()
end=time.monotonic()+15
while not (state.get("ready") and state.get("launched") and state.get("app_active") and state.get("target_key")) and time.monotonic()<end: time.sleep(.05)
assert state.get("ready") and state.get("launched") and state.get("app_active") and state.get("target_key"),"隔离原生窗口未激活"
focused=subprocess.run(["/private/tmp/yonda-focus-target",str(state["pid"]),str(state["window_id"])],capture_output=True,text=True,timeout=10)
focus_result=json.loads(focused.stdout);assert focused.returncode==0 and focus_result.get("focused") and focus_result.get("geometry_unchanged"),focus_result
time.sleep(.5)

stream=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM);stream.settimeout(45);stream.connect(str(pathlib.Path.home()/"Library/Application Support/com.yonder.desktop/agent.sock"));buffer=stream.makefile("rwb",buffering=0);counter=0
def call(method,capability,**params):
    global counter
    counter+=1;request={"jsonrpc":"2.0","id":f"computer-{counter}","method":method,"params":{"agent_id":"codex-cli","capability":capability,"deadline":int(time.time()*1000)+40000,**params}}
    buffer.write(json.dumps(request,ensure_ascii=False).encode()+b"\n");response=json.loads(buffer.readline());assert response.get("id")==request["id"] and "error" not in response,response;return response["result"]
try:
    coarse="--coarse" in sys.argv
    hello=call("gateway.hello","task.read",protocol_version={"major":1,"minor":12 if coarse else 11});cap=next(item for item in hello["capabilities"] if item["name"]=="computer.execute")
    if "--capability-only" in sys.argv:
        print(json.dumps({"protocol":hello["protocol_version"],"computer_capability":cap["availability"],"reason":cap.get("reason")},ensure_ascii=False));raise SystemExit(0)
    if "--cancel" in sys.argv:
        index=sys.argv.index("--cancel");task=call("task.cancel","task.cancel",task_id=sys.argv[index+1],expected_sequence=sys.argv[index+2])["task"];print(json.dumps({"task_id":task["task_id"],"status":task["status"]}));raise SystemExit(0)
    assert hello["protocol_version"]=={"major":1,"minor":12 if coarse else 11} and cap["availability"]=="available"
    created=call("task.create","task.create",idempotency_key=f"computer-gateway-{time.time_ns()}",description="验证本地Agent CUA完整生命周期",name="Computer Gateway 验证")["task"]
    key_bridge="--press-key" in sys.argv
    if coarse:
        executed=call("computer.step","computer.execute",task_id=created["task_id"],expected_sequence=created["sequence"],step_id="type-text",label="向当前工作窗口输入测试标记",tool_name="press_key" if key_bridge else "type_text",arguments={"key":"tab","delivery_mode":"background"} if key_bridge else {"text":"YONDER_SDK_INPUT_A","delivery_mode":"background"})
    else:
        declared=call("task.step.declare","task.step.declare",task_id=created["task_id"],expected_sequence=created["sequence"],step_id="type-text",label="向当前工作窗口输入测试标记")["task"]
        executed=call("computer.execute","computer.execute",task_id=created["task_id"],expected_sequence=declared["sequence"],tool_name="press_key" if key_bridge else "type_text",arguments={"key":"tab","delivery_mode":"background"} if key_bridge else {"text":"YONDER_SDK_INPUT_A","delivery_mode":"background"})
    if "--interrupt" in sys.argv:
        status=executed["status"] if coarse else executed["task"]["status"];unknown=executed["unknown_reason"] if coarse else executed["attempt_result"]["unknown_reason"]
        assert status=="interrupted" and unknown=="user-input",executed
        result={"protocol":hello["protocol_version"],"computer_capability":"available","task_id":created["task_id"],"final_status":"interrupted","unknown_reason":"user-input","final_sequence":executed["sequence"] if coarse else executed["task"]["sequence"],"recording_started":False,"passed":True}
        (output/"result.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n");print(json.dumps(result,ensure_ascii=False));raise SystemExit(0)
    assert (executed["status"]=="running" and executed["action_succeeded"] is True) if coarse else (executed["task"]["status"]=="running" and executed["attempt_result"]["phase"]=="observed" and executed["attempt_result"]["action_succeeded"] is True),{"gateway":executed,"fixture":{"pid":state.get("pid"),"window_id":state.get("window_id"),"active":state.get("app_active"),"key":state.get("target_key")}}
    screenshot=pathlib.Path(executed["observation"]["screenshot_path"]) if coarse and executed.get("observation",{}).get("screenshot_path") else None
    if coarse:
        assert executed.get("observation") and (executed["observation"]["element_count"]>0 or screenshot),executed
        if screenshot: assert screenshot.is_file() and 0<screenshot.stat().st_size<=4*1024*1024 and executed["observation"]["screenshot_mime"] in ("image/png","image/jpeg","image/webp"),executed
    if not key_bridge:
        end=time.monotonic()+5
        while not state.get("matches") and time.monotonic()<end:time.sleep(.05)
        assert state.get("matches"),"原生目标未收到SDK输入"
    advanced={"sequence":executed["sequence"]} if coarse else call("task.step.advance","task.step.advance",task_id=created["task_id"],expected_sequence=executed["task"]["sequence"])["task"]
    worker_pid=None
    if "--continuous" in sys.argv:
        pids=lambda:subprocess.run(["pgrep","-f","Resources/cua/node.*cua_worker.mjs"],capture_output=True,text=True).stdout.split()
        first_pids=pids();assert len(first_pids)==1,first_pids;worker_pid=first_pids[0]
        if coarse: executed2=call("computer.step","computer.execute",task_id=created["task_id"],expected_sequence=advanced["sequence"],step_id="press-key",label="在同一CUA会话执行第二个动作",tool_name="press_key",arguments={"key":"tab","delivery_mode":"background"})
        else:
            declared2=call("task.step.declare","task.step.declare",task_id=created["task_id"],expected_sequence=advanced["sequence"],step_id="press-key",label="在同一CUA会话执行第二个动作")["task"]
            executed2=call("computer.execute","computer.execute",task_id=created["task_id"],expected_sequence=declared2["sequence"],tool_name="press_key",arguments={"key":"tab","delivery_mode":"background"})
        assert executed2["action_succeeded"] is True if coarse else executed2["attempt_result"]["phase"]=="observed" and executed2["attempt_result"]["action_succeeded"] is True,executed2
        assert pids()==[worker_pid],{"before":first_pids,"after":pids()}
        advanced={"sequence":executed2["sequence"]} if coarse else call("task.step.advance","task.step.advance",task_id=created["task_id"],expected_sequence=executed2["task"]["sequence"])["task"]
    completed=call("task.complete","task.complete",task_id=created["task_id"],expected_sequence=advanced["sequence"])["task"];assert completed["status"]=="completed"
    worker_exited=None
    if worker_pid:
        end=time.monotonic()+5
        while worker_pid in pids() and time.monotonic()<end:time.sleep(.05)
        worker_exited=worker_pid not in pids();assert worker_exited,"任务完成后CUA Worker仍在运行"
    if screenshot: assert not screenshot.exists(),"任务完成后Observe截图仍存在"
    events=call("task.events","task.read",task_id=created["task_id"],after_sequence="0",limit=100)["events"]
    result={"protocol":hello["protocol_version"],"computer_capability":"available","sdk_tool":"press_key" if key_bridge else "type_text","task_id":created["task_id"],"native_target_matches":True,"attempt_phase":"observed" if coarse else executed["attempt_result"]["phase"],"one_call_step":coarse,"observe_element_count":executed.get("observation",{}).get("element_count") if coarse else None,"observe_screenshot_created_and_cleaned":bool(screenshot) if coarse else None,"worker_pid_reused":worker_pid,"worker_exited_after_completion":worker_exited,"final_status":completed["status"],"final_sequence":completed["sequence"],"events":len(events),"recording_started":False,"passed":True}
    (output/"result.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n");print(json.dumps(result,ensure_ascii=False))
finally:
    buffer.close();stream.close();fixture.terminate();fixture.wait(timeout=5)
