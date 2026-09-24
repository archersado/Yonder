import unittest
from pathlib import Path
import tempfile

from release import collect_manifest, tree_sha256


class ReleaseManifestTest(unittest.TestCase):
    def test_repository_freezes_current_release_versions(self):
        manifest = collect_manifest(Path(__file__).resolve().parents[1], include_artifacts=False)
        self.assertEqual(manifest["version"], "0.1.0")
        self.assertEqual(manifest["protocol"], {"major": 1, "minor": 20})
        self.assertEqual(manifest["sqlite_schema"], 18)
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


if __name__ == "__main__":
    unittest.main()
