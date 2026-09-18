"""macOS/Unix 真实双进程往返检查；不代替生产 Gateway 或跨用户权限测试。"""
import json
from pathlib import Path
import subprocess
import socket
import sys
import os
import time
import tempfile

root = Path(__file__).resolve().parent
assert sys.argv[1:] in ([], ['--other-user']), '未知验证参数'
binary = root / "target/release/yonder-ipc-spike"
endpoint = Path(tempfile.gettempdir()) / "yonder-e0-desktop-foundation.sock"
assert not endpoint.exists(), "验证端点已存在，请先确认没有其他 Spike 在运行"
def stop_probe(server, identity):
    if server.poll() is None:
        server.terminate()
        server.wait(timeout=5)
        # 强制结束探针不走Rust析构；只清理本次捕获身份的测试端点。
        if identity and endpoint.exists():
            current = endpoint.lstat()
            if (current.st_dev, current.st_ino) == identity:
                endpoint.unlink()

server = subprocess.Popen([str(binary), "server"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
identity = None
try:
    deadline = time.monotonic() + 5
    while not endpoint.exists():
        assert server.poll() is None, "服务端提前退出"
        assert time.monotonic() < deadline, "端点建立超时"
        time.sleep(.02)
    current = endpoint.lstat()
    identity = (current.st_dev, current.st_ino)
    sockets = subprocess.run(["lsof", "-nP", "-a", "-p", str(server.pid), "-iTCP"], capture_output=True, text=True)
    assert sockets.returncode == 1 and not sockets.stdout and not sockets.stderr, "无法确认没有 TCP 套接字"
    if sys.argv[1:] == ['--other-user']:
        # 不创建账户、不改权限；仅让既有nobody账户尝试连接这个测试端点。
        probe = '''import errno,json,os,socket,sys
assert os.geteuid() != int(sys.argv[2])
with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as connection:
 connection.settimeout(3)
 try:
  connection.connect(sys.argv[1])
 except OSError as error:
  assert error.errno in (errno.EACCES,errno.EPERM), str(error)
  print(json.dumps({"other_uid":os.geteuid(),"denied_errno":error.errno}))
 else:
  raise AssertionError("其他用户不应连接成功")
'''
        denied = subprocess.run(['sudo', '-n', '-u', 'nobody', '/usr/bin/python3', '-c', probe,
                                 str(endpoint), str(os.geteuid())], capture_output=True, text=True, timeout=10)
        assert denied.returncode == 0, '跨用户探针未通过：' + denied.stderr.strip()
        print(denied.stdout.strip())
    client = subprocess.run([str(binary), "client", "macos-e0"], capture_output=True, text=True, timeout=5, check=True)
    assert json.loads(client.stdout) == {"version": "0.1", "payload": "macos-e0"}
    assert server.wait(timeout=5) == 0
    print(json.dumps({"echo": "通过", "server_exit": 0, "tcp_sockets": 0,
                      "endpoint_removed": not endpoint.exists(), "parent_mode": oct(endpoint.parent.stat().st_mode & 0o777)}, ensure_ascii=False))
finally:
    stop_probe(server, identity)

# 真实独立进程拒绝路径；固定无敏感内容样本，不将原始输入写入证据。
for name, request, expected_error in (
    ('错误版本', b'{"version":"invalid","payload":"probe"}\n', '协议版本不匹配'),
    ('无效JSON', b'not-json\n', 'expected'),
):
    assert not endpoint.exists(), '上个用例遗留端点，停止'
    server = subprocess.Popen([str(binary), 'server'], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    identity = None
    try:
        deadline = time.monotonic() + 5
        while not endpoint.exists():
            assert server.poll() is None, '服务端提前退出'
            assert time.monotonic() < deadline, '端点建立超时'
            time.sleep(.02)
        current = endpoint.lstat()
        identity = (current.st_dev, current.st_ino)
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.settimeout(5)
            client.connect(str(endpoint))
            client.sendall(request)
            assert client.recv(4096) == b'', '错误请求不得返回成功响应'
        stdout, stderr = server.communicate(timeout=5)
        assert server.returncode != 0 and expected_error in stderr.decode(), '未按预期拒绝请求'
        assert not stdout and not endpoint.exists(), '异常退出存在输出或端点残留'
        print(json.dumps({'case': name, 'rejected': True, 'exit': server.returncode,
                          'endpoint_removed': True}, ensure_ascii=False))
    finally:
        stop_probe(server, identity)
