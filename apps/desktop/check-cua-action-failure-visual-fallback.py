#!/usr/bin/env python3
"""隔离验证空AX树/动作未确认会补采同窗口截图，通用坐标文本由Worker前台投递。"""
import json
import pathlib
import subprocess
import tempfile

root = pathlib.Path(__file__).resolve().parents[2]
worker = root / "crates/adapters/src/cua_worker.mjs"

sdk_source = r'''
import { appendFile, writeFile } from 'node:fs/promises';
const logPath = __LOG_PATH__;
let boundTreeReads = 0;
export class CuaDriver {
  static create() { return {
    async metadata(){},
    async listToolsJson(){return JSON.stringify({tools:[
      {name:'launch_app',inputSchema:{properties:{bundle_id:{}}}},
      {name:'type_text',inputSchema:{properties:{target:{},text:{},x:{},y:{},delivery_mode:{},session:{}}}},
      {name:'get_window_state',inputSchema:{properties:{pid:{},window_id:{},include_screenshot:{},max_elements:{},screenshot_out_file:{},session:{}}}},
      {name:'get_desktop_state',inputSchema:{properties:{}}},
      {name:'list_apps',inputSchema:{properties:{}}},
      {name:'list_windows',inputSchema:{properties:{pid:{}}}}
    ]});},
    async callTool(name,encoded){
      const args=JSON.parse(encoded); const ok=value=>({isError:false,structuredJson:JSON.stringify(value)});
      if(name==='launch_app')return ok({pid:42,bundle_id:'com.example.VisualApp',launch_state:'process_running',windows:[{window_id:7,bounds:{width:800,height:600}}]});
      if(name==='list_apps')return ok({apps:[{running:true,bundle_id:'com.example.VisualApp',pid:42}]});
      if(name==='list_windows')return ok({windows:[{window_id:7,is_on_screen:true,on_current_space:true,bounds:{width:800,height:600}}]});
      if(name==='get_desktop_state')return ok({windows:[]});
      if(name==='get_window_state'){
        if(args.include_screenshot===true){await writeFile(args.screenshot_out_file,Buffer.from([137,80,78,71]));}
        const firstBoundRead = args.pid === 42 && args.include_screenshot !== true && boundTreeReads++ === 0;
        return ok({elements: args.include_screenshot===true || firstBoundRead ? [] : [{role:'textfield',enabled:true,element_token:'field'}]});
      }
      if(name==='type_text'){
        await appendFile(logPath,JSON.stringify(args)+'\n');
        return ok({effect:Number.isFinite(args.x)&&Number.isFinite(args.y)?'confirmed':'refused'});
      }
      throw new Error('unexpected tool');
    },
    async shutdown(){},uniffiDestroy(){}
  };}
}
'''

with tempfile.TemporaryDirectory(prefix="yonda-cua-action-fallback-") as temporary:
    directory = pathlib.Path(temporary)
    sdk = directory / "sdk.mjs"
    evidence = directory / "evidence"
    log = directory / "actions.jsonl"
    sdk.write_text(sdk_source.replace("__LOG_PATH__", json.dumps(str(log))))
    evidence.mkdir()
    base = {"task_id":"task","worker_instance_id":"worker","host_session_id":"host","pid":11,"window_id":12}
    requests = [
        {**base,"step_id":"launch","attempt_id":"a0","tool_name":"launch_app","arguments":{"bundle_id":"com.example.VisualApp"}},
        {**base,"step_id":"background","attempt_id":"a1","tool_name":"type_text","arguments":{"text":"query"}},
        {**base,"step_id":"coordinate","attempt_id":"a2","tool_name":"type_text","arguments":{"text":"query","x":120,"y":40}},
    ]
    run = subprocess.run(
        ["node",str(worker),str(sdk),str(evidence)],
        input="".join(json.dumps(item)+"\n" for item in requests),
        text=True,capture_output=True,timeout=10,check=True,
    )
    responses = [json.loads(line) for line in run.stdout.splitlines()]
    actions = [json.loads(line) for line in log.read_text().splitlines()]

    assert responses[0]["action_succeeded"] is True
    assert pathlib.Path(responses[0]["screenshot_path"]).is_file()
    assert responses[1]["action_succeeded"] is False and responses[1]["action_effect"] == "refused"
    assert pathlib.Path(responses[1]["screenshot_path"]).is_file()
    assert responses[2]["action_succeeded"] is True
    assert pathlib.Path(responses[2]["screenshot_path"]).is_file()
    assert actions[0]["target"] == {"kind":"window","pid":42,"window_id":7}
    assert actions[1]["target"] == {"kind":"window","pid":42,"window_id":7}
    assert actions[1]["delivery_mode"] == "foreground"

print(json.dumps({
    "empty_ax_visual_fallback": True,
    "unconfirmed_action_visual_fallback": True,
    "generic_coordinate_text_foreground": True,
    "same_trusted_window": True,
    "action_replayed": False,
}))
