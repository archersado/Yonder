#!/usr/bin/env python3
"""真实 Agent 保持连接，等待三分钟隐藏后创建任务并验证主动恢复。"""
import json, pathlib, select, subprocess, sys, time

root = pathlib.Path(__file__).resolve().parents[2]
output = pathlib.Path(sys.argv[1]); output.mkdir(parents=True, exist_ok=False)
probe = output / "wait-pet-geometry"
subprocess.run(["swiftc", str(root / "apps/desktop/wait-pet-geometry-macos.swift"), "-o", str(probe)], check=True, timeout=45)
binary = root / "apps/desktop/target/preview/Yonda Task Space.app/Contents/MacOS/yonder-desktop"
agent = subprocess.Popen([str(binary), "--local-agent-stdio"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=(output/"host.log").open("wb"), start_new_session=True)
counter = 0

def call(method, capability, **params):
    global counter
    counter += 1
    frame = {"jsonrpc":"2.0","id":f"hidden-{counter}","method":method,"params":{"agent_id":"local-test-agent","capability":capability,"deadline":int(time.time()*1000)+20000,**params}}
    agent.stdin.write(json.dumps(frame).encode()+b"\n"); agent.stdin.flush(); assert select.select([agent.stdout],[],[],20)[0]
    return json.loads(agent.stdout.readline())

try:
    assert "result" in call("gateway.hello","task.read",protocol_version={"major":1,"minor":3})
    subprocess.run([str(probe),str(agent.pid),"docked",str(output/"docked.json"),str(output/"docked.png")],check=True,timeout=195)
    created = call("task.create","task.create",idempotency_key=f"hidden-wake-{time.time_ns()}",description="验证隐藏态状态变化唤醒",name="隐藏态唤醒验证")["result"]["task"]
    subprocess.run([str(probe),str(agent.pid),"awake",str(output/"awake.json"),str(output/"awake.png")],check=True,timeout=15)
    cancelled = call("task.cancel","task.cancel",task_id=created["task_id"],expected_sequence=created["sequence"])["result"]["task"]
    result={"pid":agent.pid,"natural_dock":True,"real_agent_create":True,"woke_on_state_change":True,"task_retained_as":cancelled["status"],"passed":cancelled["status"]=="cancelled"}
    (output/"result.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n"); print(json.dumps(result,ensure_ascii=False))
finally:
    if agent.stdin: agent.stdin.close()
    if agent.stdout: agent.stdout.close()
