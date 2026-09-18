"""测量正常.app启动至既有渲染报告；不声称冷缓存、首帧或完整交互就绪。"""
import json
import hashlib
from pathlib import Path
import subprocess
import sys
import time

root = Path(__file__).resolve().parent
output = Path(sys.argv[1]).resolve()
assert not output.exists(), '证据已存在，不覆盖'
existing = subprocess.run(['pgrep', '-x', 'yonder-desktop-spike'], capture_output=True)
assert existing.returncode == 1, '请先退出已有桌宠，或检查进程查询错误'
binary = root / 'src-tauri/target/release/bundle/macos/Yonda.app/Contents/MacOS/yonder-desktop-spike'
bundle = binary.parent.parent.parent
assert binary.is_file(), '缺少已打包程序'
binary_sha256 = hashlib.sha256(binary.read_bytes()).hexdigest()
output.parent.mkdir(parents=True, exist_ok=True)
log = output.with_suffix('.log')
with log.open('x') as stream:
    started = time.monotonic()
    launch = subprocess.run(['open', '-n', '--stdout', str(log), '--stderr', str(log), str(bundle)],
                            capture_output=True, timeout=10)
    launch_returned = time.monotonic()
    assert launch.returncode == 0, 'LaunchServices启动失败'
    result = {'scope': 'LaunchServices .app启动至既有渲染诊断报告，含页面加载后300ms等待及20ms轮询；非冷缓存或完整交互就绪',
              'binary_sha256': binary_sha256, 'ready': False,
              'launch_return_ms': (launch_returned - started) * 1000,
              'process_query_total_ms': 0, 'process_query_max_ms': 0, 'poll_count': 0}
    while time.monotonic() - started < 10:
        query_started = time.monotonic()
        processes = subprocess.run(['pgrep', '-x', 'yonder-desktop-spike'], capture_output=True, text=True, timeout=2)
        query_ms = (time.monotonic() - query_started) * 1000
        result['process_query_total_ms'] += query_ms
        result['process_query_max_ms'] = max(result['process_query_max_ms'], query_ms)
        result['poll_count'] += 1
        assert processes.returncode == 0, '桌宠提前退出'
        pids = processes.stdout.split()
        assert len(pids) == 1, '未取得唯一桌宠宿主'
        result['pid'] = int(pids[0])
        content = log.read_text()
        observed = time.monotonic()
        if '桌宠启动：' in content and 'startup_log_observed_ms' not in result:
            result['startup_log_observed_ms'] = (observed - started) * 1000
        if '桌宠页面加载：Finished' in content and 'page_finished_observed_ms' not in result:
            result['page_finished_observed_ms'] = (observed - started) * 1000
        assert len(content) < 65536, '诊断日志异常增长'
        reports = [line for line in content.splitlines() if line.startswith('桌宠渲染：')]
        if reports:
            result['report_latency_ms'] = (observed - started) * 1000
            result['report_after_launch_return_ms'] = (observed - launch_returned) * 1000
            fields = reports[-1].removeprefix('桌宠渲染：').split(', ')
            result['ready'] = all(field in fields for field in (
                'ready=true', 'image_width=600', 'style_loaded=true', 'script_ready=true'))
            break
        time.sleep(.02)
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2))
    print(json.dumps(result, ensure_ascii=False))
    assert result['ready'], '未取得有效就绪报告，不能判启动通过'
    # 当前阶段按用户变更以功能就绪验收；3秒保留为优化目标。
