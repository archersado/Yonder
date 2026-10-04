#!/usr/bin/env python3
"""隔离验证Sky应用级AX transcript、任务绑定与封闭语义动作。"""
import json
import os
import pathlib
import shutil
import subprocess
import tempfile


root = pathlib.Path(__file__).resolve().parents[2]
worker = root / "crates/adapters/src/sky_cua_worker.mjs"

with tempfile.TemporaryDirectory(prefix="yonder-sky-worker-") as directory:
    fixture = pathlib.Path(directory)
    bridge = fixture / "SkyComputerUseClient"
    log = fixture / "actions.jsonl"
    evidence = fixture / "evidence"
    evidence.mkdir()
    bridge.write_text(
        """#!/usr/bin/env python3
import json, os, sys
text = '1 文本框 搜索\\n2 按钮 播放'
calculator_text = '1 按钮 1\\n2 文本 0'
approved = False
log = os.environ['YONDER_SKY_TEST_LOG']
for line in sys.stdin:
    request = json.loads(line)
    if 'id' not in request:
        continue
    if request['method'] == 'initialize':
        print(json.dumps({'jsonrpc':'2.0','id':'rogue-approval','method':'elicitation/create','params':{'message':'范围外请求','requestedSchema':{'type':'object','properties':{}}}}, ensure_ascii=False), flush=True)
        rogue = json.loads(sys.stdin.readline())
        with open(log, 'a', encoding='utf-8') as output:
            output.write(json.dumps({'rogue_approval':rogue.get('result', {}).get('action') == 'accept'}, ensure_ascii=False)+'\\n')
        result = {'protocolVersion':'2025-06-18','capabilities':{},'serverInfo':{'name':'fixture','version':'1'}}
    else:
        name = request['params']['name']
        args = request['params']['arguments']
        if name == 'list_apps':
            value = [
                {'id':'com.tencent.QQMusicMac','displayName':'QQ音乐','isRunning':False},
                {'id':'com.apple.calculator','displayName':'计算器','isRunning':False},
            ]
            result = {'content':[{'type':'text','text':json.dumps(value, ensure_ascii=False)}]}
        elif name == 'get_app_state':
            if not approved:
                print(json.dumps({'jsonrpc':'2.0','id':'approval-1','method':'elicitation/create','params':{'message':'本地化提示不作为应用身份','requestedSchema':{'type':'object','properties':{}},'_meta':{'persist':['always']}}}, ensure_ascii=False), flush=True)
                approval = json.loads(sys.stdin.readline())
                approved = approval.get('result', {}).get('action') == 'accept'
                with open(log, 'a', encoding='utf-8') as output:
                    output.write(json.dumps({'approval':approved}, ensure_ascii=False)+'\\n')
            state = calculator_text if args['app'] == 'com.apple.calculator' else text
            result = {'content':[{'type':'text','text':state}]} if approved else {'isError':True,'content':[{'type':'text','text':'denied'}]}
        else:
            with open(log, 'a', encoding='utf-8') as output:
                output.write(json.dumps({'name':name,'args':args}, ensure_ascii=False)+'\\n')
            if name == 'set_value':
                text = f\"1 文本框 搜索 {args['value']}\\n2 按钮 播放\"
            elif name == 'press_key':
                text += '\\n3 文本 搜索结果'
            elif name == 'type_text':
                calculator_text = f\"1 按钮 1\\n2 文本 {args['text']}\"
            result = {'content':[]}
    print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':result}, ensure_ascii=False), flush=True)
""",
        encoding="utf-8",
    )
    bridge.chmod(0o700)
    node = shutil.which("node")
    assert node is not None
    process = subprocess.Popen(
        [node, str(worker), str(bridge), str(evidence)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=dict(os.environ, YONDER_SKY_TEST_LOG=str(log)),
    )

    def request(step, tool, arguments):
        payload = {
            "task_id": "task-sky",
            "step_id": step,
            "attempt_id": f"attempt-{step}",
            "worker_instance_id": "worker-sky",
            "host_session_id": "host-sky",
            "pid": 1,
            "window_id": 1,
            "tool_name": tool,
            "arguments": arguments,
        }
        process.stdin.write(json.dumps(payload, ensure_ascii=False) + "\n")
        process.stdin.flush()
        return json.loads(process.stdout.readline())

    launched = request("launch", "launch_app", {"bundle_id": "com.tencent.QQMusicMac"})
    focused = request("focus", "hotkey", {"keys": ["cmd", "f"], "_yonder_action_kind": "focus-control"})
    entered = request("input", "type_text", {"text": "one last kiss", "_yonder_action_kind": "input-text"})
    activated = request("activate", "press_key", {"key": "ENTER", "_yonder_action_kind": "activate-control"})
    calculator_launched = request("calculator-launch", "launch_app", {"bundle_id": "com.apple.calculator"})
    calculator_input = request("calculator-input", "type_text", {"text": "1+1", "_yonder_action_kind": "input-text"})
    process.terminate()
    process.wait(timeout=5)

    records = [json.loads(line) for line in log.read_text(encoding="utf-8").splitlines()]
    assert records[0] == {"rogue_approval": False}
    assert records[1] == {"approval": True}
    actions = records[2:]
    assert all(result["action_succeeded"] and result["observe_valid"] for result in (launched, focused, entered, activated, calculator_launched, calculator_input))
    assert launched["element_count"] == 2 and activated["element_count"] == 3
    assert [item["name"] for item in actions] == ["click", "set_value", "press_key", "type_text"]
    assert actions[0]["args"]["element_index"] == "1"
    assert actions[1]["args"]["element_index"] == "1"
    assert actions[2]["args"]["key"] == "Return"
    assert actions[3]["args"]["text"] == "1+1"
    assert all("_yonder_action_kind" not in item["args"] and "_yonder_private_text" not in item["args"] for item in actions)
    print(json.dumps({
        "driver": "sky",
        "transcript_elements": activated["element_count"],
        "actions": [item["name"] for item in actions],
        "out_of_scope_approval_rejected": True,
        "task_binding_reused": True,
        "passed": True,
    }, ensure_ascii=False))
