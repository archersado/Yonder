"""使用合成 Cargo 元数据和 PR 事件验证门禁拒绝行为。"""

import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch

from check_architecture import DOCUMENTS, cargo_executable, check_dependencies, check_desktop_verification_layout, check_frontend_single_source, check_planning, check_pr, check_protocol_single_source, repository_file


def metadata(name, dependencies):
    return {"workspace_members": [name], "packages": [
        {"id": name, "name": name, "dependencies": dependencies}
    ]}


class DependencyTests(unittest.TestCase):
    def test_cargo_path_prefers_explicit_then_path_then_rustup(self):
        with patch.dict("check_architecture.os.environ", {"CARGO": "/custom/cargo"}, clear=True):
            self.assertEqual(cargo_executable(), "/custom/cargo")
        with patch.dict("check_architecture.os.environ", {}, clear=True), patch("check_architecture.shutil.which", return_value="/usr/bin/cargo"):
            self.assertEqual(cargo_executable(), "/usr/bin/cargo")
        with patch.dict("check_architecture.os.environ", {}, clear=True), patch("check_architecture.shutil.which", return_value=None), patch("check_architecture.Path.home", return_value=Path("/home/test")):
            self.assertEqual(cargo_executable(), "/home/test/.cargo/bin/cargo")

    def test_existing_directions(self):
        for name, targets in {
            "yonder-domain": [],
            "yonder-application": ["yonder-domain", "yonder-protocol"],
            "yonder-adapters": ["yonder-application", "yonder-protocol", "rusqlite"],
            "yonder-protocol": ["serde", "schemars", "ts-rs"],
        }.items():
            check_dependencies(metadata(name, [{"name": t} for t in targets]))

    def test_alias_build_dev_and_platform_cannot_bypass(self):
        for kind in (None, "dev", "build"):
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                check_dependencies(metadata("yonder-application", [{
                    "name": "yonder-adapters", "rename": "innocent",
                    "kind": kind, "target": "cfg(windows)",
                }]))

    def test_domain_external_dependency_rejected(self):
        with self.assertRaises(ValueError):
            check_dependencies(metadata("yonder-domain", [{"name": "serde"}]))

    def test_unknown_module_and_unregistered_local_dependency(self):
        with self.assertRaises(ValueError):
            check_dependencies(metadata("new-module", []))
        with self.assertRaises(ValueError):
            check_dependencies(metadata("yonder-application", [{
                "name": "hidden-adapter", "path": "/tmp/hidden-adapter",
            }]))


class ProtocolSingleSourceTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        files = {
            "crates/protocol/src/lib.rs": "pub const PROTOCOL_VERSION: u16 = 49;\npub fn valid_id(value: &str) -> bool { !value.is_empty() }\n",
            "crates/application/src/unknown_reason.rs": "pub enum UnknownReason { Invalid }\nimpl From<UnknownReason> for yonder_protocol::AttemptUnknownReason {}\n",
            "apps/yonder-cli/src/codex_agent_bridge.rs": "// 有意钉定 1.51\nlet version = ProtocolVersion { major: PROTOCOL_VERSION.major, minor: 51 };\nlet capability = OfferedCapability::UserInputReply;\n",
        }
        for name, content in files.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")

    def test_accepts_single_sources_and_documented_bridge_pin(self):
        check_protocol_single_source(self.root)

    def test_rejects_duplicate_id_version_reason_and_floating_bridge(self):
        cases = {
            "crates/application/src/runtime.rs": "fn valid_id(value: &str) -> bool { true }\n",
            "crates/application/src/version.rs": "pub const PROTOCOL_VERSION: u16 = 50;\n",
            "crates/application/src/reason.rs": "pub enum UnknownReason { Other }\n",
        }
        for name, content in cases.items():
            with self.subTest(name=name):
                path = self.root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8")
                with self.assertRaises(ValueError):
                    check_protocol_single_source(self.root)
                path.unlink()
        bridge = self.root / "apps/yonder-cli/src/codex_agent_bridge.rs"
        bridge.write_text("let version = ProtocolVersion { major: PROTOCOL_VERSION.major, minor: PROTOCOL_VERSION.minor };\n", encoding="utf-8")
        with self.assertRaises(ValueError):
            check_protocol_single_source(self.root)


class FrontendSingleSourceTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        generated = self.root / "crates/protocol/generated/protocol.ts"
        generated.parent.mkdir(parents=True)
        generated.write_text("\n".join([
            'export type TaskStatus = "created" | "running" | "completed";',
            'export type TaskSource = "local-agent" | "cloud-agent";',
            'export type TaskObservationResult = "matched" | "not-matched" | "unknown";',
            'export type AttemptUnknownReason = "invalid-input" | "timed-out";',
            'export type FocusFailure = "window-missing" | "activation-failed";',
            'export type ArtifactAvailability = "available" | "missing";',
            'export type FileGrantPurpose = "read" | "replace";',
        ]), encoding="utf-8")
        self.page = self.root / "apps/desktop/ui/task-space.js"
        self.page.parent.mkdir(parents=True)
        self.page.write_text(
            "// 协议模型唯一来源：crates/protocol/generated/protocol.ts\n"
            "const labels = { created: '已创建', running: '执行中', completed: '已完成' };\n",
            encoding="utf-8",
        )

    def test_accepts_display_mapping_with_generated_source(self):
        check_frontend_single_source(self.root)

    def test_rejects_missing_or_wrong_source_declaration(self):
        original = self.page.read_text(encoding="utf-8")
        for source in ("", "// 协议模型唯一来源：apps/desktop/ui/protocol.ts\n"):
            with self.subTest(source=source):
                self.page.write_text(source + original.split("\n", 1)[1], encoding="utf-8")
                with self.assertRaises(ValueError):
                    check_frontend_single_source(self.root)

    def test_rejects_a_second_protocol_enum(self):
        self.page.write_text(
            self.page.read_text(encoding="utf-8")
            + "const taskStatuses = ['created', 'running', 'completed'];\n",
            encoding="utf-8",
        )
        with self.assertRaises(ValueError):
            check_frontend_single_source(self.root)


class DesktopVerificationLayoutTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        checks = self.root / "scripts/verification/desktop"
        checks.mkdir(parents=True)
        (checks / "check-smoke.py").write_text("print('ok')\n", encoding="utf-8")

    def test_accepts_the_unified_verification_directory(self):
        check_desktop_verification_layout(self.root)

    def test_rejects_a_legacy_desktop_root_script(self):
        legacy = self.root / "apps/desktop/check-old.py"
        legacy.parent.mkdir(parents=True)
        legacy.write_text("print('old')\n", encoding="utf-8")
        with self.assertRaises(ValueError):
            check_desktop_verification_layout(self.root)

    def test_rejects_an_empty_verification_directory(self):
        (self.root / "scripts/verification/desktop/check-smoke.py").unlink()
        with self.assertRaises(ValueError):
            check_desktop_verification_layout(self.root)


class AssociationTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        files = {
            "docs/specs/README.md": "技术模块规划",
            "docs/specs/epic-OCT/README.md": "Epic: OCT",
            "docs/specs/epic-OCT/story-OCT-S1/README.md":
                "Story: OCT-S1\nEpic: OCT\nStatus: ready\nOpenSpec: oct-s1-task-status\nopenspec/changes/oct-s1-task-status/",
            "openspec/changes/oct-s1-task-status/proposal.md": "关联 OCT-S1",
            "openspec/changes/oct-s1-task-status/design.md": "已有架构下的实现增量",
            "openspec/changes/oct-s1-task-status/tasks.md": "实施后独立验证",
            "openspec/changes/oct-s1-task-status/specs/tasks/spec.md": "场景与验收",
            "openspec/changes/oct-s1-task-status/verification-goal.md": "OCT-S1：未通过",
        }
        for name, headings in DOCUMENTS.items():
            files[f"docs/specs/epic-OCT/story-OCT-S1/{name}"] = "\n".join(f"## {heading}\n明确需求与可观察失败条件。\n" for heading in headings)
        for name, content in files.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        self.body = (
            "Story: OCT-S1\nOpenSpec: oct-s1-task-status\n"
            "Verification: openspec/changes/oct-s1-task-status/verification-goal.md\n"
        )

    def check(self, body):
        check_pr(self.root, {"pull_request": {"body": body}})

    def test_linked_pending_verification_is_valid(self):
        self.check(self.body)

    def test_archived_change_is_valid_for_planning(self):
        source = self.root / "openspec/changes/oct-s1-task-status"
        archived = self.root / "openspec/changes/archive/2026-09-20-oct-s1-task-status"
        archived.parent.mkdir()
        source.rename(archived)
        story = self.root / "docs/specs/epic-OCT/story-OCT-S1/README.md"
        story.write_text(story.read_text().replace(
            "openspec/changes/oct-s1-task-status/",
            "openspec/changes/archive/2026-09-20-oct-s1-task-status/",
        ), encoding="utf-8")
        check_planning(self.root)

    def test_windows_line_endings(self):
        self.check(self.body.replace("\n", "\r\n"))

    def test_verification_story_prefix_rejected(self):
        path = self.root / "openspec/changes/oct-s1-task-status/verification-goal.md"
        path.write_text("OCT-S10：未通过", encoding="utf-8")
        with self.assertRaises(ValueError):
            self.check(self.body)

    def test_verification_result_gate(self):
        story = self.root / "docs/specs/epic-OCT/story-OCT-S1/README.md"
        verification = self.root / "openspec/changes/oct-s1-task-status/verification-goal.md"
        original_story = story.read_text()
        original_verification = verification.read_text()
        story.write_text(original_story.replace("Status: ready", "Status: verifying"), encoding="utf-8")
        with self.assertRaises(ValueError):
            self.check(self.body)
        verification.write_text("OCT-S1\nResult: FAIL", encoding="utf-8")
        with self.assertRaises(ValueError):
            self.check(self.body)
        verification.write_text("OCT-S1\nResult: PASS", encoding="utf-8")
        self.check(self.body)
        story.write_text(original_story, encoding="utf-8")
        verification.write_text(original_verification, encoding="utf-8")

    def test_change_prefix_rejected(self):
        path = self.root / "docs/specs/epic-OCT/story-OCT-S1/README.md"
        path.write_text(path.read_text().replace("openspec/changes/oct-s1-task-status/", "openspec/changes/oct-s1-task-status-other/"), encoding="utf-8")
        with self.assertRaises(ValueError):
            self.check(self.body)

    def test_missing_empty_duplicate_fields_rejected(self):
        for body in (None, "", self.body.replace("Story: OCT-S1", "Story:"),
                     self.body + "Story: OCT-S1\n"):
            with self.subTest(body=body), self.assertRaises(ValueError):
                self.check(body)

    def test_missing_file_or_reverse_reference_rejected(self):
        with self.assertRaises(ValueError):
            self.check(self.body.replace("verification-goal.md", "absent.md"))
        proposal = self.root / "openspec/changes/oct-s1-task-status/proposal.md"
        proposal.write_text("关联 OCT-S10", encoding="utf-8")
        with self.assertRaises(ValueError):
            self.check(self.body)

    def test_path_escape_rejected(self):
        for value in ("../outside.md", "/tmp/outside.md"):
            with self.subTest(path=value), self.assertRaises(ValueError):
                repository_file(self.root, value)

    def test_missing_or_empty_design_cannot_pass(self):
        for name in DOCUMENTS:
            path = self.root / f"docs/specs/epic-OCT/story-OCT-S1/{name}"
            original = path.read_text()
            for text in ("", "不适用", "\n".join(f"## {h}\nTODO\n" for h in DOCUMENTS[name])):
                path.write_text(text, encoding="utf-8")
                with self.assertRaises(ValueError):
                    check_planning(self.root)
            path.unlink()
            with self.assertRaises(ValueError):
                check_planning(self.root)
            path.write_text(original, encoding="utf-8")

    def test_draft_and_wrong_module_cannot_pass_pr(self):
        path = self.root / "docs/specs/epic-OCT/story-OCT-S1/README.md"
        original = path.read_text()
        for status in ("draft", "design-review", "deferred"):
            path.write_text(original.replace("Status: ready", f"Status: {status}"), encoding="utf-8")
            check_planning(self.root)  # 可规划，不可作为实施 PR。
            with self.assertRaises(ValueError):
                self.check(self.body)
        path.write_text(original.replace("Epic: OCT", "Epic: DS"), encoding="utf-8")
        with self.assertRaises(ValueError):
            check_planning(self.root)

    def test_legacy_flat_story_cannot_bypass(self):
        path = self.root / "docs/specs/epic-OCT/story-OCT-S1/README.md"
        path.unlink()
        old = self.root / "_bmad-output/implementation-artifacts/OCT-S1-TASK-STATUS.md"
        old.parent.mkdir(parents=True)
        old.write_text("openspec/changes/oct-s1-task-status/", encoding="utf-8")
        with self.assertRaises(ValueError):
            self.check(self.body)

    def test_proposal_alone_cannot_authorize_implementation(self):
        for name in ("design.md", "tasks.md", "specs/tasks/spec.md"):
            path = self.root / "openspec/changes/oct-s1-task-status" / name
            original = path.read_text()
            path.unlink()
            with self.assertRaises(ValueError):
                self.check(self.body)
            path.write_text(original, encoding="utf-8")

    def test_archived_proposal_keeps_planning_valid(self):
        active = self.root / "openspec/changes/oct-s1-task-status/proposal.md"
        story = self.root / "docs/specs/epic-OCT/story-OCT-S1/README.md"
        original_story = story.read_text()
        archived = self.root / "openspec/changes/archive/2026-09-22-oct-s1-task-status/proposal.md"
        archived.parent.mkdir(parents=True)
        archived.write_text(active.read_text(), encoding="utf-8")
        active.unlink()
        try:
            story.write_text(original_story.replace("openspec/changes/oct-s1-task-status/", "openspec/changes/archive/2026-09-22-oct-s1-task-status/"), encoding="utf-8")
            check_planning(self.root)
        finally:
            story.write_text(original_story, encoding="utf-8")
            active.write_text(archived.read_text(), encoding="utf-8")
            archived.unlink()
            archived.parent.rmdir()


if __name__ == "__main__":
    unittest.main()
