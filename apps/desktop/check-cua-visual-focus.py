#!/usr/bin/env python3
"""隔离验证不可核实视觉聚焦交回后，仅允许同任务同窗口引用文本消费一次。"""
import json
import pathlib
import subprocess
import tempfile

root = pathlib.Path(__file__).resolve().parents[2]
worker = root / "crates/adapters/src/cua_worker.mjs"

sdk_source = r'''
import { appendFile, writeFile } from 'node:fs/promises';
const logPath = __LOG_PATH__;
export class CuaDriver {
  static create() { return {
    async metadata(){},
    async listToolsJson(){return JSON.stringify({tools:[
      {name:'launch_app',inputSchema:{properties:{bundle_id:{}}}},
      {name:'hotkey',inputSchema:{properties:{target:{},keys:{},delivery_mode:{}}}},
      {name:'click',inputSchema:{properties:{target:{},x:{},y:{},delivery_mode:{}}}},
      {name:'type_text',inputSchema:{properties:{target:{},text:{},x:{},y:{},delivery_mode:{}}}},
      {name:'get_window_state',inputSchema:{properties:{pid:{},window_id:{},include_screenshot:{},max_elements:{},screenshot_out_file:{}}}},
      {name:'get_desktop_state',inputSchema:{properties:{}}},{name:'list_apps',inputSchema:{properties:{}}},{name:'list_windows',inputSchema:{properties:{pid:{}}}}
    ]});},
    hotkeys:0,
    async callTool(name,encoded){const args=JSON.parse(encoded);const ok=value=>({isError:false,structuredJson:JSON.stringify(value)});
      if(name==='launch_app')return ok({pid:42,bundle_id:'com.tencent.WeWorkMac',launch_state:'process_running',windows:[{window_id:7,bounds:{width:800,height:600}}]});
      if(name==='list_apps')return ok({apps:[{running:true,bundle_id:'com.tencent.WeWorkMac',pid:42}]});
      if(name==='list_windows')return ok({windows:[{window_id:7,is_on_screen:true,on_current_space:true,bounds:{width:800,height:600}}]});
      if(name==='get_window_state'){if(args.screenshot_out_file)await writeFile(args.screenshot_out_file,'png');return ok({elements:[]});}
      if(name==='get_desktop_state')return ok({windows:[]});
      if(name==='hotkey'){await appendFile(logPath,JSON.stringify({name,args})+'\n');return ok({effect:this.hotkeys++===0?'confirmed':'unverifiable',route:'synthetic_events',delivery:{mode:'background'}});}
      if(name==='click'){await appendFile(logPath,JSON.stringify({name,args})+'\n');return ok({effect:'confirmed',route:'synthetic_events',delivery:{mode:'background'}});}
      if(name==='type_text'){await appendFile(logPath,JSON.stringify({name,args})+'\n');return ok({effect:Number.isFinite(args.x)&&Number.isFinite(args.y)?'confirmed':'unverifiable',route:'synthetic_events',delivery:{mode:args.delivery_mode??'background'}});}
      throw new Error('unexpected tool');},async shutdown(){},uniffiDestroy(){}
  };}
}
'''

with tempfile.TemporaryDirectory(prefix="yonda-cua-visual-focus-") as temporary:
    directory = pathlib.Path(temporary)
    sdk = directory / "sdk.mjs"
    evidence = directory / "evidence"
    log = directory / "actions.jsonl"
    sdk.write_text(sdk_source.replace("__LOG_PATH__", json.dumps(str(log))))
    evidence.mkdir()
    base = {"task_id":"task","worker_instance_id":"worker","host_session_id":"host","pid":11,"window_id":12}
    requests = [
        {**base,"step_id":"launch","attempt_id":"a0","tool_name":"launch_app","arguments":{"bundle_id":"com.tencent.WeWorkMac"}},
        {**base,"step_id":"focus-search","attempt_id":"a1","tool_name":"hotkey","arguments":{"keys":["cmd","f"],"_yonder_action_kind":"focus-target-search","_yonder_private_text":"private-target"}},
        {**base,"step_id":"query","attempt_id":"a2","tool_name":"type_text","arguments":{"_yonder_action_kind":"enter-target-query","_yonder_private_text":"private-target"}},
        {**base,"step_id":"focus-composer","attempt_id":"a3","tool_name":"click","arguments":{"x":400,"y":500,"_yonder_action_kind":"focus-message-composer"}},
        {**base,"step_id":"draft","attempt_id":"a4","tool_name":"type_text","arguments":{"_yonder_action_kind":"draft-message-ref","_yonder_private_text":"private-message"}},
        {**base,"task_id":"other-task","step_id":"unverified-focus","attempt_id":"a5","tool_name":"hotkey","arguments":{"keys":["cmd","f"],"_yonder_action_kind":"focus-target-search"}},
        {**base,"task_id":"other-task","step_id":"untrusted","attempt_id":"a6","tool_name":"type_text","arguments":{"_yonder_action_kind":"enter-target-query","_yonder_private_text":"must-not-dispatch"}},
        {**base,"task_id":"atomic-task","step_id":"atomic-draft","attempt_id":"a7","tool_name":"type_text","arguments":{"x":400,"y":500,"_yonder_action_kind":"draft-message-ref","_yonder_private_text":"private-atomic"}},
    ]
    run = subprocess.run(["node",str(worker),str(sdk),str(evidence)],input="".join(json.dumps(item)+"\n" for item in requests),text=True,capture_output=True,timeout=10,check=True)
    responses = [json.loads(line) for line in run.stdout.splitlines()]
    actions = [json.loads(line) for line in log.read_text().splitlines()]

assert responses[0]["action_succeeded"] is True
assert responses[1]["action_succeeded"] is True and responses[3]["action_succeeded"] is True
assert all(responses[index]["action_succeeded"] is False and responses[index]["action_effect"] == "unverifiable" and responses[index]["observe_valid"] for index in [2,4,5]), responses
assert responses[6]["action_succeeded"] is False and responses[6]["action_effect"] == "refused"
assert responses[7]["action_succeeded"] is True and responses[7]["observe_valid"] is True
assert [item["name"] for item in actions] == ["hotkey","type_text","click","type_text","hotkey","type_text"]
assert actions[1]["args"]["text"] == "private-target" and actions[3]["args"]["text"] == "private-message"
assert all(item["args"]["target"] == {"kind":"window","pid":42,"window_id":7} for item in actions[:4])
assert all(item["args"]["target"] == {"kind":"window","pid":11,"window_id":12} for item in actions[4:])
assert all("x" not in item["args"] and "y" not in item["args"] for item in [actions[1],actions[3]])
assert actions[2]["args"]["delivery_mode"] == "foreground"
assert actions[0]["args"]["delivery_mode"] == "foreground" and actions[4]["args"]["delivery_mode"] == "foreground"
assert actions[5]["args"]["delivery_mode"] == "foreground" and actions[5]["args"]["text"] == "private-atomic"
assert all("_yonder_private_text" not in item["args"] and "_yonder_action_kind" not in item["args"] for item in actions)
print(json.dumps({"closed_search_shortcut":True,"confirmed_focus_only":True,"unverifiable_focus_refused":True,"exact_window_target":True,"foreground_coordinates":True,"atomic_coordinate_text":True,"focus_consumed_once":True,"untrusted_input_refused":True}))
