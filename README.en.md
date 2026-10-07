# FantetySkills

> Practical experience gathered while developing with Agents — common problems encountered and the Skills that solve them.

[简体中文](./README.md) | **English**

This repository is a personal knowledge base built up through hands-on development with AI Agents (Codex / Trae, etc.). Whenever a recurring problem surfaces during Agent collaboration, I abstract it into a reusable **Skill** and document the problem background, the reasoning behind the solution, and the Skill itself — so it can be invoked directly in future projects and the same pitfalls can be avoided.

## Included Skills

### [build-user-centered-products](./skills/build-user-centered-products/SKILL.md)

**Problem it solves**: Agents tend to treat a request as a complete product specification and execute it literally, ignoring the user's real goal and the coherence of the overall experience.

**Core idea**: Treat every request as evidence of a user need, not as the final spec. Before making consequential decisions, build an internal product model — identify the user and their goal, determine the primary task, model the states and transitions, evaluate the full journey — and only then choose the implementation, rather than applying blanket rules.

**When to use**: Designing, implementing, modifying, or reviewing user-facing software. Use this Skill when you want the Agent to do more than "finish the requirement" and actually produce a coherent, usable, well-prioritized experience.

### [create-posters](./skills/create-posters/SKILL.md)

Create or revise single-page posters from copy, images, logos, QR codes, and delivery constraints. Covers hierarchy, readability, asset integrity, and export quality for promotional, event, and information posters, including fixed-canvas HTML and PNG/PDF deliverables.

### [oy-pixel](./skills/oy-pixel/SKILL.md)

Create and edit pixel art, sprites, icons, tiles, effects, and frame animations. Render JSON recipes and Lua drawing scripts into PNG images, animated GIFs, sprite sheets, or PNG frame sequences. Includes a Python renderer, a Rust/Lua engine, and a prebuilt Windows executable.

See the [oy-pixel usage and development guide](./docs/oy-pixel.md) for requirements, examples, and verification commands.

## Installation and usage

Each `skills/<skill-name>/` is an independently installable skill. Copy the **entire directory** into your agent's skills directory. Codex defaults to `~/.codex/skills/` (`%USERPROFILE%\.codex\skills\` on Windows, or `skills/` inside `CODEX_HOME` when configured).

For example, an oy-pixel installation should contain `~/.codex/skills/oy-pixel/SKILL.md` alongside its `scripts/`, `references/`, `assets/`, and license files. Agents that support explicit invocation can use `$oy-pixel`, `$create-posters`, or `$build-user-centered-products`.

The two guidance-only skills have no runtime dependencies. oy-pixel requires Python 3.10+ and Pillow 10–12. A Windows renderer is bundled; other platforms need Cargo and a C compiler for the first run.

## Repository layout

```text
FantetySkills/
├── README.md / README.en.md      Repository overview and skill index
├── LICENSE                      Default Apache-2.0 license
├── THIRD_PARTY_NOTICES.md        Imported components and license scope
├── CONTRIBUTING.md              Contribution and verification guide
├── .github/workflows/           Continuous integration
├── docs/                        Repository usage and development guides
├── scripts/                     Repository maintenance tools
├── tests/<skill-name>/          Automated tests
├── requirements-dev.txt         Development and validation dependencies
└── skills/
    ├── build-user-centered-products/
    ├── create-posters/
    └── oy-pixel/
```

## Contributing

This is a continuously growing personal project. Issues and pull requests describing recurring problems and reusable solutions are welcome. Read [CONTRIBUTING.md](./CONTRIBUTING.md) before adding or changing a skill.

```sh
python -m pip install -r requirements-dev.txt
python scripts/validate_skills.py
```

Changes to rendering behavior or the engine also require the Python and Rust checks listed in the contribution guide.

## License

Unless otherwise stated, this repository is licensed under [Apache License 2.0](./LICENSE). `skills/oy-pixel/` and its rendering tests in `tests/oy-pixel/` retain the original project's [MIT license](./skills/oy-pixel/LICENSE). See [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md) for dependency and distribution notices.
