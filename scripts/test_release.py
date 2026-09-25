import unittest
from pathlib import Path
import plistlib
import tempfile

from release import audit_macos_bundle, collect_manifest, tree_sha256


class ReleaseManifestTest(unittest.TestCase):
    def test_repository_freezes_current_release_versions(self):
        manifest = collect_manifest(Path(__file__).resolve().parents[1], include_artifacts=False)
        self.assertEqual(manifest["version"], "0.1.0")
        self.assertEqual(manifest["protocol"], {"major": 1, "minor": 25})
        self.assertEqual(manifest["sqlite_schema"], 19)
        self.assertEqual({driver["kind"] for driver in manifest["drivers"]}, {"cua", "bua"})
        self.assertIn("release_contract", manifest["sources"])

    def test_workspace_version_mismatch_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text(
                'members = ["crates/one", "crates/two"]',
                encoding="utf-8",
            )
            (root / "crates").mkdir()
            (root / "crates/one").mkdir()
            (root / "crates/two").mkdir()
            (root / "crates/one/Cargo.toml").write_text(
                'name = "one"\nversion = "0.1.0"',
                encoding="utf-8",
            )
            (root / "crates/two/Cargo.toml").write_text(
                'name = "two"\nversion = "0.2.0"',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "Workspace 版本不一致"):
                collect_manifest(root, include_artifacts=False)

    def test_tree_hash_changes_when_a_bundled_file_changes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "Yonda.app"
            (root / "Contents/MacOS").mkdir(parents=True)
            executable = root / "Contents/MacOS/yonder-desktop"
            executable.write_bytes(b"old")
            before = tree_sha256(root)
            executable.write_bytes(b"new")
            self.assertNotEqual(before, tree_sha256(root))

    def make_bundle(self, directory):
        bundle = Path(directory) / "Yonda.app"
        text = {
            "Contents/Resources/channel.json": '{"channel":"dev"}\n',
            "Contents/Resources/driver-manifest.json": '{"version":"0.1.0"}\n',
            "Contents/Resources/release-contract.json": '{"version":"0.1.0"}\n',
            "Contents/Resources/cua/cua_worker.mjs": "export const worker = true;\n",
            "Contents/Resources/cua/jev_worker.mjs": "export const worker = true;\n",
        }
        binary = {
            "Contents/MacOS/yonder-desktop",
            "Contents/MacOS/yonder",
            "Contents/Resources/cua/node",
            "Contents/Resources/cua/node_modules/example/package.json",
            "Contents/_CodeSignature/CodeResources",
        }
        for relative, content in text.items():
            path = bundle / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        for relative in binary:
            path = bundle / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"fixture")
        info = bundle / "Contents/Info.plist"
        info.parent.mkdir(parents=True, exist_ok=True)
        with info.open("wb") as stream:
            plistlib.dump({"CFBundleIdentifier": "com.yonder.desktop"}, stream)
        return bundle

    def test_bundle_audit_accepts_allowlisted_release_contents(self):
        with tempfile.TemporaryDirectory() as directory:
            bundle = self.make_bundle(directory)
            result = audit_macos_bundle(bundle, expected_channel="dev")
            self.assertTrue(result["passed"])
            self.assertEqual(result["channel"], "dev")
            self.assertFalse(result["sensitive_values_recorded"])
            with self.assertRaisesRegex(ValueError, "发布通道不一致"):
                audit_macos_bundle(bundle, expected_channel="stable")

    def test_bundle_audit_rejects_user_data_and_secret_text_without_echoing_value(self):
        with tempfile.TemporaryDirectory() as directory:
            bundle = self.make_bundle(directory)
            database = bundle / "Contents/Resources/tasks.db"
            database.write_bytes(b"private task data")
            with self.assertRaisesRegex(ValueError, "未允许路径|敏感文件") as error:
                audit_macos_bundle(bundle)
            self.assertNotIn("private task data", str(error.exception))
            database.unlink()
            secret = "-----BEGIN PRIVATE KEY-----"
            (bundle / "Contents/Resources/cua/jev_worker.mjs").write_text(secret, encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "自有文本包含敏感内容") as error:
                audit_macos_bundle(bundle)
            self.assertNotIn(secret, str(error.exception))


if __name__ == "__main__":
    unittest.main()
