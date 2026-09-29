#!/usr/bin/env python3
"""正式 GUI、Local Socket 与真实 trycua 的步骤成功/待核实投影验证。"""
import json,pathlib,socket,sqlite3,subprocess,sys,tempfile,time

root=pathlib.Path(__file__).resolve().parents[2]
out=pathlib.Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=False)
temporary=tempfile.TemporaryDirectory(prefix="yonder-step-status-")
watcher=pathlib.Path(temporary.name)/"watch-control"
fixture_bin=pathlib.Path(temporary.name)/"input-fixture"
subprocess.run(["swiftc",str(root/"apps/desktop/check-cua-control-macos.swift"),"-o",str(watcher)],check=True,timeout=45)
subprocess.run(["swiftc",str(root/"apps/desktop/tests/input-fixture-macos.swift"),"-o",str(fixture_bin)],check=True,timeout=45)
fixture=subprocess.Popen([str(fixture_bin)],stdout=subprocess.PIPE,text=True)
stream=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM);stream.settimeout(60)
stream.connect(str(pathlib.Path.home()/"Library/Application Support/com.yonder.desktop/agent.sock"));f=stream.makefile("rwb",buffering=0);counter=0
task=None
def call(method,capability,**params):
 global counter;counter+=1
 request={"jsonrpc":"2.0","id":f"step-status-{counter}","method":method,"params":{"agent_id":"codex-cli","capability":capability,"deadline":int(time.time()*1000)+50000,**params}}
 f.write(json.dumps(request,ensure_ascii=False).encode()+b"\n");response=json.loads(f.readline());assert "error" not in response,response;return response["result"]
try:
 deadline=time.monotonic()+10;ready={}
 while time.monotonic()<deadline:
  ready=json.loads(fixture.stdout.readline())
  if ready.get("ready") and ready.get("app_active"):break
 assert ready.get("ready") and ready.get("app_active"),ready
 hello=call("gateway.hello","task.read",protocol_version={"major":1,"minor":31});caps={item["name"]:item["availability"] for item in hello["capabilities"]};assert caps.get("task.plan.execute")=="available",caps
 db_path=pathlib.Path.home()/"Library/Application Support/com.yonder.desktop/tasks.db"
 with sqlite3.connect(db_path) as db:stale=db.execute("SELECT id,sequence FROM tasks WHERE name='CUA 步骤状态原生验证' AND state IN ('created','running')").fetchall()
 for task_id,sequence in stale:call("task.cancel","task.cancel",task_id=task_id,expected_sequence=str(sequence))
 task=call("task.create","task.create",idempotency_key=f"step-status-{time.time_ns()}",description="验证真实 CUA 成功边界与顶部步骤图标",name="CUA 步骤状态原生验证")["task"]
 candidate=lambda index:{"candidate_id":f"type-{index}","tool_name":"type_text","arguments":{"text":"YONDER_SDK_INPUT_A","delivery_mode":"background"},"action_kind":"draft-message","target_ref":"fixture-field","preconditions":[{"fact":"composer-ready","expected":True}],"expected_observe":[{"fact":"composer-ready","expected":True}]}
 slots=[{"step_id":f"type-{index}","label":f"连续 CUA 步骤 {index}","candidates":[candidate(index)]} for index in (1,2)]
 submitted=call("task.plan.submit","task.plan.submit",task_id=task["task_id"],expected_sequence=task["sequence"],plan_id="step-status-plan",plan_version=1,token_budget=100,slots=slots)
 executed=call("task.plan.execute","task.plan.execute",task_id=task["task_id"],expected_sequence=submitted["sequence"],plan_id="step-status-plan",plan_version=1)
 with sqlite3.connect(db_path) as db:
  row=db.execute("SELECT current_slot FROM task_plan_fragments WHERE task_id=? AND plan_id='step-status-plan'",(task["task_id"],)).fetchone()
  conclusions=[{"phase":phase,"action_succeeded":bool(success) if success is not None else None,"unknown_reason":reason} for phase,success,reason in db.execute("SELECT phase,action_succeeded,unknown_reason FROM task_attempts WHERE task_id=? ORDER BY accepted_sequence",(task["task_id"],)).fetchall()]
 first_confirmed=bool(conclusions and conclusions[0]["phase"]=="stopped" and conclusions[0]["action_succeeded"] is True)
 expected_status=("已完成：" if first_confirmed else "待核实：")+"连续 CUA 步骤 1"
 native=subprocess.run([str(watcher),str(out/"native"),"慢脑单候选直接授权：填写消息草稿","observe-only",expected_status],capture_output=True,text=True,timeout=25)
 assert native.returncode==0,(native.returncode,native.stdout,native.stderr)
 current=call("task.get","task.read",task_id=task["task_id"])["task"]
 cancelled=call("task.cancel","task.cancel",task_id=task["task_id"],expected_sequence=current["sequence"])["task"]
 window=json.loads((out/"native/window-result.json").read_text())
 result={"platform":"macos","real_driver":True,"first_step_committed":bool(row and row[0]>=1),"recorded_conclusions":conclusions,"projected_status":"completed" if first_confirmed else "unverified","status_icon_accessible":window["step_status_found"],"executed_disposition":executed.get("disposition"),"task_status":cancelled["status"]}
 result["passed"]=bool(conclusions) and result["status_icon_accessible"] and result["task_status"]=="cancelled" and (result["first_step_committed"] if first_confirmed else conclusions[0]["action_succeeded"] is not True)
 (out/"result.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n");print(json.dumps(result,ensure_ascii=False));assert result["passed"],result
finally:
 if task is not None:
  try:
   current=call("task.get","task.read",task_id=task["task_id"])["task"]
   if current["status"] in ("created","running"):call("task.cancel","task.cancel",task_id=task["task_id"],expected_sequence=current["sequence"])
  except Exception:pass
 fixture.terminate()
 try:fixture.wait(timeout=3)
 except subprocess.TimeoutExpired:fixture.kill();fixture.wait()
 f.close();stream.close()
 temporary.cleanup()
