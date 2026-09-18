"""将已编译的 E0 桌宠封装为本机预览 .app；不用于签名发布。"""
from pathlib import Path
import argparse
import plistlib
import shutil
import subprocess

root = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--task-space', action='store_true', help='封装正式任务面板的本机debug预览')
options = parser.parse_args()
binary = root.parent.parent / "target/debug/yonder-desktop" if options.task_space else root / "src-tauri/target/release/yonder-desktop-spike"
if not binary.is_file():
    raise SystemExit("请先编译桌宠 Release 版本")
bundle = root.parent.parent / "apps/desktop/target/preview/Yonda Task Space.app" if options.task_space else root / "src-tauri/target/release/bundle/macos/Yonda.app"
executable = bundle / "Contents/MacOS" / binary.name
executable.parent.mkdir(parents=True, exist_ok=True)
shutil.copy2(binary, executable)
if options.task_space:
    cli = root.parent.parent / "target/debug/yonder"
    if not cli.is_file():
        raise SystemExit("请先编译 yonder CLI")
    shutil.copy2(cli, executable.parent / "yonder")
    resources = bundle / "Contents/Resources/cua"
    resources.mkdir(parents=True, exist_ok=True)
    node = shutil.which("node")
    if not node:
        raise SystemExit("缺少Node Runtime，无法封装CUA SDK")
    shutil.copy2(node, resources / "node")
    shutil.copy2(root.parent.parent / "crates/adapters/src/cua_worker.mjs", resources / "cua_worker.mjs")
    modules = root / "../cua-driver-comparison/node_modules"
    for package in ["@trycua/cua-driver", "@trycua/cua-driver-darwin-arm64", "@ubjs/core", "@ubjs/node", "@ubjs/node-darwin-arm64"]:
        shutil.copytree(modules / package, resources / "node_modules" / package, dirs_exist_ok=True)
bundle_identifier = "com.yonder.desktop" if options.task_space else "com.yonder.e0-spike"
with (bundle / "Contents/Info.plist").open("wb") as output:
    plistlib.dump({
        "CFBundleExecutable": binary.name,
        "CFBundleIdentifier": bundle_identifier,
        "CFBundleName": "Yonda",
        "CFBundleDisplayName": "Yonda",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": "0.0.0",
        "CFBundleVersion": "1",
        "NSHighResolutionCapable": True,
        "NSPrincipalClass": "NSApplication",
        "LSUIElement": True,
        "NSMicrophoneUsageDescription": "仅在你点击小龙旁的语音输入后采集本次语音并转成文字。",
        "NSSpeechRecognitionUsageDescription": "将你主动录制的语音转换为文字。",
    }, output)
subprocess.run([
    "/usr/bin/codesign", "--force", "--deep", "--sign", "-",
    "--requirements", f'=designated => identifier "{bundle_identifier}"',
    bundle,
], check=True)
print(bundle)
