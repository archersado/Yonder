#!/usr/bin/env python3
"""通过正式桌面私有 stdio Agent 验证步骤声明、兼容读取和数据保留。"""
import json, pathlib, select, subprocess, sys, time

root = pathlib.Path(__file__).resolve().parents[2]
output = pathlib.Path(sys.argv[1]); output.mkdir(parents=True, exist_ok=False)
binary = root / "apps/desktop/target/preview/Yonda Task Space.app/Contents/MacOS/yonder-desktop"
agent = subprocess.Popen([str(binary), "--local-agent-stdio"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=(output/"host.log").open("wb"), start_new_session=True)
counter = 0

def call(method, capability, **params):
    global counter
    counter += 1
    request = {"jsonrpc":"2.0","id":f"step-{counter}","method":method,"params":{"agent_id":"local-test-agent","capability":capability,"deadline":int(time.time()*1000)+20000,**params}}
    agent.stdin.write(json.dumps(request,ensure_ascii=False).encode()+b"\n"); agent.stdin.flush()
    assert select.select([agent.stdout],[],[],20)[0]
    return json.loads(agent.stdout.readline())

try:
    hello = call("gateway.hello","task.read",protocol_version={"major":1,"minor":4})["result"]
    created = call("task.create","task.create",idempotency_key=f"agent-step-{time.time_ns()}",description="验证Agent步骤声明",name="Agent步骤声明验证")["result"]["task"]
    declared = call("task.step.declare","task.step.declare",task_id=created["task_id"],expected_sequence=created["sequence"],step_id="open-document",label="打开目标文档")["result"]
    current = call("task.step.get","task.read",task_id=created["task_id"])["result"]
    events14 = call("task.events","task.read",task_id=created["task_id"],after_sequence="0",limit=100)["result"]["events"]
    call("gateway.hello","task.read",protocol_version={"major":1,"minor":3})
    events13 = call("task.events","task.read",task_id=created["task_id"],after_sequence="0",limit=100)["result"]["events"]
    call("gateway.hello","task.read",protocol_version={"major":1,"minor":4})
    cancelled = call("task.cancel","task.cancel",task_id=created["task_id"],expected_sequence=declared["task"]["sequence"])["result"]["task"]
    retry = call("task.step.declare","task.step.declare",task_id=created["task_id"],expected_sequence=created["sequence"],step_id="open-document",label="打开目标文档")["result"]
    result = {
        "pid":agent.pid,"protocol":hello["protocol_version"],"task_id":created["task_id"],
        "declared_sequence":declared["step"]["accepted_sequence"],"current_matches":current["step"]==declared["step"],
        "events_14_have_step":any("step_declaration" in event for event in events14),
        "events_13_hide_step":all("step_declaration" not in event for event in events13),
        "task_retained_as":cancelled["status"],"retry_after_cancel":retry["task"]["status"],
    }
    result["passed"] = all([result["protocol"]=={"major":1,"minor":4},result["current_matches"],result["events_14_have_step"],result["events_13_hide_step"],result["task_retained_as"]=="cancelled",result["retry_after_cancel"]=="cancelled"])
    (output/"result.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n")
    print(json.dumps(result,ensure_ascii=False)); raise SystemExit(0 if result["passed"] else 1)
finally:
    if agent.stdin: agent.stdin.close()
    if agent.stdout: agent.stdout.close()
