#!/usr/bin/env python3
"""真实UDS CUA步骤后从Yonda卡片接管并验证定位事实。"""
import json,pathlib,socket,sqlite3,subprocess,sys,threading,time
root=pathlib.Path(__file__).resolve().parents[2];out=pathlib.Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=False)
fixture=subprocess.Popen(["/private/tmp/yonda-cu-gateway-fixture"],stdout=subprocess.PIPE,text=True);state={}
def read():
 for line in fixture.stdout:
  try:state.update(json.loads(line))
  except json.JSONDecodeError:pass
threading.Thread(target=read,daemon=True).start();end=time.monotonic()+12
while not(state.get("ready") and state.get("launched") and state.get("app_active") and state.get("target_key")) and time.monotonic()<end:time.sleep(.05)
assert state.get("ready") and state.get("app_active") and state.get("target_key"),state
focused=subprocess.run(["/private/tmp/yonda-focus-target",str(state["pid"]),str(state["window_id"])],capture_output=True,text=True,timeout=10)
assert json.loads(focused.stdout).get("focused"),focused.stdout
stream=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM);stream.settimeout(45);stream.connect(str(pathlib.Path.home()/"Library/Application Support/com.yonder.desktop/agent.sock"));f=stream.makefile("rwb",buffering=0);count=0
def call(method,capability,**params):
 global count;count+=1;request={"jsonrpc":"2.0","id":f"focus-{count}","method":method,"params":{"agent_id":"codex-cli","capability":capability,"deadline":int(time.time()*1000)+40000,**params}};f.write(json.dumps(request,ensure_ascii=False).encode()+b"\n");response=json.loads(f.readline());assert "error" not in response,response;return response["result"]
try:
 hello=call("gateway.hello","task.read",protocol_version={"major":1,"minor":13});assert hello["protocol_version"]=={"major":1,"minor":13},hello
 name="接管定位原生验证";created=call("task.create","task.create",idempotency_key=f"takeover-focus-{time.time_ns()}",description="验证CUA步骤停止后的精确工作定位",name=name)["task"]
 step=call("computer.step","computer.execute",task_id=created["task_id"],expected_sequence=created["sequence"],step_id="type",label="输入原生测试标记",tool_name="type_text",arguments={"text":"YONDER_TAKEOVER_FOCUS","delivery_mode":"background"})
 assert step["status"]=="running" and step["action_succeeded"] is True,step
 print(json.dumps({"ready":True,"task_id":created["task_id"],"task_name":name},ensure_ascii=False),flush=True)
 input()
 end=time.monotonic()+5
 while not state.get("app_active") and time.monotonic()<end:time.sleep(.05)
 db=sqlite3.connect(pathlib.Path.home()/"Library/Application Support/com.yonder.desktop/tasks.db")
 task=db.execute("SELECT state,sequence FROM tasks WHERE id=?",(created["task_id"],)).fetchone();control=db.execute("SELECT phase,focus_phase,focus_failure FROM task_controls WHERE task_id=?",(created["task_id"],)).fetchone();events=db.execute("SELECT count(*) FROM events WHERE task_id=?",(created["task_id"],)).fetchone()[0];db.close()
 assert task[0]=="paused" and control==("stopped","focused",None),(task,control,state)
 result={"protocol":hello["protocol_version"],"task_id":created["task_id"],"task_status":task[0],"control_phase":control[0],"focus_phase":control[1],"native_target_frontmost":bool(state.get("app_active")),"events":events,"recording_started":False,"passed":True};(out/"result.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n");print(json.dumps(result,ensure_ascii=False))
finally:
 f.close();stream.close();fixture.terminate();fixture.wait(timeout=5)
