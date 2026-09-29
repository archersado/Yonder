#!/usr/bin/env python3
"""隔离验证 trycua partial 证据消费；不调用真实桌面副作用。"""
import json
import pathlib
import subprocess
import tempfile

root = pathlib.Path(__file__).resolve().parents[2]
worker = root / "crates/adapters/src/cua_worker.mjs"

sdk_source = r'''
export class CuaDriver {
  static create() {
    return {
      async metadata() {},
      async listToolsJson() { return JSON.stringify({tools:[
        {name:'launch_app',inputSchema:{properties:{bundle_id:{}}}},
        {name:'bring_to_front',inputSchema:{properties:{pid:{},window_id:{}}}},
        {name:'get_window_state',inputSchema:{properties:{pid:{},window_id:{}}}},
        {name:'get_desktop_state',inputSchema:{properties:{}}},
        {name:'list_apps',inputSchema:{properties:{}}},
        {name:'list_windows',inputSchema:{properties:{pid:{}}}}
      ]}); },
      async callTool(name) {
        const ok = value => ({isError:false,structuredJson:JSON.stringify(value)});
        if (name === 'get_window_state' || name === 'get_desktop_state') return ok({elements:[]});
        if (name === 'list_apps') return ok({apps:[{running:true,bundle_id:'com.example.OneWindow',pid:42}]});
        if (name === 'list_windows') return ok({windows:[{window_id:7,is_on_screen:true,on_current_space:true,bounds:{width:800,height:600}}]});
        if (name === 'launch_app') return ok({pid:42,bundle_id:'com.example.OneWindow',launch_state:'process_running',windows:[{window_id:7,bounds:{width:800,height:600}}]});
        if (name === 'bring_to_front') return {isError:true,structuredJson:JSON.stringify({
          code:'bring_to_front_exact_window_unverified',request_accepted:true,process_activated:true,
          observed:{front_process_matches_target:true,workspace_frontmost_pid:42},
          exact_window_effect:{verified:false},effect:'refused'
        })};
        throw new Error('unexpected tool');
      },
      async shutdown() {},
      uniffiDestroy() {}
    };
  }
}
'''

with tempfile.TemporaryDirectory(prefix="yonda-cua-partial-") as temporary:
    directory = pathlib.Path(temporary)
    sdk = directory / "sdk.mjs"
    evidence = directory / "evidence"
    sdk.write_text(sdk_source)
    evidence.mkdir()
    base = {
        "task_id": "task", "step_id": "step", "attempt_id": "attempt",
        "worker_instance_id": "worker", "host_session_id": "host",
        "pid": 11, "window_id": 12,
    }
    launch = {**base, "tool_name": "launch_app", "arguments": {"bundle_id": "com.example.OneWindow"}}
    bring = {**base, "step_id": "focus", "attempt_id": "focus-attempt", "tool_name": "bring_to_front", "arguments": {}}
    run = subprocess.run(
        ["node", str(worker), str(sdk), str(evidence)],
        input=json.dumps(launch) + "\n" + json.dumps(bring) + "\n",
        text=True, capture_output=True, timeout=10, check=True,
    )
    responses = [json.loads(line) for line in run.stdout.splitlines()]
    assert responses[0]["action_succeeded"] is True
    response = responses[1]

assert response["action_known"] is True
assert response["action_succeeded"] is True
assert response["action_effect"] == "confirmed"
assert response["observe_valid"] is True
assert response["target_visible"] is True
print(json.dumps({"partial_evidence_consumed": True, "visible_application_confirmed": True}))
