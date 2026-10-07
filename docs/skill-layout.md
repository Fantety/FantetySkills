# Skill repository layout

FantetySkills is a collection of independently installable [Agent Skills](https://agentskills.io/specification). Each skill lives at `skills/<skill-name>/`; users install that folder, while contributors work with the full repository.

## Installable skill

`SKILL.md` is the required entry point. Start it with YAML frontmatter containing `name` and `description`, followed by the agent instructions. Match `name` to the directory, use lowercase letters, digits, and single hyphens, and keep it within 64 characters. Describe what the skill does and when it applies in at most 1024 characters. Set `license: MIT` and include a copy of the repository's `LICENSE` in each skill for independent distribution.

Use these locations for supporting files:

| Location | Purpose |
| --- | --- |
| `agents/openai.yaml` | Codex display information and invocation metadata |
| `references/` | Detailed guidance read only when relevant, linked from the entry point or another reference |
| `scripts/` | Executable helpers and any runtime source or binaries they need |
| `assets/` | Templates, example inputs, and reusable materials copied or adapted into output |
| `LICENSE`, `THIRD_PARTY_NOTICES.md` | Applicable license and dependency notices that must accompany an installed copy |

Create resource directories only when needed. A guidance-only skill can consist of `SKILL.md`, `LICENSE`, and optional agent metadata. An executable skill may need more nested resources.

Keep runtime dependencies inside the installable folder. Derive their paths from the script location and resolve user-supplied inputs according to the documented interface. Do not embed contributor-specific absolute paths or depend on sibling skills, repository tests, or the original source project.

For oy-pixel, that produces:

```text
skills/oy-pixel/
├── SKILL.md
├── LICENSE
├── THIRD_PARTY_NOTICES.md
├── agents/openai.yaml
├── references/
│   ├── workflow.md
│   ├── lua-api.md
│   └── craft.md
├── assets/examples/          JSON recipes and Lua input
└── scripts/
    ├── render.py
    ├── requirements.txt
    ├── bin/                 Cached renderer and source fingerprint
    └── engine/              Cargo manifest, lockfile, and Rust/Lua source
```

The engine's `README.md` is retained because Rust includes it as crate documentation. It is part of the engine source, while the agent-facing drawing API stays in `references/lua-api.md`.

## Repository development files

Use root-level `docs/` for installation and developer guides, `scripts/` for repository maintenance, `tests/<skill-name>/` for behavioral tests, and `.github/workflows/` for CI. Add a skill to both README indexes and document third-party dependency licenses in the relevant notices.

The repository itself is not nested inside a skill. Keep `.git/`, virtual environments, generated artwork, Python caches, and compiler output out of installable copies. Do not commit nested repositories or copy a source project's build cache into a skill.

See [CONTRIBUTING.md](../CONTRIBUTING.md) for validation commands.
