#!/usr/bin/env python3
"""无副作用 macOS Gateway 片段提交样本；证据不含候选参数正文。"""
import json, os, pathlib, select, subprocess, sys, tempfile, time

root = pathlib.Path(__file__).resolve().parents[2]
binary = root / "target/debug/Yonda.app/Contents/MacOS/yonder-desktop"
home = tempfile.TemporaryDirectory(dir="/tmp", prefix="ex2-")
environment = dict(os.environ, HOME=home.name)
agent = subprocess.Popen([str(binary), "--local-agent-stdio"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, env=environment)

def call(number, method, capability, **params):
    request = {"jsonrpc":"2.0","id":f"plan-{number}","method":method,"params":{"agent_id":"local-test-agent","capability":capability,"deadline":int(time.time()*1000)+20_000,**params}}
    agent.stdin.write(json.dumps(request).encode()+b"\n"); agent.stdin.flush()
    if not select.select([agent.stdout], [], [], 20)[0]: raise RuntimeError("gateway-timeout")
    response = json.loads(agent.stdout.readline())
    if "error" in response: raise RuntimeError(json.dumps(response["error"], ensure_ascii=False))
    return response

try:
    hello=call(1,"gateway.hello","task.read",protocol_version={"major":1,"minor":31})
    task=call(2,"task.create","task.create",idempotency_key=f"plan-{time.time_ns()}",description="EX-S2计划片段验证",name="EX-S2验证")["result"]["task"]
    submitted=call(3,"task.plan.submit","task.plan.submit",task_id=task["task_id"],expected_sequence=task["sequence"],plan_id="verify-plan",plan_version=1,token_budget=1,slots=[{"step_id":"verify-step","label":"隔离验证步骤","candidates":[{"candidate_id":"safe","tool_name":"type_text","arguments":{"delivery_mode":"background"},"action_kind":"draft-message","target_ref":"isolated-fixture-composer","preconditions":[{"fact":"composer-ready","expected":True}],"expected_observe":[{"fact":"composer-ready","expected":True}]}]}])
    capabilities={entry.get("name") for entry in hello.get("result",{}).get("capabilities",[])}
    result={
        "hello_ok":hello.get("result",{}).get("protocol_version",{}).get("minor")==31,
        "plan_capabilities": {"task.plan.submit","task.plan.execute"}.issubset(capabilities),
        "accepted":submitted.get("result",{}).get("disposition")=="accepted",
        "sequence_advanced":submitted.get("result",{}).get("sequence")!=task["sequence"],
    }
    result["passed"]=all(result.values())
    print(json.dumps(result)); sys.exit(0 if result["passed"] else 1)
finally:
    agent.terminate(); agent.wait(timeout=5)
    home.cleanup()
