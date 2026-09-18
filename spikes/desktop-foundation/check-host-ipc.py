"""验证当前桌宠宿主确实持有UDS，CLI正常往返，错误连接不终止宿主服务。"""
import hashlib
import json
from pathlib import Path
import socket
import subprocess
import sys
import tempfile

host, output = int(sys.argv[1]), Path(sys.argv[2])
assert not output.exists(), '证据已存在，不覆盖'
root = Path(__file__).resolve().parent
endpoint = Path(tempfile.gettempdir()) / 'yonder-e0-desktop-foundation.sock'
name = subprocess.check_output(['ps', '-p', str(host), '-o', 'comm='], text=True).strip()
assert name.endswith('/Yonda.app/Contents/MacOS/yonder-desktop-spike'), '非目标预览宿主'
uds = subprocess.check_output(['lsof', '-nP', '-a', '-p', str(host), '-U'], text=True)
assert str(endpoint) in uds, '端点不属于桌宠宿主'
tcp = subprocess.run(['lsof', '-nP', '-a', '-p', str(host), '-iTCP'], capture_output=True, text=True)
assert tcp.returncode == 1 and not tcp.stdout and not tcp.stderr, '宿主TCP检查失败'
with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
    connection.settimeout(5)
    connection.connect(str(endpoint))
    connection.sendall(b'{"version":"invalid","payload":"probe"}\n')
    assert connection.recv(4096) == b'', '错误版本未拒绝'
for _ in range(2):
    result = subprocess.run([str(root / 'target/release/yonder-ipc-spike'), 'client', 'host-probe'], capture_output=True, text=True, timeout=5, check=True)
    assert json.loads(result.stdout) == {'version': '0.1', 'payload': 'host-probe'}
report = {'host_pid': host, 'uds_owned_by_host': True, 'host_tcp_sockets': 0,
          'rejected_bad_version': True, 'subsequent_echo_count': 2,
          'binary_sha256': hashlib.sha256(Path(name).read_bytes()).hexdigest(),
          'scope': 'macOS预览宿主，同用户；无正式Gateway/任务或跨用户拒绝验收'}
output.write_text(json.dumps(report, ensure_ascii=False, indent=2))
print(json.dumps(report, ensure_ascii=False))
