#!/usr/bin/env python3
"""通过安装包内 Yonder MCP 验证受保护的企业微信消息计划；不直连 Socket/Driver。"""
import argparse
import json
import os
import pathlib
import subprocess
import time
import uuid

root = pathlib.Path(__file__).resolve().parents[2]
cli = root / "target/debug/Yonda.app/Contents/MacOS/yonder"

parser = argparse.ArgumentParser()
parser.add_argument("--target")
parser.add_argument("--message")
parser.add_argument("--resume-task")
parser.add_argument("--resume-sequence")
parser.add_argument("--intent-ref")
parser.add_argument("--confirmation-ref")
parser.add_argument("--focus-x", type=float)
parser.add_argument("--focus-y", type=float)
parser.add_argument("--resume-from", choices=["foreground","search","query","target","composer","draft","send"], default="search")
parser.add_argument("--confirmation-timeout", type=int, default=120)
args = parser.parse_args()
if args.resume_task:
    required = [args.resume_sequence, args.intent_ref, args.confirmation_ref]
    if any(value is None for value in required):
        parser.error("恢复任务必须提供 --resume-sequence、--intent-ref 和 --confirmation-ref")
elif args.target is None or args.message is None:
    parser.error("新任务必须提供 --target 和 --message")

environment = dict(os.environ, YONDER_AGENT_ID="codex-cli")
process = subprocess.Popen([str(cli), "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, env=environment)
counter = 0

def rpc(method, params):
    global counter
    counter += 1
    request = {"jsonrpc":"2.0","id":counter,"method":method,"params":params}
    process.stdin.write(json.dumps(request, ensure_ascii=False) + "\n")
    process.stdin.flush()
    response = json.loads(process.stdout.readline())
    if "error" in response:
        raise RuntimeError(response["error"]["message"])
    return response["result"]

def tool(name, arguments):
    result = rpc("tools/call", {"name":name,"arguments":arguments})
    content = json.loads(result["content"][0]["text"])
    if result.get("isError"):
        raise RuntimeError(content.get("message", "Yonder tool failed"))
    return content

def candidate(candidate_id, tool_name, action_kind, target_ref, fact, confirmation_ref=None):
    value = {
        "candidate_id":candidate_id,"tool_name":tool_name,"arguments":{},"action_kind":action_kind,
        "target_ref":target_ref,"preconditions":[{"fact":fact,"expected":True}],
        "expected_observe":[{"fact":fact,"expected":True}],
    }
    if confirmation_ref:
        value["confirmation_ref"] = confirmation_ref
    return value

task_id = args.resume_task
preserve_task = False
try:
    rpc("initialize", {"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"yonder-formal-verification","version":"1"}})
    if args.resume_task:
        sequence = args.resume_sequence
        target_ref, confirmation_ref = args.intent_ref, args.confirmation_ref
    else:
        created = tool("task_create", {"idempotency_key":f"cua-message-{uuid.uuid4().hex}","name":"企业微信发送确认验证","description":"在企业微信完成一条经本机确认的消息"})
        task = created["task"]
        task_id, sequence = task["task_id"], task["sequence"]
        intent = tool("task_cua_intent_propose", {"task_id":task_id,"target":args.target,"message":args.message})["intent"]
        target_ref, confirmation_ref = intent["intent_ref"], intent["confirmation_ref"]
    plan_id = f"wecom-message-{uuid.uuid4().hex}"
    slots = []
    if not args.resume_task:
        slots.extend([
            {"step_id":"launch","label":"打开企业微信","candidates":[candidate("launch-wecom","launch_app","launch-application","wecom-app","application-ready") | {"arguments":{"bundle_id":"com.tencent.WeWorkMac"}}]},
            {"step_id":"focus-search","label":"使用企业微信搜索快捷键聚焦会话搜索","candidates":[candidate("focus-search","hotkey","focus-target-search",target_ref,"application-ready") | {"arguments":{"keys":["cmd","f"]}}]},
            {"step_id":"enter-query","label":"输入会话目标","candidates":[candidate("enter-query","type_text","enter-target-query",target_ref,"target-resolved")]},
        ])
    elif args.focus_x is not None and args.focus_y is not None:
        if args.resume_from == "search":
            slots.append({"step_id":"focus-search-visual","label":"根据视觉证据聚焦会话搜索","candidates":[candidate("focus-search-coordinate","click","focus-target-search",target_ref,"application-ready") | {"arguments":{"x":args.focus_x,"y":args.focus_y}}]})
        elif args.resume_from == "target":
            slots.append({"step_id":"activate-target-visual","label":"根据视觉证据打开目标会话","candidates":[candidate("activate-target-coordinate","click","activate-target",target_ref,"target-resolved") | {"arguments":{"x":args.focus_x,"y":args.focus_y}}]})
        elif args.resume_from == "composer":
            slots.append({"step_id":"focus-composer-visual","label":"根据视觉证据聚焦消息输入框","candidates":[candidate("focus-composer-coordinate","click","focus-message-composer",target_ref,"composer-ready") | {"arguments":{"x":args.focus_x,"y":args.focus_y}}]})
    elif args.resume_from == "foreground":
        slots.append({"step_id":"foreground-wecom","label":"后台投递无效后将企业微信置于前台","candidates":[candidate("foreground-wecom","bring_to_front","bring-to-front",target_ref,"application-ready")]})
    if not args.resume_task:
        slots.append({"step_id":"activate-target","label":"打开目标会话","candidates":[candidate("activate-target","click","activate-target",target_ref,"target-resolved")]})
    if not args.resume_task:
        slots.append({"step_id":"focus-composer","label":"聚焦消息输入框","candidates":[candidate("focus-composer","click","focus-message-composer",target_ref,"composer-ready")]})
    if not args.resume_task:
        slots.append({"step_id":"draft-message","label":"填写消息草稿","candidates":[candidate("draft-message","type_text","draft-message-ref",target_ref,"composer-ready")]})
    elif args.resume_from == "query":
        slots.append({"step_id":"enter-query-visual","label":"向已核验焦点输入会话目标","candidates":[candidate("enter-query-focused","type_text","enter-target-query",target_ref,"target-resolved")]})
    elif args.resume_from == "draft":
        slots.append({"step_id":"draft-message-visual","label":"向已核验焦点填写消息草稿","candidates":[candidate("draft-message-focused","type_text","draft-message-ref",target_ref,"composer-ready")]})
    if not args.resume_task or args.resume_from == "send":
        slots.append({"step_id":"send-message","label":"确认并发送消息","candidates":[candidate("send-enter","press_key","send-message",target_ref,"delivery-confirmed",confirmation_ref)]})
    submitted = tool("task_plan_submit", {"task_id":task_id,"expected_sequence":sequence,"plan_id":plan_id,"plan_version":1,"token_budget":800,"slots":slots})
    sequence = submitted["sequence"]
    executed = tool("task_plan_execute", {"task_id":task_id,"expected_sequence":sequence,"plan_id":plan_id,"plan_version":1})
    sequence = executed["sequence"]
    if executed["disposition"] == "awaiting-confirmation":
        deadline = time.monotonic() + args.confirmation_timeout
        while time.monotonic() < deadline:
            time.sleep(1)
            executed = tool("task_plan_execute", {"task_id":task_id,"expected_sequence":sequence,"plan_id":plan_id,"plan_version":1})
            sequence = executed["sequence"]
            if executed["disposition"] != "awaiting-confirmation":
                break
    if executed["disposition"] != "fragment-complete":
        preserve_task = True
        print(json.dumps({"task_id":task_id,"status":"running","sequence":sequence,"plan_id":plan_id,"plan_version":1,"intent_ref":target_ref,"confirmation_ref":confirmation_ref,"plan_disposition":executed["disposition"],"handoff_reason":executed.get("handoff_reason"),"observation":executed.get("observation")},ensure_ascii=False))
        raise SystemExit(3)
    completed = tool("task_complete", {"task_id":task_id,"expected_sequence":sequence})
    print(json.dumps({"task_id":task_id,"status":completed["task"]["status"],"plan_disposition":executed["disposition"],"sensitive_values_printed":False},ensure_ascii=False))
except Exception:
    if task_id and not preserve_task:
        try:
            fresh = tool("task_get", {"task_id":task_id})["task"]
            if fresh["status"] not in ["completed","failed","cancelled"]:
                tool("task_cancel", {"task_id":task_id,"expected_sequence":fresh["sequence"]})
        except Exception:
            pass
    raise
finally:
    process.terminate()
    process.wait(timeout=5)
