"""将正式桌面宿主及固定版本 CUA SDK 封装为 macOS .app。"""
from pathlib import Path
import argparse
import plistlib
import shutil
import subprocess

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release", action="store_true", help="使用 release 产物并输出到 target/release")
    parser.add_argument("--identity", default="-", help="codesign 身份；默认使用临时签名")
    parser.add_argument("--allow-adhoc", action="store_true", help="仅本机验证时允许临时签名")
    parser.add_argument("--channel", choices=["dev", "stable"], default="dev", help="发布通道")
    parser.add_argument("--notary-profile", help="Apple Notary Keychain Profile；提供后执行公证")
    args = parser.parse_args()

    desktop = Path(__file__).resolve().parent
    root = desktop.parent.parent
    profile = "release" if args.release else "debug"
    binary = root / f"target/{profile}/yonder-desktop"
    cli = root / f"target/{profile}/yonder"
    modules = desktop / "cua/node_modules"
    bundle = root / f"target/{profile}/Yonda.app"
    if not binary.is_file() or not cli.is_file():
        raise SystemExit("请先编译 yonder-desktop 与 yonder CLI")
    if not (modules / "@trycua/cua-driver/dist/index.js").is_file():
        raise SystemExit("请先在 apps/desktop/cua 执行 npm ci --ignore-scripts")
    if args.release and args.identity == "-" and not args.allow_adhoc:
        raise SystemExit("release 产物必须提供正式 codesign 身份")
    if args.notary_profile and (args.identity == "-" or args.allow_adhoc):
        raise SystemExit("公证产物必须使用正式 codesign 身份")

    executable = bundle / "Contents/MacOS/yonder-desktop"
    if bundle.exists():
        shutil.rmtree(bundle)
    executable.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(binary, executable)
    shutil.copy2(cli, executable.parent / "yonder")
    resources = bundle / "Contents/Resources/cua"
    resources.mkdir(parents=True)
    node = shutil.which("node")
    if not node:
        raise SystemExit("缺少 Node Runtime，无法封装 CUA SDK")
    shutil.copy2(node, resources / "node")
    shutil.copy2(root / "crates/adapters/src/cua_worker.mjs", resources / "cua_worker.mjs")
    shutil.copy2(root / "crates/adapters/src/jev_worker.mjs", resources / "jev_worker.mjs")
    for package in ["@trycua/cua-driver", "@trycua/cua-driver-darwin-arm64", "@ubjs/core", "@ubjs/node", "@ubjs/node-darwin-arm64", "@typesafe-ai/sdk"]:
        shutil.copytree(modules / package, resources / "node_modules" / package)
    shutil.copy2(desktop / "release-contract.json", bundle / "Contents/Resources/release-contract.json")
    shutil.copy2(desktop / "driver-manifest.json", bundle / "Contents/Resources/driver-manifest.json")
    (bundle / "Contents/Resources/channel.json").write_text(
        f'{{"channel":"{args.channel}"}}\n', encoding="utf-8"
    )

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
    subprocess.run(["/usr/bin/codesign", "--force", "--deep", "--sign", args.identity, bundle], check=True)
    subprocess.run(["/usr/bin/codesign", "--verify", "--deep", "--strict", bundle], check=True)
    if args.notary_profile:
        archive = bundle.with_suffix(".zip")
        subprocess.run(
            ["/usr/bin/ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", bundle, archive],
            check=True,
        )
        subprocess.run(
            ["xcrun", "notarytool", "submit", archive, "--keychain-profile", args.notary_profile, "--wait"],
            check=True,
        )
        subprocess.run(["xcrun", "stapler", "staple", bundle], check=True)
        subprocess.run(["/usr/sbin/spctl", "--assess", "--type", "execute", bundle], check=True)
    print(bundle)


if __name__ == "__main__":
    main()
