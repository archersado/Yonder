"""检查 Cargo 模块依赖与 PR 研发关联；只使用 Python 标准库。"""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys


ALLOWED = {
    "yonder-domain": set(),
    "yonder-protocol": set(),
    "yonder-application": {"yonder-domain", "yonder-protocol"},
    "yonder-adapters": {"yonder-application", "yonder-protocol"},
    "yonder-desktop": {"yonder-adapters", "yonder-application"},
    "yonder-cli": {"yonder-protocol"},
}

FRONTEND_PROTOCOL_SOURCE = "crates/protocol/generated/protocol.ts"
FRONTEND_PROTOCOL_TYPES = {
    "TaskStatus",
    "TaskSource",
    "TaskObservationResult",
    "AttemptUnknownReason",
    "FocusFailure",
    "ArtifactAvailability",
    "FileGrantPurpose",
}


def cargo_executable():
    return os.environ.get("CARGO") or shutil.which("cargo") or str(Path.home() / ".cargo/bin/cargo")


def check_dependencies(metadata):
    members = set(metadata["workspace_members"])
    packages = [p for p in metadata["packages"] if p["id"] in members]
    internal = {p["name"] for p in packages}
    for package in packages:
        name = package["name"]
        if name not in ALLOWED:
            raise ValueError(f"未登记的 Workspace 模块：{name}，须先核对架构")
        for dep in package["dependencies"]:
            # Cargo 的 name 是真实包名；rename 是使用别名，不能据其判断方向。
            target = dep["name"]
            if name == "yonder-domain" or (
                (target in internal or dep.get("path") or target.startswith("yonder-"))
                and target not in ALLOWED[name]
            ):
                raise ValueError(f"禁止依赖：{name} → {target} ({dep.get('kind') or 'normal'})")


def check_protocol_single_source(root):
    rust_files = list((root / "crates").glob("**/*.rs"))
    valid_id_definitions = [
        path for path in rust_files
        if re.search(r"^(?:pub(?:\([^)]*\))?\s+)?fn\s+valid_id\s*\(", path.read_text(encoding="utf-8"), re.MULTILINE)
    ]
    expected_valid_id = root / "crates/protocol/src/lib.rs"
    if valid_id_definitions != [expected_valid_id]:
        names = ", ".join(str(path.relative_to(root)) for path in valid_id_definitions) or "无"
        raise ValueError(f"valid_id 必须仅定义于 yonder-protocol，当前：{names}")

    version_definitions = [
        path for path in rust_files
        if re.search(r"^pub\s+const\s+PROTOCOL_VERSION\b", path.read_text(encoding="utf-8"), re.MULTILINE)
    ]
    if version_definitions != [expected_valid_id]:
        raise ValueError("PROTOCOL_VERSION 必须仅定义于 yonder-protocol")

    application = root / "crates/application/src"
    enums = [path for path in application.glob("**/*.rs") if re.search(r"^pub\s+enum\s+UnknownReason\b", path.read_text(encoding="utf-8"), re.MULTILINE)]
    mappings = [path for path in application.glob("**/*.rs") if "impl From<UnknownReason> for yonder_protocol::AttemptUnknownReason" in path.read_text(encoding="utf-8")]
    expected_reason = application / "unknown_reason.rs"
    if enums != [expected_reason] or mappings != [expected_reason]:
        raise ValueError("UnknownReason 与 wire 枚举必须只有 unknown_reason.rs 中的一份映射")

    bridge = (root / "apps/yonder-cli/src/codex_agent_bridge.rs").read_text(encoding="utf-8")
    if (
        "有意钉定 1.51" not in bridge
        or not re.search(r"minor:\s*51\b", bridge)
        or "OfferedCapability::UserInputReply" not in bridge
        or "minor: PROTOCOL_VERSION.minor" in bridge
    ):
        raise ValueError("Codex bridge 必须显式钉定抢占关闭所需的协议 1.51")


