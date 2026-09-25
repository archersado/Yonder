"""把独立标识的调试二进制封装成不接触正式数据目录的原生验证 App。"""
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys

desktop = Path(__file__).resolve().parent
root = desktop.parent.parent
source = root / "target/debug/Yonda.app"
legacy_source = desktop / "target/preview/Yonda Task Space.app"
if not source.is_dir() and legacy_source.is_dir():
    source = legacy_source
variant = sys.argv[1] if len(sys.argv) == 2 else "observation"
if variant not in ("observation", "control", "focus", "creation", "attempt-start", "voice-session"):
    raise SystemExit("仅允许 observation、control、focus、creation、attempt-start 或 voice-session 隔离夹具")
label = {"observation": "Observe", "control": "Control", "focus": "Focus", "creation": "Creation", "attempt-start": "Attempt Start", "voice-session": "Voice Session"}[variant]
bundle = desktop / f"target/preview/Yonda {label} Fixture.app"
binary = root / "target/debug/yonder-desktop"
identifier = f"com.yonder.{variant}.fixture"
if not source.is_dir() or not binary.is_file() or bundle.exists():
    raise SystemExit("缺少预览包或隔离二进制，或测试包已存在；拒绝覆盖")
shutil.copytree(source, bundle)
shutil.copy2(binary, bundle / "Contents/MacOS/yonder-desktop")
plist_path = bundle / "Contents/Info.plist"
with plist_path.open("rb") as stream:
    plist = plistlib.load(stream)
plist["CFBundleIdentifier"] = identifier
plist["CFBundleName"] = f"Yonda {label} Fixture"
plist["CFBundleDisplayName"] = f"Yonda {label} Fixture"
with plist_path.open("wb") as stream:
    plistlib.dump(plist, stream)
subprocess.run(["/usr/bin/codesign", "--force", "--deep", "--sign", "-", str(bundle)], check=True)
print(bundle)
