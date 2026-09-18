#!/usr/bin/env python3
"""通过正式UDS Gateway验证本地Agent完整Browser任务。"""
import json, pathlib, socket, sys, time

output = pathlib.Path(sys.argv[1]); output.mkdir(parents=True, exist_ok=False)
path = pathlib.Path.home() / "Library/Application Support/com.yonder.desktop/agent.sock"
stream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM); stream.settimeout(45); stream.connect(str(path))
buffer = stream.makefile("rwb", buffering=0); counter = 0

def call(method, capability, **params):
    global counter
    counter += 1
    request = {"jsonrpc":"2.0","id":f"browser-{counter}","method":method,"params":{"agent_id":"codex-cli","capability":capability,"deadline":int(time.time()*1000)+40000,**params}}
    buffer.write(json.dumps(request, ensure_ascii=False).encode()+b"\n")
    response = json.loads(buffer.readline())
    assert response.get("id") == request["id"] and "error" not in response, response
    return response["result"]

try:
    hello = call("gateway.hello","task.read",protocol_version={"major":1,"minor":8})
    browser_cap = next(item for item in hello["capabilities"] if item["name"] == "browser.execute")
    assert hello["protocol_version"] == {"major":1,"minor":8} and browser_cap["availability"] == "available"
    created = call("task.create","task.create",idempotency_key=f"browser-gateway-{time.time_ns()}",description="验证本地Agent Browser Gateway完整生命周期",name="Browser Gateway 验证")["task"]
    declared = call("task.step.declare","task.step.declare",task_id=created["task_id"],expected_sequence=created["sequence"],step_id="create-space",label="创建Browser Task Space")["task"]
    opened = call("browser.execute","browser.execute",task_id=created["task_id"],expected_sequence=declared["sequence"],operation="create")
    assert opened["task"]["status"] == "running" and opened["reference"]["external_task_ref"].startswith("ego:") and not opened["reference"]["finished"]
    advanced = call("task.step.advance","task.step.advance",task_id=created["task_id"],expected_sequence=opened["task"]["sequence"])["task"]
    closing = call("task.step.declare","task.step.declare",task_id=created["task_id"],expected_sequence=advanced["sequence"],step_id="finish-space",label="完成Browser Task Space")["task"]
    finished = call("browser.execute","browser.execute",task_id=created["task_id"],expected_sequence=closing["sequence"],operation="finish")
    assert finished["task"]["status"] == "completed" and finished["reference"]["finished"] and finished["reference"]["external_task_ref"] == opened["reference"]["external_task_ref"]
    events = call("task.events","task.read",task_id=created["task_id"],after_sequence="0",limit=100)["events"]
    result = {"protocol":hello["protocol_version"],"browser_capability":"available","task_id":created["task_id"],"external_task_ref":opened["reference"]["external_task_ref"],"final_status":finished["task"]["status"],"final_sequence":finished["task"]["sequence"],"events":len(events),"observed_attempts":sum(1 for event in events if event.get("attempt_result",{}).get("phase")=="observed"),"passed":True}
    (output/"result.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n")
    print(json.dumps(result,ensure_ascii=False))
finally:
    buffer.close(); stream.close()
