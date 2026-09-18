"""采集指定桌宠宿主五分钟 CPU/RSS；不把宿主预算误称为 WebKit 全进程预算。"""
import json
from pathlib import Path
import subprocess
import sys
import time

pid = int(sys.argv[1])
output = Path(sys.argv[2])
samples = []
started = time.monotonic()
for index in range(61):
    row = subprocess.check_output(["ps", "-p", str(pid), "-o", "time=,rss=,comm="], text=True).strip().split(maxsplit=2)
    assert len(row) == 3 and row[2].endswith("/yonder-desktop-spike"), "指定桌宠进程已退出或发生替换"
    cpu = 0.0
    for part in row[0].split(":"):
        cpu = cpu * 60 + float(part)
    samples.append({"elapsed": time.monotonic() - started, "cpu_seconds": cpu, "rss_kib": int(row[1])})
    output.write_text(json.dumps({"pid": pid, "scope": "仅桌宠宿主，不含 WebKit 辅助进程", "samples": samples}, ensure_ascii=False, indent=2))
    if index < 60:
        time.sleep(max(0, started + (index + 1) * 5 - time.monotonic()))
elapsed = samples[-1]["elapsed"] - samples[0]["elapsed"]
summary = {"duration_seconds": elapsed,
           "host_cpu_percent": 100 * (samples[-1]["cpu_seconds"] - samples[0]["cpu_seconds"]) / elapsed,
           "host_mean_rss_mib": sum(s["rss_kib"] for s in samples) / len(samples) / 1024,
           "host_peak_rss_mib": max(s["rss_kib"] for s in samples) / 1024}
report = json.loads(output.read_text())
report["summary"] = summary
output.write_text(json.dumps(report, ensure_ascii=False, indent=2))
print(json.dumps(summary))
