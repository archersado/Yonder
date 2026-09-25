import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest import mock


SCRIPT = Path(__file__).resolve().parents[1] / "apps/desktop/package-macos-preview.py"
SPEC = importlib.util.spec_from_file_location("package_macos_preview", SCRIPT)
package_macos_preview = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(package_macos_preview)


class PackageBuildProvenanceTest(unittest.TestCase):
    def info(self, package, commit="a" * 40, profile="release"):
        return {
            "schema": 1,
            "package": package,
            "version": "0.1.0",
            "profile": profile,
            "commit": commit,
        }

    def test_build_provenance_records_both_binary_identities(self):
        with tempfile.TemporaryDirectory() as directory:
            desktop = Path(directory) / "yonder-desktop"
            cli = Path(directory) / "yonder"
            desktop.write_bytes(b"desktop")
            cli.write_bytes(b"cli")
            with mock.patch.object(
                package_macos_preview,
                "read_build_info",
                side_effect=[self.info("yonder-desktop"), self.info("yonder-cli")],
            ):
                result = package_macos_preview.build_provenance(
                    desktop, cli, "0.1.0", "release", "a" * 40
                )
            self.assertEqual(result["commit"], "a" * 40)
            self.assertEqual(result["artifacts"]["desktop"], {"package": "yonder-desktop"})
            self.assertEqual(result["artifacts"]["cli"], {"package": "yonder-cli"})

    def test_release_rejects_missing_or_stale_commit(self):
        with self.assertRaisesRegex(ValueError, "缺少release构建提交"):
            package_macos_preview.validate_build_info(
                self.info("yonder-desktop", commit="development"),
                "yonder-desktop",
                "0.1.0",
                "release",
            )
        with self.assertRaisesRegex(ValueError, "构建提交不一致"):
            package_macos_preview.validate_build_info(
                self.info("yonder-cli"),
                "yonder-cli",
                "0.1.0",
                "release",
                "b" * 40,
            )

    def test_release_rejects_debug_profile(self):
        with self.assertRaisesRegex(ValueError, "profile不一致"):
            package_macos_preview.validate_build_info(
                self.info("yonder-cli", profile="debug"),
                "yonder-cli",
                "0.1.0",
                "release",
            )

    def test_two_release_binaries_must_report_the_same_commit(self):
        with tempfile.TemporaryDirectory() as directory:
            desktop = Path(directory) / "yonder-desktop"
            cli = Path(directory) / "yonder"
            desktop.write_bytes(b"desktop")
            cli.write_bytes(b"cli")
            with mock.patch.object(
                package_macos_preview,
                "read_build_info",
                side_effect=[self.info("yonder-desktop", "a" * 40), self.info("yonder-cli", "b" * 40)],
            ):
                with self.assertRaisesRegex(ValueError, "desktop与CLI构建提交不一致"):
                    package_macos_preview.build_provenance(desktop, cli, "0.1.0", "release")


if __name__ == "__main__":
    unittest.main()
