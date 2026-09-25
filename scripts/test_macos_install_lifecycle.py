import shutil
from pathlib import Path
import tempfile
import unittest

from verify_macos_install_lifecycle import (
    LifecycleError,
    install_application,
    replace_application,
    uninstall_application,
)


def copy_fixture(source, destination):
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(source, destination)


def verify_fixture(bundle):
    if not (bundle / "marker").is_file():
        raise LifecycleError("fixture无效")


class MacosInstallLifecycleTest(unittest.TestCase):
    def app(self, root, name, value):
        bundle = root / name
        bundle.mkdir(parents=True)
        (bundle / "marker").write_text(value, encoding="utf-8")
        return bundle

    def test_fresh_install_and_recoverable_uninstall(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = self.app(root, "candidate.app", "new")
            installed = root / "Applications/Yonda.app"
            install_application(source, installed, copy_fixture, verify_fixture)
            self.assertEqual((installed / "marker").read_text(encoding="utf-8"), "new")
            trash = root / "Trash/Yonda.app"
            uninstall_application(installed, trash)
            self.assertFalse(installed.exists())
            self.assertEqual((trash / "marker").read_text(encoding="utf-8"), "new")

    def test_upgrade_keeps_a_previous_copy(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            previous = self.app(root, "previous.app", "old")
            candidate = self.app(root, "candidate.app", "new")
            installed = root / "Applications/Yonda.app"
            install_application(previous, installed, copy_fixture, verify_fixture)
            backup, _, _ = replace_application(
                installed, candidate, copy_fixture, verify_fixture
            )
            self.assertEqual((installed / "marker").read_text(encoding="utf-8"), "new")
            self.assertEqual((backup / "marker").read_text(encoding="utf-8"), "old")

    def test_injected_failures_before_and_after_placement_restore_previous_application(self):
        for failure_stage in ["before-placement", "after-placement"]:
            with self.subTest(failure_stage=failure_stage), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                previous = self.app(root, "previous.app", "old")
                candidate = self.app(root, "candidate.app", "new")
                installed = root / "Applications/Yonda.app"
                install_application(previous, installed, copy_fixture, verify_fixture)
                with self.assertRaisesRegex(LifecycleError, "注入候选落位[前后]失败"):
                    replace_application(
                        installed,
                        candidate,
                        copy_fixture,
                        verify_fixture,
                        failure_stage=failure_stage,
                    )
                self.assertEqual((installed / "marker").read_text(encoding="utf-8"), "old")


if __name__ == "__main__":
    unittest.main()