def check_frontend_single_source(root):
    generated = root / FRONTEND_PROTOCOL_SOURCE
    if not generated.is_file():
        raise ValueError(f"前端协议生成来源不存在：{FRONTEND_PROTOCOL_SOURCE}")
    generated_text = generated.read_text(encoding="utf-8")
    unions = {}
    for name, body in re.findall(
        r'^export type (\w+) = ((?:"[^"]+"(?: \| )?)+);$',
        generated_text,
        re.MULTILINE,
    ):
        if name in FRONTEND_PROTOCOL_TYPES:
            unions[name] = set(re.findall(r'"([^"]+)"', body))
    missing = sorted(FRONTEND_PROTOCOL_TYPES - unions.keys())
    if missing:
        raise ValueError(f"前端协议生成来源缺少受控类型：{', '.join(missing)}")

    declaration = f"协议模型唯一来源：{FRONTEND_PROTOCOL_SOURCE}"
    for page in sorted((root / "apps/desktop/ui").glob("*.js")):
        text = page.read_text(encoding="utf-8")
        mappings = re.findall(
            r"\bconst\s+(?:labels|[A-Za-z_$][\w$]*(?:Labels|Statuses|Sources|Purposes|Availabilities))\s*=\s*([\[{][^;]+)",
            text,
            re.DOTALL,
        )
        mapping_text = "\n".join(mappings)
        literals = set(re.findall(r"(?<![\w-])['\"]([a-z][a-z0-9-]*)['\"]", mapping_text))
        literals.update(re.findall(r"\b([a-z][a-z0-9-]*)\s*:", mapping_text))
        used = sorted(
            name for name, values in unions.items()
            if len(literals & values) >= min(2, len(values))
        )
        if not used:
            continue
        relative = page.relative_to(root)
        if declaration not in text:
            raise ValueError(
                f"{relative} 使用协议状态 {', '.join(used)}，但未声明唯一生成来源"
            )
        for variable, body in re.findall(
            r"\bconst\s+([A-Za-z_$][\w$]*(?:Statuses|Sources|Purposes|Availabilities))\s*=\s*\[([^\]]*)\]",
            text,
            re.DOTALL | re.IGNORECASE,
        ):
            values = set(re.findall(r"['\"]([a-z][a-z0-9-]*)['\"]", body))
            if any(len(values & union) >= min(2, len(union)) for union in unions.values()):
                raise ValueError(f"{relative} 重新声明协议枚举集合：{variable}")


def check_desktop_verification_layout(root):
    legacy = sorted((root / "apps/desktop").glob("check-*"))
    if legacy:
        names = ", ".join(path.name for path in legacy)
        raise ValueError(f"Desktop 验证脚本必须迁入 scripts/verification/desktop：{names}")
    directory = root / "scripts/verification/desktop"
    checks = sorted(path for path in directory.glob("check-*") if path.is_file())
    if not checks:
        raise ValueError("缺少统一 Desktop 验证脚本目录或 check-* 脚本")


def repository_file(root, value):
    relative = Path(value)
    if relative.is_absolute() or ".." in relative.parts:
        raise ValueError("关联文件必须使用仓库内相对路径")
    path = (root / relative).resolve()
    if root.resolve() not in path.parents or not path.is_file():
        raise ValueError(f"关联文件不存在或越界：{value}")
    return path


def change_directory(root, change):
    matches = [path for path in (root / "openspec/changes").glob(f"**/{change}") if path.is_dir()]
    if len(matches) != 1:
        raise ValueError(f"OpenSpec Change 不存在或重复：{change}")
    return matches[0]


def check_pr(root, event):
    body = (event["pull_request"].get("body") or "").replace("\r\n", "\n")
    fields = {}
    for label in ("Story", "OpenSpec", "Verification"):
        values = re.findall(rf"^{label}:[ \t]*([^\r\n]+)$", body, re.MULTILINE)
        if len(values) != 1:
            raise ValueError(f"PR 正文必须有唯一的 {label}: 字段")
        fields[label] = values[0].strip()
    story = fields["Story"]
    change = fields["OpenSpec"]
    if not re.fullmatch(r"[A-Z][A-Z0-9]*-S[0-9]+", story):
        raise ValueError("Story ID 格式错误")
    if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", change):
        raise ValueError("OpenSpec Change 名称格式错误")
    stories = check_planning(root)
    if story not in stories:
        raise ValueError("Story 必须存在于技术模块 Epic 目录")
    story_path, info = stories[story]
    if info["Status"] not in {"ready", "implementing", "verifying", "done"}:
        raise ValueError("Story 设计尚未就绪，不得进入实施 PR")
    if info["OpenSpec"] != change:
        raise ValueError("Story 与 PR 的 OpenSpec 不一致")
    story_text = story_path.read_text(encoding="utf-8")
    directory = change_directory(root, change)
    prefix = str(directory.relative_to(root))
    proposal = repository_file(root, f"{prefix}/proposal.md").read_text(encoding="utf-8")
    for name in ("design.md", "tasks.md"):
        if not repository_file(root, f"{prefix}/{name}").read_text(encoding="utf-8").strip():
            raise ValueError(f"OpenSpec 文档为空：{name}")
    specs = list((root / prefix / "specs").glob("*/spec.md"))
    if not specs:
        raise ValueError("OpenSpec 缺少 delta spec")
    for spec in specs:
        if not repository_file(root, str(spec.relative_to(root))).read_text(encoding="utf-8").strip():
            raise ValueError("OpenSpec delta spec 为空")
    story_pattern = rf"(?<![A-Za-z0-9_-]){re.escape(story)}(?![A-Za-z0-9_-])"
    change_pattern = rf"(?<![A-Za-z0-9_/-]){re.escape(prefix)}(?![A-Za-z0-9_-])"
    if not re.search(change_pattern, story_text) or not re.search(story_pattern, proposal):
        raise ValueError("Story 与 OpenSpec 缺少双向引用")
    verification = repository_file(root, fields["Verification"])
    verification_text = verification.read_text(encoding="utf-8")
    if verification.suffix != ".md" or not re.search(story_pattern, verification_text):
        raise ValueError("验证记录必须是引用该 Story 的 Markdown 文件")
    result = re.findall(r"^Result:\s*(PASS|FAIL|PENDING)\s*$", verification_text, re.MULTILINE)
    if result and result != ["PASS"]:
        raise ValueError("验证记录未通过，不能进入 PR")
    if info["Status"] in {"verifying", "done"} and result != ["PASS"]:
        raise ValueError("verifying/done Story 必须引用 Result: PASS 的验证记录")


