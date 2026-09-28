#!/usr/bin/env python3
"""动作响应已返回、没有后续Gateway帧时，顶部控制卡仍须完成显式接管。"""
import json,pathlib,socket,sqlite3,subprocess,sys,time

root=pathlib.Path(__file__).resolve().parents[2]
out=pathlib.Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=False)
watcher=out/"watch-control"
subprocess.run(["swiftc",str(root/"apps/desktop/check-cua-control-macos.swift"),"-o",str(watcher)],check=True,timeout=45)
stream=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM);stream.settimeout(60)
stream.connect(str(pathlib.Path.home()/"Library/Application Support/com.yonder.desktop/agent.sock"));f=stream.makefile("rwb",buffering=0);counter=0

def call(method,capability,**params):
 global counter;counter+=1
 request={"jsonrpc":"2.0","id":f"idle-takeover-{counter}","method":method,"params":{"agent_id":"codex-cli","capability":capability,"deadline":int(time.time()*1000)+50000,**params}}
 f.write(json.dumps(request,ensure_ascii=False).encode()+b"\n");response=json.loads(f.readline());assert "error" not in response,response;return response["result"]

native=None
try:
 hello=call("gateway.hello","task.read",protocol_version={"major":1,"minor":31});caps={item["name"]:item["availability"] for item in hello["capabilities"]};assert caps.get("computer.execute")=="available",caps
 task=call("task.create","task.create",idempotency_key=f"idle-takeover-{time.time_ns()}",description="验证CUA动作响应后无后续Gateway请求仍可接管",name="CUA 步骤间接管验证")["task"]
 executed=call("computer.step","computer.execute",task_id=task["task_id"],expected_sequence=task["sequence"],step_id="idle-boundary",label="连续 CUA 步骤 1",tool_name="press_key",arguments={"key":"tab","delivery_mode":"background"})
 # 关键回归：动作响应已完整返回后才启动原生观察器；此后不发送任何Gateway帧来“唤醒”接管。
 native=subprocess.Popen([str(watcher),str(out/"native")],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
 stdout,stderr=native.communicate(timeout=25);assert native.returncode==0,(native.returncode,stdout,stderr)
 deadline=time.monotonic()+8;row=None;control=None
 db_path=pathlib.Path.home()/"Library/Application Support/com.yonder.desktop/tasks.db"
 while time.monotonic()<deadline:
  with sqlite3.connect(db_path) as db:
   row=db.execute("SELECT state,sequence FROM tasks WHERE id=?",(task["task_id"],)).fetchone()
   control=db.execute("SELECT phase,focus_phase FROM task_controls WHERE task_id=?",(task["task_id"],)).fetchone()
  if row and row[0]=="paused" and control and control[0]=="stopped":break
  time.sleep(.05)
 window=json.loads((out/"native/window-result.json").read_text())
 result={"platform":"macos","task_id":task["task_id"],"action_response_status":executed["status"],"takeover_pressed_after_response":window["takeover_pressed"],"task_status":row[0] if row else None,"control_phase":control[0] if control else None,"focus_phase":control[1] if control else None,"completed_without_followup_gateway_frame":row is not None and row[0]=="paused"}
 result["passed"]=result["action_response_status"]=="running" and result["takeover_pressed_after_response"] and result["task_status"]=="paused" and result["control_phase"]=="stopped" and result["completed_without_followup_gateway_frame"]
 (out/"result.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n");print(json.dumps(result,ensure_ascii=False));assert result["passed"],result
finally:
 if native is not None and native.poll() is None:native.kill();native.wait()
 f.close();stream.close()
