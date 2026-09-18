"""限定Yonda资源组，通过现有托盘退出后检查宿主及已归属WebKit进程是否清理。"""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import time

host, output, probe = int(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]).resolve()
assert not output.exists(), '证据已存在，不覆盖'
spec = importlib.util.spec_from_file_location('budget', Path(__file__).with_name('measure-app-macos.py'))
budget = importlib.util.module_from_spec(spec)
spec.loader.exec_module(budget)

def group(pid):
    return budget.coalition(subprocess.check_output(['/bin/launchctl', 'print', f'pid/{pid}'], text=True, stderr=subprocess.PIPE))

expected = group(host)
assert expected[2] == 'com.yonder.e0-spike', '非目标应用'
members = {}
for line in subprocess.check_output(['ps', '-A', '-o', 'pid=,comm='], text=True).splitlines():
    pid_text, name = line.strip().split(maxsplit=1)
    pid = int(pid_text)
    if pid == host or '/WebKit.framework/' in name:
        if group(pid) == expected:
            members[pid] = name.rsplit('/', 1)[-1]
assert members.get(host) == 'yonder-desktop-spike', '宿主身份不匹配'
assert 'com.apple.WebKit.WebContent' in members.values(), '未确认渲染进程归属'
report = {'coalition': expected, 'members_before': members, 'status': '检查中'}
output.parent.mkdir(parents=True, exist_ok=True)
try:
    report['menu'] = json.loads(subprocess.check_output([str(probe), str(host), 'quit'], text=True, timeout=10))
    started = time.monotonic()
    remaining = set(members)
    while remaining and time.monotonic() - started < 30:
        live = {int(p) for p in subprocess.check_output(['ps', '-A', '-o', 'pid='], text=True).split()}
        remaining = set(members) & live
        if remaining:
            time.sleep(.5)
    report.update(elapsed_seconds=time.monotonic() - started, remaining_pids=sorted(remaining))
    assert not remaining, '30秒后仍存在已归属进程，不能判清理通过'
    report['status'] = '托盘退出后已归属进程全部结束'
finally:
    output.write_text(json.dumps(report, ensure_ascii=False, indent=2))
    print(json.dumps(report, ensure_ascii=False))
