#!/usr/bin/env python3
import json, os, pathlib, subprocess

root = pathlib.Path(__file__).resolve().parents[4]
home = pathlib.Path(os.environ["YONDER_HOME"])
binary = root / "target/debug/yonder"
output = pathlib.Path(__file__).with_name("cancel-result.json")
agent_id = "local-test-agent"
tasks = json.loads(pathlib.Path(__file__).with_name("agent-result.json").read_text())["task_ids"]

def call(process, number, name, arguments):
    process.stdin.write(json.dumps({"jsonrpc":"2.0","id":number,"method":"tools/call","params":{"name":name,"arguments":arguments}}, ensure_ascii=False) + "\n")
    process.stdin.flush()
    result = json.loads(process.stdout.readline())["result"]
    if result["isError"]:
        raise RuntimeError(json.loads(result["content"][0]["text"]))
    return json.loads(result["content"][0]["text"])

with subprocess.Popen([str(binary), "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True,
                      env={"HOME": str(home), "PATH": "/usr/bin:/bin", "YONDER_AGENT_ID": agent_id}) as process:
    cancelled = []
    for number, task_id in enumerate(tasks, 10):
        task = call(process, number, "task_cancel", {"task_id": task_id, "expected_sequence": "1"})["task"]
        assert task["status"] == "cancelled"
        cancelled.append(task)
    result = {"passed": True, "cancelled_count": len(cancelled), "cancelled_ids": [task["task_id"] for task in cancelled]}
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(result, ensure_ascii=False, indent=2))
