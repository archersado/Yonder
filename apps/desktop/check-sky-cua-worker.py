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
    package = fixture / "node_modules/@oai/sky"
    package.mkdir(parents=True)
    log = fixture / "actions.jsonl"
    evidence = fixture / "evidence"
    evidence.mkdir()
    (package / "package.json").write_text(
        json.dumps({"name": "@oai/sky", "version": "0.7.1", "type": "module"}),
        encoding="utf-8",
    )
    (package / "index.js").write_text(
        """
import { appendFile } from 'node:fs/promises';
const log = process.env.YONDER_SKY_TEST_LOG;
let text = '1 文本框 搜索\\n2 按钮 播放';
async function record(name,args){await appendFile(log,JSON.stringify({name,args})+'\\n');}
export const sky={
  target:'mac',
  async list_apps(){return [{id:'com.tencent.QQMusicMac',displayName:'QQ音乐',isRunning:false}]},
  async get_app_state(){return {app:'com.tencent.QQMusicMac',text,screenshot:null}},
  async click(args){await record('click',args)},
  async set_value(args){await record('set_value',args);text=`1 文本框 搜索 ${args.value}\\n2 按钮 播放`},
  async press_key(args){await record('press_key',args);text+='\\n3 文本 搜索结果'}
};
""",
        encoding="utf-8",
    )
    node = shutil.which("node")
    assert node is not None
    process = subprocess.Popen(
        [node, str(worker), str(package / "index.js"), str(evidence)],
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
    process.terminate()
    process.wait(timeout=5)

    actions = [json.loads(line) for line in log.read_text(encoding="utf-8").splitlines()]
    assert all(result["action_succeeded"] and result["observe_valid"] for result in (launched, focused, entered, activated))
    assert launched["element_count"] == 2 and activated["element_count"] == 3
    assert [item["name"] for item in actions] == ["click", "set_value", "press_key"]
    assert actions[0]["args"]["element_index"] == 1
    assert actions[1]["args"]["element_index"] == 1
    assert actions[2]["args"]["key"] == "Return"
    assert all("_yonder_action_kind" not in item["args"] and "_yonder_private_text" not in item["args"] for item in actions)
    print(json.dumps({
        "driver": "sky",
        "transcript_elements": activated["element_count"],
        "actions": [item["name"] for item in actions],
        "task_binding_reused": True,
        "passed": True,
    }, ensure_ascii=False))
