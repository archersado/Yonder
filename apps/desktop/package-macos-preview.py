"""将正式桌面宿主及固定版本 CUA SDK 封装为本机预览 .app。"""
from pathlib import Path
import plistlib
import shutil
import subprocess

desktop = Path(__file__).resolve().parent
root = desktop.parent.parent
binary = root / "target/debug/yonder-desktop"
cli = root / "target/debug/yonder"
modules = desktop / "cua/node_modules"
bundle = desktop / "target/preview/Yonda Task Space.app"
if not binary.is_file() or not cli.is_file():
    raise SystemExit("请先编译 yonder-desktop 与 yonder CLI")
if not (modules / "@trycua/cua-driver/dist/index.js").is_file():
    raise SystemExit("请先在 apps/desktop/cua 执行 npm ci --ignore-scripts")

executable = bundle / "Contents/MacOS/yonder-desktop"
executable.parent.mkdir(parents=True, exist_ok=True)
shutil.copy2(binary, executable)
shutil.copy2(cli, executable.parent / "yonder")
resources = bundle / "Contents/Resources/cua"
if resources.exists():
    shutil.rmtree(resources)
resources.mkdir(parents=True)
node = shutil.which("node")
if not node:
    raise SystemExit("缺少 Node Runtime，无法封装 CUA SDK")
shutil.copy2(node, resources / "node")
shutil.copy2(root / "crates/adapters/src/cua_worker.mjs", resources / "cua_worker.mjs")
for package in ["@trycua/cua-driver", "@trycua/cua-driver-darwin-arm64", "@ubjs/core", "@ubjs/node", "@ubjs/node-darwin-arm64"]:
    shutil.copytree(modules / package, resources / "node_modules" / package)

with (bundle / "Contents/Info.plist").open("wb") as output:
    plistlib.dump({
        "CFBundleExecutable": "yonder-desktop",
        "CFBundleIdentifier": "com.yonder.desktop",
        "CFBundleName": "Yonda",
        "CFBundleDisplayName": "Yonda",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": "0.1.0",
        "CFBundleVersion": "1",
        "NSHighResolutionCapable": True,
        "NSPrincipalClass": "NSApplication",
        "LSUIElement": True,
        "NSMicrophoneUsageDescription": "仅在你点击小龙旁的语音输入后采集本次语音并转成文字。",
        "NSSpeechRecognitionUsageDescription": "将你主动录制的语音转换为文字。",
        "NSScreenCaptureUsageDescription": "仅在你主动使用圈选提问时截取所选区域，并只在本机内存中预览。",
    }, output)
subprocess.run(["/usr/bin/codesign", "--force", "--deep", "--sign", "-", bundle], check=True)
print(bundle)
