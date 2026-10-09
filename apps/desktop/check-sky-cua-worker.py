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
import base64, json, os, sys, time
state_path = os.environ['YONDER_SKY_TEST_STATE']
delay_marker = state_path + '.startup-delayed'
text = '1 文本框 搜索\\n2 按钮 播放\\n3 row (selectable)\\n\\t4 单元格\\n\\t\\t5 文本 狼顾科技\\n6 图像\\n7 按钮 账户\\n\\t8 图像\\n9 按钮 disabled 未读' + open(state_path, encoding='utf-8').read()
calculator_text = '1 按钮 1\\n2 文本 0'
approved = False
log = os.environ['YONDER_SKY_TEST_LOG']
for line in sys.stdin:
    request = json.loads(line)
    if 'id' not in request:
        continue
    if request['method'] == 'initialize':
        # 回归真实官方Client首次XPC握手略高于5秒的情况；只延迟首个Worker，
        # 避免把整个夹具放大为长时间测试。
        if not os.path.exists(delay_marker):
            open(delay_marker, 'w', encoding='utf-8').close()
            time.sleep(6)
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
            semantic_change = '\\n10 文本 同尺寸新界面' if os.path.exists(state_path + '.semantic-changed') else ''
            state = calculator_text if args['app'].endswith('Calculator.app') else text + semantic_change
            tree_state = '\\n'.join('├─ ' + item for item in state.split('\\n'))
            wrapped = 'App state follows:\\n```json\\n' + json.dumps({'app':args['app'],'text':tree_state}, ensure_ascii=False) + '\\n```' if args['app'].endswith('QQMusic.app') else state
            screenshot_size = 120 if os.path.exists(state_path + '.geometry-changed') else 100
            png_header = bytes([137,80,78,71,13,10,26,10,0,0,0,13,73,72,68,82]) + screenshot_size.to_bytes(4, 'big') + screenshot_size.to_bytes(4, 'big')
            result = {'isError':True,'content':[{'type':'text','text':'Computer Use server error -10005: cgWindowNotFound'}]} if args['app'].endswith('NoWindow.app') else ({'content':[{'type':'text','text':wrapped},{'type':'image','mimeType':'image/png','data':base64.b64encode(png_header).decode()}]} if approved else {'isError':True,'content':[{'type':'text','text':'denied'}]})
        else:
            with open(log, 'a', encoding='utf-8') as output:
                output.write(json.dumps({'name':name,'args':args}, ensure_ascii=False)+'\\n')
            if name == 'set_value' and args['value'] != 'ignored value':
                text = f\"1 文本框 搜索 {args['value']}\\n2 按钮 播放\\n3 选项 one\\n4 选项 last\\n5 选项 kiss 宇多田光\"
            elif name == 'type_text' and args['app'].endswith('QQMusic.app'):
                text = f\"1 文本框 搜索 {args['text']}\\n2 按钮 播放\\n3 选项 one\\n4 选项 last\\n5 选项 kiss 宇多田光\"
            elif name == 'click' and args.get('element_index') == '3':
                text += '\\n7 文本 已打开搜索结果'
            elif name == 'click' and args.get('element_index') == '6':
                text += '\\n20 文本 已激活图像入口'
            elif name == 'click' and args.get('element_index') == '2':
                text += '\\n10 文本 企业菜单已打开'
            elif name == 'click' and args.get('click_count') == 2:
                text += '\\n8 文本 已双击激活自绘结果'
            elif name == 'click' and args.get('x') == 30 and args.get('y') == 59:
                text += '\\n9 文本 企业菜单已打开'
                with open(state_path, 'a', encoding='utf-8') as state_output:
                    state_output.write('\\n9 文本 企业菜单已打开')
                result = {'isError':True,'content':[{'type':'text','text':'request timed out'}]}
                print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':result}, ensure_ascii=False), flush=True)
                continue
            elif name == 'click' and args.get('x') == 31 and args.get('y') == 58:
                result = {'isError':True,'content':[{'type':'text','text':'request timed out'}]}
                print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':result}, ensure_ascii=False), flush=True)
                continue
            elif name == 'click' and args.get('x') == 32 and args.get('y') == 57:
                text += '\\n11 文本 窗口代次已变化'
                result = {'isError':True,'content':[{'type':'text','text':'Computer Use server error -10005: cgWindowNotFound'}]}
                print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':result}, ensure_ascii=False), flush=True)
                continue
            elif name == 'click' and args.get('x') == 33 and args.get('y') == 56:
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
    focused = request("focus", "hotkey", {"keys": ["cmd", "f"], "_yonder_action_kind": "focus-control"})
    semantic_marker = pathlib.Path(str(state) + ".semantic-changed")
    semantic_marker.touch()
    rejected_changed_semantics = request("rejected-changed-semantics", "click", {"x": 10, "y": 20, "observation_ref": focused["observation_ref"], "_yonder_action_kind": "activate-control"})
    timed_coordinate = request("timeout-coordinate", "click", {"x": 30, "y": 40, "observation_ref": rejected_changed_semantics["observation_ref"], "_yonder_action_kind": "activate-control"})
    timed_noop = request("timeout-coordinate-noop", "click", {"x": 31, "y": 41, "observation_ref": timed_coordinate["observation_ref"], "_yonder_action_kind": "activate-control"})
    stale_coordinate = request("stale-coordinate", "click", {"x": 32, "y": 42, "observation_ref": timed_noop["observation_ref"], "_yonder_action_kind": "activate-control"})
    generic_coordinate_error = request("generic-coordinate-error", "click", {"x": 33, "y": 43, "observation_ref": stale_coordinate["observation_ref"], "_yonder_action_kind": "activate-control"})
    rejected_stale_ref = request("rejected-stale-ref", "click", {"x": 10, "y": 20, "observation_ref": focused["observation_ref"], "_yonder_action_kind": "activate-control"})
    rejected_out_of_bounds = request("rejected-out-of-bounds", "click", {"x": 100, "y": 20, "observation_ref": rejected_stale_ref["observation_ref"], "_yonder_action_kind": "activate-control"})
    geometry_marker = pathlib.Path(str(state) + ".geometry-changed")
    geometry_marker.touch()
    rejected_changed_geometry = request("rejected-changed-geometry", "click", {"x": 10, "y": 20, "observation_ref": rejected_out_of_bounds["observation_ref"], "_yonder_action_kind": "activate-control"})
    geometry_marker.unlink()
    observed_element = request("observed-element", "click", {"observed_element_index": 3, "observation_ref": rejected_changed_geometry["observation_ref"], "_yonder_action_kind": "activate-control"})
    image_element = request("image-element", "click", {"observed_element_index": 6, "observation_ref": observed_element["observation_ref"], "_yonder_action_kind": "activate-control"})
    entered = request("input", "type_text", {"text": "one last kiss", "_yonder_action_kind": "input-text"})
    activated = request("activate", "press_key", {"key": "ENTER", "_yonder_action_kind": "activate-control"})
    navigated = request("navigate-down", "press_key", {"key": "ARROWDOWN", "_yonder_action_kind": "activate-control"})
    activated_by_element = request("activate-element", "click", {"_yonder_action_kind": "activate-control"})
    refocused = request("refocus", "hotkey", {"keys": ["cmd", "f"], "_yonder_action_kind": "focus-control"})
    focused_entered = request("focused-input", "type_text", {"text": "focused query", "_yonder_action_kind": "input-text"})
    unconfirmed_input = request("unconfirmed-input", "type_text", {"text": "ignored value", "_yonder_action_kind": "input-text"})
    double_activated = request("activate-double", "click", {"x": 10, "y": 20, "click_count": 2, "observation_ref": unconfirmed_input["observation_ref"], "_yonder_action_kind": "activate-control"})
    visual_entered = request("visual-input", "type_text", {"text": "visual query", "x": 10, "y": 20, "observation_ref": double_activated["observation_ref"], "_yonder_action_kind": "input-text"})
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
    successful_results = (launched, observed_element, image_element, focused, entered, activated, navigated, activated_by_element, refocused, focused_entered, double_activated, visual_entered, calculator_launched, calculator_input, installed_fallback)
    assert all(result["action_succeeded"] and result["observe_valid"] for result in successful_results), [(result["step_id"], result["action_effect"], result["failure_stage"], result["element_count"]) for result in successful_results]
    assert all(not result["action_succeeded"] and result["observe_valid"] and result["action_effect"] == "suspected_noop" for result in (timed_coordinate, timed_noop, stale_coordinate, generic_coordinate_error, unconfirmed_input))
    assert timed_coordinate["failure_stage"] == "driver-action-timed-out"
    assert timed_noop["failure_stage"] == "driver-action-timed-out"
    assert stale_coordinate["failure_stage"] == "driver-action-rejected"
    assert generic_coordinate_error["failure_stage"] == "driver-action-rejected"
    assert unconfirmed_input["failure_stage"] == "action-unconfirmed-unparsed-transcript"
    assert all(result["action_known"] and not result["action_succeeded"] and result["observe_valid"] and result["action_effect"] == "unverifiable" and result["failure_stage"] == "target-stale-visual-observation" for result in (rejected_changed_semantics, rejected_stale_ref, rejected_out_of_bounds, rejected_changed_geometry))
    assert all(result["screenshot_path"] is None and result["screenshot_mime"] is None for result in (launched, focused, entered))
    assert all(result["screenshot_width_px"] == 100 and result["screenshot_height_px"] == 100 for result in (generic_coordinate_error, unconfirmed_input))
    row = next(item for item in launched["elements"] if item["index"] == 3)
    assert row["role"] == "selectable-row" and "狼顾科技" in row["label"]
    image = next(item for item in launched["elements"] if item["index"] == 6)
    assert image == {"index": 6, "role": "image", "label": "图像"}
    assert next(item for item in launched["elements"] if item["index"] == 7)["role"] == "button"
    assert all(item["index"] not in (8, 9) for item in launched["elements"])
    assert launched["element_count"] == 9
    expected_action_names = ["press_key", "click", "click", "click", "click", "click", "click", "set_value", "press_key", "press_key", "click", "press_key", "type_text", "set_value", "click", "click", "type_text", "type_text"]
    assert [item["name"] for item in actions] == expected_action_names, [item["name"] for item in actions]
    assert actions[0]["args"]["key"] == "super+f"
    assert actions[1]["args"]["x"] == 30 and actions[1]["args"]["y"] == 59
    assert actions[1]["args"]["mouse_button"] == "left" and actions[1]["args"]["click_count"] == 1
    assert actions[2]["args"]["x"] == 31 and actions[2]["args"]["y"] == 58
    assert actions[2]["args"]["mouse_button"] == "left" and actions[2]["args"]["click_count"] == 1
    assert actions[3]["args"]["x"] == 32 and actions[3]["args"]["y"] == 57
    assert actions[3]["args"]["mouse_button"] == "left" and actions[3]["args"]["click_count"] == 1
    assert actions[4]["args"]["x"] == 33 and actions[4]["args"]["y"] == 56
    assert actions[4]["args"]["mouse_button"] == "left" and actions[4]["args"]["click_count"] == 1
    assert actions[5]["args"]["element_index"] == "3" and "observed_element_index" not in actions[5]["args"] and "observation_ref" not in actions[5]["args"]
    assert actions[6]["args"]["element_index"] == "6" and "observed_element_index" not in actions[6]["args"] and "observation_ref" not in actions[6]["args"]
    assert actions[7]["args"]["element_index"] == "1" and actions[7]["args"]["value"] == "one last kiss"
    assert actions[8]["args"]["key"] == "Return"
    assert actions[9]["args"]["key"] == "Down"
    assert actions[10]["args"]["element_index"] == "3"
    assert actions[11]["args"]["key"] == "super+f"
    assert actions[12]["args"]["text"] == "focused query"
    assert actions[13]["args"]["element_index"] == "1" and actions[13]["args"]["value"] == "ignored value"
    assert actions[14]["args"]["x"] == 10 and actions[14]["args"]["y"] == 79 and actions[14]["args"]["click_count"] == 2
    assert actions[14]["args"]["mouse_button"] == "left"
    assert actions[15]["args"]["x"] == 10 and actions[15]["args"]["y"] == 79
    assert actions[16]["args"]["text"] == "visual query"
    assert actions[17]["args"]["text"] == "1+1"
    assert all(item["args"]["app"] == "/Applications/QQMusic.app" for item in actions[:17])
    assert actions[17]["args"]["app"] == "/System/Applications/Calculator.app"
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
