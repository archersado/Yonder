"""使用合成 Cargo 元数据和 PR 事件验证门禁拒绝行为。"""

import tempfile
from pathlib import Path
import unittest

from check_architecture import DOCUMENTS, check_dependencies, check_planning, check_pr, repository_file


def metadata(name, dependencies):
    return {"workspace_members": [name], "packages": [
        {"id": name, "name": name, "dependencies": dependencies}
    ]}


class DependencyTests(unittest.TestCase):
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

    def test_windows_line_endings(self):
        self.check(self.body.replace("\n", "\r\n"))

    def test_verification_story_prefix_rejected(self):
        path = self.root / "openspec/changes/oct-s1-task-status/verification-goal.md"
        path.write_text("OCT-S10：未通过", encoding="utf-8")
        with self.assertRaises(ValueError):
            self.check(self.body)

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
