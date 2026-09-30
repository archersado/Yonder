#!/usr/bin/env python3
"""隔离验证通用桌面计划语义可连续映射到同一可信窗口。"""
import json
import pathlib
import subprocess
import tempfile

root = pathlib.Path(__file__).resolve().parents[2]
worker = root / "crates/adapters/src/cua_worker.mjs"

sdk_source = r'''
import { appendFile } from 'node:fs/promises';
const logPath=__LOG_PATH__;
let focused=false;
export class CuaDriver { static create(){return{
  async metadata(){},
  async listToolsJson(){return JSON.stringify({tools:[
    {name:'launch_app',inputSchema:{properties:{bundle_id:{}}}},
    {name:'hotkey',inputSchema:{properties:{target:{},keys:{},session:{}}}},
    {name:'type_text',inputSchema:{properties:{target:{},text:{},element_token:{},session:{}}}},
    {name:'press_key',inputSchema:{properties:{target:{},key:{},session:{}}}},
    {name:'get_window_state',inputSchema:{properties:{pid:{},window_id:{},include_screenshot:{},max_elements:{},session:{}}}},
    {name:'get_desktop_state',inputSchema:{properties:{}}},{name:'list_apps',inputSchema:{properties:{}}},{name:'list_windows',inputSchema:{properties:{pid:{}}}}
  ]});},
  async callTool(name,encoded){const args=JSON.parse(encoded);const ok=value=>({isError:false,structuredJson:JSON.stringify(value)});
    if(name==='launch_app')return ok({pid:42,bundle_id:'com.example.Media',launch_state:'process_running',windows:[{window_id:7,bounds:{width:800,height:600}}]});
    if(name==='list_apps')return ok({apps:[{running:true,bundle_id:'com.example.Media',pid:42}]});
    if(name==='list_windows')return ok({windows:[{window_id:7,is_on_screen:true,on_current_space:true,bounds:{width:800,height:600}}]});
    if(name==='get_desktop_state')return ok({windows:[]});
    if(name==='get_window_state')return ok({elements:focused?[]:[{role:'searchfield',label:'搜索',enabled:true,element_token:'search'}]});
    if(['hotkey','type_text','press_key'].includes(name)){await appendFile(logPath,JSON.stringify({name,args})+'\n');if(name==='hotkey')focused=true;return ok({effect:'confirmed'});}
    throw new Error('unexpected tool');
  },async shutdown(){},uniffiDestroy(){}
};}}
'''

with tempfile.TemporaryDirectory(prefix="yonda-cua-generic-plan-") as temporary:
    directory=pathlib.Path(temporary); sdk=directory/"sdk.mjs"; evidence=directory/"evidence"; log=directory/"actions.jsonl"
    sdk.write_text(sdk_source.replace("__LOG_PATH__",json.dumps(str(log)))); evidence.mkdir()
    base={"task_id":"task","worker_instance_id":"worker","host_session_id":"host","pid":11,"window_id":12}
    requests=[
      {**base,"step_id":"launch","attempt_id":"a0","tool_name":"launch_app","arguments":{"bundle_id":"com.example.Media"}},
      {**base,"step_id":"focus","attempt_id":"a1","tool_name":"hotkey","arguments":{"keys":["cmd","f"],"_yonder_action_kind":"focus-control"}},
      {**base,"step_id":"input","attempt_id":"a2","tool_name":"type_text","arguments":{"text":"one last kiss","_yonder_action_kind":"input-text"}},
      {**base,"step_id":"activate","attempt_id":"a3","tool_name":"press_key","arguments":{"key":"ENTER","_yonder_action_kind":"activate-control"}},
    ]
    run=subprocess.run(["node",str(worker),str(sdk),str(evidence)],input="".join(json.dumps(item)+"\n" for item in requests),text=True,capture_output=True,timeout=10,check=True)
    responses=[json.loads(line) for line in run.stdout.splitlines()]; actions=[json.loads(line) for line in log.read_text().splitlines()]
    assert all(item["action_succeeded"] is True for item in responses)
    assert [item["name"] for item in actions]==["hotkey","type_text","press_key"]
    assert all(item["args"]["target"]=={"kind":"window","pid":42,"window_id":7} for item in actions)
    assert actions[1]["args"]["text"]=="one last kiss" and "element_token" not in actions[1]["args"]
    assert all("_yonder_action_kind" not in item["args"] for item in actions)

print(json.dumps({"generic_plan_actions":True,"same_trusted_window":True,"confirmed_focus_handoff":True,"plan_text_bounded":True,"private_intent_not_required":True}))
