#!/usr/bin/env python3
"""验证企微原生AX角色、消息引用映射与发送后置事实；不连接真实桌面。"""
import json
import os
from pathlib import Path
import subprocess
import tempfile

worker = Path(__file__).resolve().parents[2] / "crates/adapters/src/sky_cua_worker.mjs"
with tempfile.TemporaryDirectory(prefix="yonder-sky-message-") as directory:
    root = Path(directory)
    bridge = root / "bridge"
    log = root / "calls.jsonl"
    bridge.write_text('''#!/usr/bin/env python3
import json, os, sys
query = ''
draft = 'fixture-message'
messages = []
selected = True
def tree():
    row = 'selected' if selected else 'selectable'
    return f'0 标准窗口\\n\\t1 文本栏 (settable) {query}\\n\\t2 row ({row})\\n\\t\\t3 单元格\\n\\t\\t\\t4 text fixture-recipient\\n\\t5 文本 fixture-recipient\\n\\t6 文本输入区 (settable) {draft}' + ''.join(f'\\n\\t{20+i} 文本输入区 {value}' for i, value in enumerate(messages))
for line in sys.stdin:
    request = json.loads(line)
    if 'id' not in request: continue
    method = request['method']
    if method == 'initialize': result = {'protocolVersion':'2025-06-18'}
    elif method == 'tools/list': result = {'tools':[{'name':'get_app_state','inputSchema':{'properties':{'app':{},'disable_diff':{}}}}]}
    else:
        name = request['params']['name']
        args = request['params']['arguments']
        if name == 'list_apps': result = {'content':[{'type':'text','text':json.dumps([{'id':'com.yonder.fixture.messages','path':'/Applications/MessageFixture.app','isRunning':False}])}]}
        elif name == 'get_app_state': result = {'content':[{'type':'text','text':tree()}]}
        else:
            with open(os.environ['MESSAGE_FIXTURE_LOG'], 'a') as output: output.write(json.dumps({'name':name,'args':args})+'\\n')
            if name == 'type_text': query = args['text']
            elif name == 'set_value': draft = args['value']
            elif name == 'click' and args.get('element_index') == '2': selected = True
            elif name == 'press_key' and args['key'] == 'BackSpace': query = ''
            elif name == 'press_key' and args['key'] == 'Return':
                if os.environ.get('MESSAGE_FIXTURE_NOOP') != '1': messages.append(draft); draft = ''
            result = {'content':[]}
    print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':result},ensure_ascii=False),flush=True)
''', encoding="utf-8")
    bridge.chmod(0o700)
    def sample(noop=False):
        process = subprocess.Popen(["node", str(worker), str(bridge), str(root)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True,
            env=dict(os.environ, MESSAGE_FIXTURE_LOG=str(log), MESSAGE_FIXTURE_NOOP="1" if noop else "0"))
        def request(step, tool, arguments):
            payload = dict(task_id="task-message", step_id=step, attempt_id="attempt-"+step,
                worker_instance_id="worker", host_session_id="host", pid=1, window_id=1,
                tool_name=tool, arguments=arguments)
            if tool != "launch_app": payload["bound_application_id"] = "com.yonder.fixture.messages"
            process.stdin.write(json.dumps(payload)+"\n")
            process.stdin.flush()
            return json.loads(process.stdout.readline())
        launched = request("launch", "launch_app", {"bundle_id":"com.yonder.fixture.messages"})
        assert any(item["role"] == "text-field" and item["index"] == 1 for item in launched["elements"])
        for step, tool, kind, value in [
            ("focus", "click", "focus-target-search", None),
            ("query", "type_text", "enter-target-query", "fixture-recipient"),
            ("target", "click", "activate-target", "fixture-recipient"),
            ("composer", "click", "focus-message-composer", None),
            ("same-draft", "type_text", "draft-message-ref", "fixture-message"),
            ("new-draft", "type_text", "draft-message-ref", "fixture-new-message"),
        ]:
            args = {"_yonder_action_kind":kind}
            if value is not None: args["_yonder_private_text"] = value
            result = request(step, tool, args)
            assert result["action_succeeded"] and result["observe_valid"], (step, result["failure_stage"])
        changed = request("changed-send", "press_key", {"_yonder_action_kind":"send-message", "_yonder_private_text":"fixture-different-message"})
        assert not changed["action_known"] and changed["failure_stage"] == "target-semantic-element"
        wrong_target = request("changed-target", "press_key", {"_yonder_action_kind":"send-message", "_yonder_private_text":"fixture-new-message", "_yonder_private_target":"fixture-other-recipient"})
        assert not wrong_target["action_known"]
        sent = request("send", "press_key", {"_yonder_action_kind":"send-message", "_yonder_private_text":"fixture-new-message", "_yonder_private_target":"fixture-recipient"})
        assert sent["observe_valid"] and sent["action_succeeded"] is not noop
        process.terminate()
        process.wait(timeout=5)
    sample()
    sample(noop=True)
    actions = [json.loads(line) for line in log.read_text().splitlines()]
    assert sum(action["name"] == "set_value" for action in actions) == 2  # 相同草稿不重复输入。
    assert sum(action["name"] == "press_key" and action["args"]["key"] == "Return" for action in actions) == 2
    assert all(not any(key.startswith("_yonder") for key in action["args"]) for action in actions)
    print(json.dumps({"passed":True,"unlabelled_field":True,"draft_not_duplicated":True,"send_noop_rejected":True}))
