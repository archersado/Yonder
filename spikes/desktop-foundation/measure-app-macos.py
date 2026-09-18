"""按系统资源分组采集 Yonda 全应用五分钟预算；只保存数值和归属，不保存进程域正文。"""
import ctypes
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
import time


def coalition(text):
    block = re.search(r'resource coalition = \{([^{}]*)\}', text)
    if not block:
        raise ValueError('无法读取资源分组')
    fields = dict(re.findall(r'^\s*(ID|name|bundle ID) = (.+)$', block[1], re.M))
    return fields['ID'], fields['name'], fields.get('bundle ID')


class Usage(ctypes.Structure):
    # 本机 SDK sys/resource.h 的 rusage_info_v0；CPU 时间为纳秒。
    _fields_ = [('uuid', ctypes.c_uint8 * 16)] + [(name, ctypes.c_uint64) for name in (
        'user', 'system', 'idle_wakeups', 'interrupt_wakeups', 'pageins',
        'wired', 'resident', 'footprint', 'started', 'exited')]


def main():
    if sys.argv[1:] == ['--self-test']:
        assert ctypes.sizeof(Usage) == 96
        assert coalition('resource coalition = {\n ID = 1\n name = app\n bundle ID = test\n}\njetsam coalition = {\n ID = 9\n}') == ('1', 'app', 'test')
        try:
            coalition('jetsam coalition = {\n ID = 1\n}')
        except ValueError:
            pass
        else:
            raise AssertionError('缺少资源分组不能降级为猜测')
        print('结构与归属解析检查通过')
        return
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('host', type=int)
    parser.add_argument('output', type=Path)
    parser.add_argument('--awake-probe', type=Path, help='已编译的 awake-probe-macos.swift；每120秒触发托盘找回')
    parser.add_argument('--external-frames', action='store_true', help='由独立原生探针留存截图，采样器仍核对窗口状态')
    args = parser.parse_args()
    host, output = args.host, args.output
    if output.exists():
        raise ValueError('证据文件已存在，不覆盖')
    output.parent.mkdir(parents=True, exist_ok=True)
    lib = ctypes.CDLL('/usr/lib/libproc.dylib', use_errno=True)
    lib.proc_pid_rusage.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_void_p]
    lib.proc_pid_rusage.restype = ctypes.c_int
    def usage(pid):
        value = Usage()
        if lib.proc_pid_rusage(pid, 0, ctypes.byref(value)) != 0:
            raise OSError(ctypes.get_errno(), '进程资源不可读')
        return {'start': value.started, 'cpu_ns': value.user + value.system,
                'resident_bytes': value.resident, 'footprint_bytes': value.footprint}
    def group(pid):
        return coalition(subprocess.check_output(['/bin/launchctl', 'print', f'pid/{pid}'], text=True, stderr=subprocess.PIPE))
    expected = group(host)
    if expected[2] != 'com.yonder.e0-spike':
        raise ValueError('不是 Yonda 资源分组')
    identities = {}
    def members():
        raw = subprocess.check_output(['ps', '-A', '-o', 'pid=,comm='], text=True)
        selected = {}
        for line in raw.splitlines():
            pid_text, name = line.strip().split(maxsplit=1)
            pid = int(pid_text)
            if pid != host and '/WebKit.framework/' not in name:
                continue
            if pid == host and not name.endswith('/yonder-desktop-spike'):
                raise ValueError('宿主进程已被替换')
            stats = usage(pid)
            identity = (pid, stats['start'])
            if identity not in identities:
                identities[identity] = group(pid)
            if identities[identity] == expected:
                selected[pid] = {'role': 'host' if pid == host else name.rsplit('/', 1)[-1], **stats}
        if host not in selected:
            raise ValueError('宿主已退出')
        return selected
    samples = []
    report = {'scope': '宿主及同一系统 resource coalition 的 WebKit 进程',
              'scenario': '五分钟保持展开，每120秒触发托盘找回；包含该交互开销' if args.awake_probe else '五分钟自然闲置，不触发交互或重置三分钟计时',
              'coalition': expected, 'samples': samples, 'status': '采集中'}
    report['frames_external'] = args.external_frames
    def probe(mode, frame=None):
        command = [str(args.awake_probe.resolve()), str(host), mode]
        if frame:
            command.append(str(frame))
        return json.loads(subprocess.check_output(command, text=True, timeout=10))
    try:
        if args.awake_probe:
            report['initial_wake'] = probe('wake')
            output.with_suffix('.frames').mkdir(exist_ok=True)
        initial = members()
        baseline = {pid: item['start'] for pid, item in initial.items()}
        if not any(item['role'] == 'com.apple.WebKit.WebContent' for item in initial.values()):
            raise ValueError('未确认 WebContent 归属，不进行宿主单独采样')
        started = time.monotonic()
        for index in range(61):
            window = None
            if args.awake_probe:
                frame = output.with_suffix('.frames') / f'{index:02}.png' if not args.external_frames and index in (0, 1, 30, 60) else None
                window = probe('wake' if index in (24, 48) else 'check', frame)
            current = initial if index == 0 and not args.awake_probe else members()
            if {pid: item['start'] for pid, item in current.items()} != baseline:
                raise ValueError('进程成员或启动身份变化，本轮预算不完整')
            if index == 0:
                initial = current
            samples.append({'elapsed': time.monotonic() - started, 'processes': current, **({'window': window} if window else {})})
            output.write_text(json.dumps(report, ensure_ascii=False, indent=2))
            if index % 6 == 0:
                print(json.dumps({'samples': len(samples), 'elapsed': round(samples[-1]['elapsed']),
                                  'processes': len(current)}, ensure_ascii=False), flush=True)
            if index < 60:
                time.sleep(max(0, started + (index + 1) * 5 - time.monotonic()))
        elapsed = samples[-1]['elapsed'] - samples[0]['elapsed']
        cpu_ns = sum(samples[-1]['processes'][pid]['cpu_ns'] - initial[pid]['cpu_ns'] for pid in baseline)
        footprints = [sum(p['footprint_bytes'] for p in s['processes'].values()) / 1048576 for s in samples]
        report['summary'] = {'duration_seconds': elapsed, 'cpu_percent_one_core': cpu_ns / 1e9 / elapsed * 100,
                             'mean_sum_footprint_mib': sum(footprints) / len(footprints), 'peak_sum_footprint_mib': max(footprints)}
        report['status'] = '采样完成；分进程 footprint 相加，不是系统去重后的组内存'
        print(json.dumps(report['summary'], ensure_ascii=False), flush=True)
    except Exception as error:
        report['status'] = '采样不完整'
        report['error'] = type(error).__name__ + ': ' + str(error).splitlines()[0]
        raise
    finally:
        output.write_text(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
