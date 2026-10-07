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
    state = fixture / "state.txt"
    state.write_text("", encoding="utf-8")
    evidence = fixture / "evidence"
    evidence.mkdir()
    bridge.write_text(
        """#!/usr/bin/env python3
import json, os, sys
state_path = os.environ['YONDER_SKY_TEST_STATE']
text = '1 文本框 搜索\\n2 按钮 播放\\n3 row (selectable)\\n\\t4 单元格\\n\\t\\t5 文本 狼顾科技' + open(state_path, encoding='utf-8').read()
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
    elif request['method'] == 'tools/list':
        result = {'tools':[{'name':'get_app_state','inputSchema':{'properties':{'app':{},'disable_diff':{}}}}]}
    else:
        name = request['params']['name']
        args = request['params']['arguments']
        if name == 'list_apps':
            value = [
                {'id':'com.yonder.fixture.music','displayName':'QQ音乐','path':'/Applications/QQMusic.app','isRunning':False},
                {'id':'com.yonder.fixture.music','displayName':'QQ音乐','path':'/Volumes/QQMusic/QQMusic.app','isRunning':False},
                {'id':'com.yonder.fixture.calculator','displayName':'计算器','path':'/System/Applications/Calculator.app','isRunning':False},
                {'id':'com.yonder.fixture.no-window','displayName':'无窗口夹具','path':'/Applications/NoWindow.app','isRunning':False},
            ]
            result = {'content':[{'type':'text','text':json.dumps(value, ensure_ascii=False)}]}
        elif name == 'get_app_state':
            with open(log, 'a', encoding='utf-8') as output:
                output.write(json.dumps({'observe':{'app':args['app'],'disableDiff':args.get('disableDiff'),'disable_diff':args.get('disable_diff')}}, ensure_ascii=False)+'\\n')
            if not approved:
                print(json.dumps({'jsonrpc':'2.0','id':'approval-1','method':'elicitation/create','params':{'message':'本地化提示不作为应用身份','requestedSchema':{'type':'object','properties':{}},'_meta':{'persist':['always']}}}, ensure_ascii=False), flush=True)
                approval = json.loads(sys.stdin.readline())
                approved = approval.get('result', {}).get('action') == 'accept'
                with open(log, 'a', encoding='utf-8') as output:
                    output.write(json.dumps({'approval':approved}, ensure_ascii=False)+'\\n')
            state = calculator_text if args['app'].endswith('Calculator.app') else text
            tree_state = '\\n'.join('├─ ' + item for item in state.split('\\n'))
            wrapped = 'App state follows:\\n```json\\n' + json.dumps({'app':args['app'],'text':tree_state}, ensure_ascii=False) + '\\n```' if args['app'].endswith('QQMusic.app') else state
            result = {'isError':True,'content':[{'type':'text','text':'Computer Use server error -10005: cgWindowNotFound'}]} if args['app'].endswith('NoWindow.app') else ({'content':[{'type':'text','text':wrapped}]} if approved else {'isError':True,'content':[{'type':'text','text':'denied'}]})
        else:
            with open(log, 'a', encoding='utf-8') as output:
                output.write(json.dumps({'name':name,'args':args}, ensure_ascii=False)+'\\n')
            if name == 'set_value':
                text = f\"1 文本框 搜索 {args['value']}\\n2 按钮 播放\"
            elif name == 'type_text' and args['app'].endswith('QQMusic.app'):
                text = f\"1 文本框 搜索 {args['text']}\\n2 按钮 播放\\n3 选项 one\\n4 选项 last\\n5 选项 kiss 宇多田光\"
            elif name == 'click' and args.get('element_index') == '3':
                text += '\\n7 文本 已打开搜索结果'
            elif name == 'click' and args.get('element_index') == '2':
                text += '\\n10 文本 企业菜单已打开'
            elif name == 'click' and args.get('click_count') == 2:
                text += '\\n8 文本 已双击激活自绘结果'
            elif name == 'click' and args.get('x') == 30 and args.get('y') == 40:
                text += '\\n9 文本 企业菜单已打开'
                with open(state_path, 'a', encoding='utf-8') as state_output:
                    state_output.write('\\n9 文本 企业菜单已打开')
                result = {'isError':True,'content':[{'type':'text','text':'request timed out'}]}
                print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':result}, ensure_ascii=False), flush=True)
                continue
            elif name == 'click' and args.get('x') == 31 and args.get('y') == 41:
                result = {'isError':True,'content':[{'type':'text','text':'request timed out'}]}
                print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':result}, ensure_ascii=False), flush=True)
                continue
            elif name == 'click' and args.get('x') == 32 and args.get('y') == 42:
                text += '\\n11 文本 窗口代次已变化'
                result = {'isError':True,'content':[{'type':'text','text':'Computer Use server error -10005: cgWindowNotFound'}]}
                print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':result}, ensure_ascii=False), flush=True)
                continue
            elif name == 'click' and args.get('x') == 33 and args.get('y') == 43:
                result = {'isError':True,'content':[{'type':'text','text':'click failed after dispatch'}]}
                print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':result}, ensure_ascii=False), flush=True)
                continue
            elif name == 'press_key':
                text += '\\n6 文本 搜索结果已提交'
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
    def start_worker():
        return subprocess.Popen(
            [node, str(worker), str(bridge), str(evidence)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env=dict(os.environ, YONDER_SKY_TEST_LOG=str(log), YONDER_SKY_TEST_STATE=str(state)),
        )

    process = start_worker()

    binding = None

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
        if tool != "launch_app" and binding is not None:
            payload["bound_application_id"] = binding
        process.stdin.write(json.dumps(payload, ensure_ascii=False) + "\n")
        process.stdin.flush()
        return json.loads(process.stdout.readline())

    launched = request("launch", "launch_app", {"bundle_id": "com.yonder.fixture.music"})
    binding = launched["launched_app_id"]
    process.terminate()
    process.wait(timeout=5)
    process = start_worker()
    timed_coordinate = request("timeout-coordinate", "click", {"x": 30, "y": 40, "_yonder_action_kind": "activate-control"})
    timed_noop = request("timeout-coordinate-noop", "click", {"x": 31, "y": 41, "_yonder_action_kind": "activate-control"})
    stale_coordinate = request("stale-coordinate", "click", {"x": 32, "y": 42, "_yonder_action_kind": "activate-control"})
    generic_coordinate_error = request("generic-coordinate-error", "click", {"x": 33, "y": 43, "_yonder_action_kind": "activate-control"})
    observed_element = request("observed-element", "click", {"observed_element_index": 3, "observation_ref": generic_coordinate_error["observation_ref"], "_yonder_action_kind": "activate-control"})
    focused = request("focus", "hotkey", {"keys": ["cmd", "f"], "_yonder_action_kind": "focus-control"})
    entered = request("input", "type_text", {"text": "one last kiss", "_yonder_action_kind": "input-text"})
    activated = request("activate", "press_key", {"key": "ENTER", "_yonder_action_kind": "activate-control"})
    navigated = request("navigate-down", "press_key", {"key": "ARROWDOWN", "_yonder_action_kind": "activate-control"})
    activated_by_element = request("activate-element", "click", {"_yonder_action_kind": "activate-control"})
    double_activated = request("activate-double", "click", {"x": 10, "y": 20, "click_count": 2, "_yonder_action_kind": "activate-control"})
    visual_entered = request("visual-input", "type_text", {"text": "visual query", "x": 10, "y": 20, "_yonder_action_kind": "input-text"})
    calculator_launched = request("calculator-launch", "launch_app", {"bundle_id": "com.yonder.fixture.calculator"})
    binding = calculator_launched["launched_app_id"]
    calculator_input = request("calculator-input", "type_text", {"text": "1+1", "_yonder_action_kind": "input-text"})
    installed_fallback = request("installed-fallback", "launch_app", {"bundle_id": "com.apple.TextEdit"})
    unavailable = request("unavailable", "launch_app", {"bundle_id": "com.yonder.fixture.no-window"})
    process.terminate()
    process.wait(timeout=5)

    records = [json.loads(line) for line in log.read_text(encoding="utf-8").splitlines()]
    assert records[0] == {"rogue_approval": False}
    assert any(item == {"approval": True} for item in records)
    assert all(not item.get("rogue_approval", False) for item in records if "rogue_approval" in item)
    assert all(item["approval"] for item in records if "approval" in item)
    # 两次显式启动Worker，加上末尾“无窗口”Observe的一次受限刷新。
    # 三个坐标动作均不得额外重建Client。
    assert sum("rogue_approval" in item for item in records) == 3
    assert sum(item == {"approval": True} for item in records) == 3
    observes = [item["observe"] for item in records if "observe" in item]
    assert observes and all(item["disableDiff"] is None and item["disable_diff"] is True for item in observes)
    actions = [item for item in records if "name" in item]
    successful_results = (launched, observed_element, focused, activated, navigated, activated_by_element, double_activated, calculator_launched, calculator_input, installed_fallback)
    assert all(result["action_succeeded"] and result["observe_valid"] for result in successful_results), [(result["step_id"], result["action_effect"], result["failure_stage"], result["element_count"]) for result in successful_results]
    assert all(not result["action_succeeded"] and result["observe_valid"] and result["action_effect"] == "suspected_noop" for result in (timed_coordinate, timed_noop, stale_coordinate, generic_coordinate_error, entered, visual_entered))
    row = next(item for item in launched["elements"] if item["index"] == 3)
    assert row["role"] == "selectable-row" and "狼顾科技" in row["label"]
    counts = (launched["element_count"], activated["element_count"], navigated["element_count"], activated_by_element["element_count"])
    assert counts == (5, 6, 7, 8), counts
    assert [item["name"] for item in actions] == ["click", "click", "click", "click", "click", "press_key", "click", "press_key", "press_key", "type_text", "press_key", "press_key", "click", "click", "click", "type_text", "type_text"]
    assert actions[0]["args"]["x"] == 30 and actions[0]["args"]["y"] == 40
    assert actions[0]["args"]["mouse_button"] == "left" and actions[0]["args"]["click_count"] == 1
    assert actions[1]["args"]["x"] == 31 and actions[1]["args"]["y"] == 41
    assert actions[1]["args"]["mouse_button"] == "left" and actions[1]["args"]["click_count"] == 1
    assert actions[2]["args"]["x"] == 32 and actions[2]["args"]["y"] == 42
    assert actions[2]["args"]["mouse_button"] == "left" and actions[2]["args"]["click_count"] == 1
    assert actions[3]["args"]["x"] == 33 and actions[3]["args"]["y"] == 43
    assert actions[3]["args"]["mouse_button"] == "left" and actions[3]["args"]["click_count"] == 1
    assert actions[4]["args"]["element_index"] == "3" and "observed_element_index" not in actions[4]["args"] and "observation_ref" not in actions[4]["args"]
    assert actions[5]["args"]["key"] == "super+f"
    assert actions[6]["args"]["element_index"] == "1"
    assert actions[7]["args"]["key"] == "super+a"
    assert actions[8]["args"]["key"] == "BackSpace"
    assert actions[9]["args"]["text"] == "one last kiss"
    assert actions[10]["args"]["key"] == "Return"
    assert actions[11]["args"]["key"] == "Down"
    assert actions[12]["args"]["element_index"] == "3"
    assert actions[13]["args"]["x"] == 10 and actions[13]["args"]["y"] == 20 and actions[13]["args"]["click_count"] == 2
    assert actions[13]["args"]["mouse_button"] == "left"
    assert actions[14]["args"]["x"] == 10 and actions[14]["args"]["y"] == 20
    assert actions[15]["args"]["text"] == "visual query"
    assert actions[16]["args"]["text"] == "1+1"
    assert all(item["args"]["app"] == "/Applications/QQMusic.app" for item in actions[:16])
    assert actions[16]["args"]["app"] == "/System/Applications/Calculator.app"
    assert installed_fallback["launched_app_id"] == "com.apple.TextEdit"
    assert not unavailable["action_known"] and unavailable["failure_stage"] == "target-window-unavailable"
    assert all("_yonder_action_kind" not in item["args"] and "_yonder_private_text" not in item["args"] for item in actions)
    print(json.dumps({
        "driver": "sky",
        "transcript_elements": activated["element_count"],
        "actions": [item["name"] for item in actions],
        "out_of_scope_approval_rejected": True,
        "task_binding_reused": True,
        "binding_survived_worker_restart": True,
        "passed": True,
    }, ensure_ascii=False))
