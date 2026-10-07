#!/usr/bin/env python3
"""Validate the entry points of every skill in this repository."""
from pathlib import Path
import re
import sys

import yaml


ROOT = Path(__file__).resolve().parents[1]
NAME_PATTERN = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)*")


def validate_skill(directory):
    entry = directory / "SKILL.md"
    if not entry.is_file():
        return ["missing SKILL.md"]
    content = entry.read_text(encoding="utf-8")
    match = re.match(r"\A---\n(.*?)\n---(?:\n|$)", content, re.DOTALL)
    if not match:
        return ["SKILL.md must start with YAML frontmatter"]
    try:
        metadata = yaml.safe_load(match.group(1))
    except yaml.YAMLError as error:
        return [f"invalid YAML frontmatter: {error}"]
    if not isinstance(metadata, dict):
        return ["frontmatter must be a mapping"]

    errors = []
    name = metadata.get("name")
    if not isinstance(name, str) or len(name) > 64 or not NAME_PATTERN.fullmatch(name):
        errors.append("name must use lowercase letters, digits and single hyphens (1–64 characters)")
    elif name != directory.name:
        errors.append("name must match the skill directory")
    description = metadata.get("description")
    if not isinstance(description, str) or not description.strip() or len(description) > 1024:
        errors.append("description must be a nonempty string of at most 1024 characters")
    if not content[match.end():].strip():
        errors.append("SKILL.md must include instructions after the frontmatter")
    return errors


def main():
    directories = sorted(path for path in (ROOT / "skills").iterdir() if path.is_dir())
    if not directories:
        print("No skills found.", file=sys.stderr)
        return 1
    failed = False
    for directory in directories:
        errors = validate_skill(directory)
        if errors:
            failed = True
            for error in errors:
                print(f"{directory.name}: {error}", file=sys.stderr)
        else:
            print(f"{directory.name}: valid")
    return int(failed)


if __name__ == "__main__":
    sys.exit(main())