DOCUMENTS = {
    "product-requirements.md": ("问题与目标", "范围与非目标", "验收条件"),
    "architecture-design.md": ("边界与依赖", "状态与契约", "失败与验证"),
    "visual-interaction-design.md": ("入口与流程", "状态与错误反馈", "无障碍与平台验证"),
}


def field(text, name):
    values = re.findall(rf"^{name}:[ \t]*([^\r\n]+)$", text, re.MULTILINE)
    if len(values) != 1:
        raise ValueError(f"规划必须有唯一 {name} 字段")
    return values[0].strip()


def planning_change_prefix(root, change):
    active = f"openspec/changes/{change}"
    if (root / active / "proposal.md").is_file():
        return active
    archived = sorted((root / "openspec/changes/archive").glob(f"????-??-??-{change}"))
    if len(archived) != 1:
        raise ValueError(f"OpenSpec Change 不存在或归档不唯一：{change}")
    return str(archived[0].relative_to(root))


def check_planning(root):
    repository_file(root, "docs/specs/README.md")
    stories = {}
    epics = sorted((root / "docs/specs").glob("epic-*"))
    if not epics:
        raise ValueError("缺少技术模块 Epic")
    for epic in epics:
        code = epic.name.removeprefix("epic-")
        epic_text = repository_file(root, str(epic.relative_to(root) / "README.md")).read_text(encoding="utf-8")
        if not re.fullmatch(r"[A-Z][A-Z0-9]*", code) or field(epic_text, "Epic") != code:
            raise ValueError(f"Epic 标识不匹配：{epic.name}")
        folders = list(epic.glob("story-*"))
        if not folders:
            raise ValueError(f"Epic 缺少 Story：{code}")
        for folder in folders:
            path = repository_file(root, str(folder.relative_to(root) / "README.md"))
            text = path.read_text(encoding="utf-8")
            info = {key: field(text, key) for key in ("Story", "Epic", "Status", "OpenSpec")}
            sid = info["Story"]
            if not re.fullmatch(rf"{code}-S[1-9][0-9]*", sid) or folder.name != f"story-{sid}" or info["Epic"] != code or sid in stories:
                raise ValueError(f"Story 标识、归属或唯一性错误：{folder}")
            if info["Status"] not in {"draft", "design-review", "ready", "implementing", "verifying", "done", "deferred"}:
                raise ValueError(f"未知 Story 状态：{sid}")
            for name, headings in DOCUMENTS.items():
                doc = repository_file(root, str(folder.relative_to(root) / name)).read_text(encoding="utf-8")
                for heading in headings:
                    match = re.search(rf"^## {heading}\s*\n(.*?)(?=^## |\Z)", doc, re.MULTILINE | re.DOTALL)
                    if not match or match.group(1).strip() in {"", "不适用", "N/A", "TODO", "待补充"}:
                        raise ValueError(f"{sid}/{name} 缺少有效章节：{heading}")
            change = info["OpenSpec"]
            if change == "-":
                if info["Status"] in {"implementing", "verifying", "done"}:
                    raise ValueError(f"实施 Story 缺少 OpenSpec：{sid}")
            else:
                if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", change):
                    raise ValueError(f"非法 OpenSpec：{sid}")
                prefix = planning_change_prefix(root, change)
                proposal = repository_file(root, f"{prefix}/proposal.md").read_text(encoding="utf-8")
                if not re.search(rf"(?<![A-Za-z0-9_-]){sid}(?![A-Za-z0-9_-])", proposal) or f"{prefix}/" not in text:
                    raise ValueError(f"Story/Proposal 缺少双向关联：{sid}")
            stories[sid] = (path, info)
    return stories


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--event", type=Path, help="仅 PR 事件传入 GitHub 事件 JSON")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    metadata = subprocess.check_output(
        [cargo_executable(), "metadata", "--format-version", "1", "--no-deps", "--locked", "--offline"],
        cwd=root, text=True,
    )
    check_dependencies(json.loads(metadata))
    check_protocol_single_source(root)
    check_frontend_single_source(root)
    check_desktop_verification_layout(root)
    check_planning(root)
    if args.event:
        check_pr(root, json.loads(args.event.read_text(encoding="utf-8")))
    print("架构与关联检查通过（不代表 Story 验证通过）")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, KeyError, subprocess.CalledProcessError) as error:
        print(f"门禁失败：{error}", file=sys.stderr)
        sys.exit(1)
