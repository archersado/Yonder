#!/usr/bin/env python3
"""隔离验证元素解析失败时才采集视觉证据，且不执行原动作。"""
import json
import pathlib
import subprocess
import tempfile

root = pathlib.Path(__file__).resolve().parents[2]
worker = root / "crates/adapters/src/cua_worker.mjs"

sdk_source = r'''
import { writeFile } from 'node:fs/promises';
export class CuaDriver {
  static create() {
    return {
      async metadata() {},
      async listToolsJson() { return JSON.stringify({tools:[
        {name:'launch_app',inputSchema:{properties:{bundle_id:{}}}},
        {name:'click',inputSchema:{properties:{pid:{},window_id:{},element_token:{}}}},
        {name:'type_text',inputSchema:{properties:{text:{},pid:{},window_id:{},element_token:{}}}},
        {name:'get_window_state',inputSchema:{properties:{pid:{},window_id:{},include_screenshot:{},max_elements:{},screenshot_out_file:{}}}},
        {name:'get_desktop_state',inputSchema:{properties:{}}},
        {name:'list_apps',inputSchema:{properties:{}}},
        {name:'list_windows',inputSchema:{properties:{pid:{}}}}
      ]}); },
      async callTool(name, encoded) {
        const args = JSON.parse(encoded);
        const ok = value => ({isError:false,structuredJson:JSON.stringify(value)});
        if (name === 'type_text' || name === 'click') throw new Error('side effect must not run');
        if (name === 'launch_app') return ok({pid:42,bundle_id:'com.tencent.WeWorkMac',launch_state:'process_running',windows:[{window_id:7,bounds:{width:800,height:600}}]});
        if (name === 'list_apps') return ok({apps:[{running:true,bundle_id:'com.tencent.WeWorkMac',pid:42}]});
        if (name === 'list_windows') return ok({windows:[{window_id:7,is_on_screen:false,on_current_space:false,bounds:{width:800,height:600}}]});
        if (name === 'get_desktop_state') return ok({windows:[]});
        if (name === 'get_window_state' && args.include_screenshot) {
          if (args.pid !== 42 || args.window_id !== 7) throw new Error('visual fallback used the frontmost window');
          await writeFile(args.screenshot_out_file, Buffer.from([137,80,78,71]));
          return ok({elements:[
            {role:'textfield',enabled:true,element_token:'one'},
            {role:'textfield',enabled:true,element_token:'two'}
          ]});
        }
        if (name === 'get_window_state') return ok({elements:[
          {role:'textfield',enabled:true,element_token:'one'},
          {role:'textfield',enabled:true,element_token:'two'}
        ]});
        throw new Error('unexpected tool');
      },
      async shutdown() {},
      uniffiDestroy() {}
    };
  }
}
'''

with tempfile.TemporaryDirectory(prefix="yonda-cua-visual-") as temporary:
    directory = pathlib.Path(temporary)
    sdk = directory / "sdk.mjs"
    evidence = directory / "evidence"
    sdk.write_text(sdk_source)
    evidence.mkdir()
    base = {
        "task_id": "task", "step_id": "type-contact", "attempt_id": "attempt",
        "worker_instance_id": "worker", "host_session_id": "host",
        "pid": 11, "window_id": 12,
    }
    launch = {
        **base, "step_id": "launch", "attempt_id": "launch-attempt",
        "tool_name": "launch_app", "arguments": {"bundle_id": "com.tencent.WeWorkMac"},
    }
    request = {
        **base, "tool_name": "type_text",
        "arguments": {"text": "contact", "delivery_mode": "background"},
    }
    semantic = {
        **base, "step_id": "focus-search", "attempt_id": "semantic-attempt", "tool_name": "click",
        "arguments": {"_yonder_action_kind": "focus-target-search", "_yonder_private_text": "private-target"},
    }
    run = subprocess.run(
        ["node", str(worker), str(sdk), str(evidence)],
        input=json.dumps(launch) + "\n" + json.dumps(request) + "\n" + json.dumps(semantic) + "\n", text=True, capture_output=True,
        timeout=10, check=True,
    )
    responses = [json.loads(line) for line in run.stdout.splitlines()]
    assert responses[0]["action_succeeded"] is True
    response = responses[1]
    screenshot = pathlib.Path(response["screenshot_path"])
    assert screenshot.parent == evidence
    assert screenshot.is_file()
    semantic_response = responses[2]
    assert semantic_response["action_succeeded"] is False
    assert semantic_response["observe_valid"] is True
    assert pathlib.Path(semantic_response["screenshot_path"]).is_file()

assert response["action_known"] is True
assert response["action_succeeded"] is False
assert response["action_effect"] == "refused"
assert response["observe_valid"] is True
assert response["element_count"] == 2
assert response["screenshot_mime"] == "image/png"
print(json.dumps({"element_first": True, "visual_fallback": True, "semantic_visual_fallback": True, "trusted_background_target": True, "side_effect_executed": False}))
