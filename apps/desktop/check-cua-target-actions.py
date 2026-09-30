#!/usr/bin/env python3
"""隔离验证 CUA 敏感引用展开后的封闭元素动作序列。"""
import json
import pathlib
import subprocess
import tempfile

root = pathlib.Path(__file__).resolve().parents[2]
worker = root / "crates/adapters/src/cua_worker.mjs"

sdk_source = r'''
import { appendFile } from 'node:fs/promises';
let stage = 0;
const logPath = __LOG_PATH__;
const elements = () => stage < 2
  ? [{role:'searchfield',label:'搜索',enabled:true,element_token:'search'}]
  : stage < 3
    ? [{role:'searchfield',label:'搜索',enabled:true,element_token:'search'},{role:'button',label:'宫健的分身',enabled:true,element_token:'contact'}]
    : [{role:'textfield',label:'发送消息',enabled:true,element_token:'composer'},{role:'button',label:'发送',enabled:true,element_token:'send'}];
export class CuaDriver {
  static create() { return {
    async metadata(){},
    async listToolsJson(){return JSON.stringify({tools:[
      {name:'launch_app',inputSchema:{properties:{bundle_id:{}}}},
      {name:'click',inputSchema:{properties:{pid:{},window_id:{},element_token:{}}}},
      {name:'type_text',inputSchema:{properties:{pid:{},window_id:{},element_token:{},text:{}}}},
      {name:'press_key',inputSchema:{properties:{pid:{},window_id:{},key:{}}}},
      {name:'get_window_state',inputSchema:{properties:{pid:{},window_id:{},include_screenshot:{},max_elements:{}}}},
      {name:'get_desktop_state',inputSchema:{properties:{}}},{name:'list_apps',inputSchema:{properties:{}}},{name:'list_windows',inputSchema:{properties:{pid:{}}}}
    ]});},
    async callTool(name,encoded){const args=JSON.parse(encoded);const ok=value=>({isError:false,structuredJson:JSON.stringify(value)});
      if(name==='launch_app')return ok({pid:42,bundle_id:'com.tencent.WeWorkMac',launch_state:'process_running',windows:[{window_id:7,bounds:{width:800,height:600}}]});
      if(name==='list_apps')return ok({apps:[{running:true,bundle_id:'com.tencent.WeWorkMac',pid:42}]});
      if(name==='list_windows')return ok({windows:[{window_id:7,is_on_screen:true,on_current_space:true,bounds:{width:800,height:600}}]});
      if(name==='get_window_state')return ok({elements:elements()});if(name==='get_desktop_state')return ok({windows:[]});
      if(['click','type_text','press_key'].includes(name)){await appendFile(logPath,JSON.stringify({name,args})+'\n');if(name==='type_text'&&args.element_token==='search')stage=2;else if(name==='click'&&args.element_token==='contact')stage=3;else if(name==='type_text'&&args.element_token==='composer')stage=4;else if((name==='click'&&args.element_token==='send')||name==='press_key')stage=5;return ok({effect:'confirmed'});}
      throw new Error('unexpected tool');},async shutdown(){},uniffiDestroy(){}
  };}
}
'''

with tempfile.TemporaryDirectory(prefix="yonda-cua-target-") as temporary:
    directory = pathlib.Path(temporary)
    sdk = directory / "sdk.mjs"
    evidence = directory / "evidence"
    log = directory / "actions.jsonl"
    sdk.write_text(sdk_source.replace("__LOG_PATH__", json.dumps(str(log))))
    evidence.mkdir()
    base = {"task_id":"task","worker_instance_id":"worker","host_session_id":"host","pid":11,"window_id":12}
    requests = [
        {**base,"step_id":"launch","attempt_id":"a0","tool_name":"launch_app","arguments":{"bundle_id":"com.tencent.WeWorkMac"}},
        {**base,"step_id":"focus","attempt_id":"a1","tool_name":"click","arguments":{"_yonder_action_kind":"focus-target-search","_yonder_private_text":"宫健的分身"}},
        {**base,"step_id":"query","attempt_id":"a2","tool_name":"type_text","arguments":{"_yonder_action_kind":"enter-target-query","_yonder_private_text":"宫健的分身"}},
        {**base,"step_id":"activate","attempt_id":"a3","tool_name":"click","arguments":{"_yonder_action_kind":"activate-target","_yonder_private_text":"宫健的分身"}},
        {**base,"step_id":"focus-composer","attempt_id":"a4","tool_name":"click","arguments":{"_yonder_action_kind":"focus-message-composer"}},
        {**base,"step_id":"draft","attempt_id":"a5","tool_name":"type_text","arguments":{"_yonder_action_kind":"draft-message-ref","_yonder_private_text":"hi"}},
        {**base,"step_id":"send","attempt_id":"a6","tool_name":"click","arguments":{"_yonder_action_kind":"send-message"}},
    ]
    run = subprocess.run(["node",str(worker),str(sdk),str(evidence)],input="".join(json.dumps(item)+"\n" for item in requests),text=True,capture_output=True,timeout=10,check=True)
    responses = [json.loads(line) for line in run.stdout.splitlines()]
    actions = [json.loads(line) for line in log.read_text().splitlines()]

assert all(item["action_succeeded"] for item in responses)
assert [item["args"].get("element_token") for item in actions] == ["search","search","contact","composer","composer","send"]
assert actions[1]["args"]["text"] == "宫健的分身" and actions[4]["args"]["text"] == "hi"
assert all("_yonder_private_text" not in item["args"] and "_yonder_action_kind" not in item["args"] for item in actions)
print(json.dumps({"semantic_steps":6,"element_tokens":True,"private_fields_stripped":True,"sent_after_confirmation_dispatch":True},ensure_ascii=False))
