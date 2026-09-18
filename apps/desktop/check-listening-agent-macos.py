#!/usr/bin/env python3
"""用正式私有 stdio Agent 创建并取消一项任务，截取收到请求动画。"""
import json, pathlib, select, subprocess, sys, time

root = pathlib.Path(__file__).resolve().parents[2]
output = pathlib.Path(sys.argv[1]); output.mkdir(parents=True, exist_ok=False)
binary = root / "apps/desktop/target/preview/Yonda Task Space.app/Contents/MacOS/yonder-desktop"
capture = output / "capture-pet"
subprocess.run(["swiftc", str(root / "apps/desktop/check-state-assets-macos.swift"), "-o", str(capture)], check=True, timeout=45)
agent = subprocess.Popen([str(binary), "--local-agent-stdio"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=(output / "host.log").open("wb"), start_new_session=True)
counter = 0

def call(method, capability, **params):
    global counter
    counter += 1
    request = {"jsonrpc":"2.0","id":f"listen-{counter}","method":method,"params":{"agent_id":"local-test-agent","capability":capability,"deadline":int(time.time()*1000)+20000,**params}}
    agent.stdin.write(json.dumps(request).encode()+b"\n"); agent.stdin.flush()
    assert select.select([agent.stdout], [], [], 20)[0]
    return json.loads(agent.stdout.readline())

try:
    assert "result" in call("gateway.hello", "task.read", protocol_version={"major":1,"minor":3})
    created = call("task.create", "task.create", idempotency_key=f"listening-{time.time_ns()}", description="验证真实收到请求动画", name="收到请求动画验证")["result"]["task"]
    native = subprocess.run([str(capture), str(output / "native"), "--immediate"], timeout=45)
    assert native.returncode == 0
    cancelled = call("task.cancel", "task.cancel", task_id=created["task_id"], expected_sequence=created["sequence"])["result"]["task"]
    result = {"pid":agent.pid,"real_agent_create":True,"task_id":created["task_id"],"created_state":created["status"],"cancelled_state":cancelled["status"],"passed":cancelled["status"]=="cancelled"}
    (output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2)+"\n")
    print(json.dumps(result, ensure_ascii=False))
finally:
    if agent.stdin: agent.stdin.close()
    if agent.stdout: agent.stdout.close()
