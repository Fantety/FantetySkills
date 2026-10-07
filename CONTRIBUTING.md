# Contributing

Share the task or recurring problem that motivates a skill, the behavior it should improve, and how you verified it. Keep instructions focused on decisions an agent would not reliably make without that guidance.

## Structure and documentation

- Follow [the layout conventions](docs/skill-layout.md). Use `skills/<skill-name>/SKILL.md` as the entry point and match the frontmatter `name` to the directory name.
- Keep every skill independently installable. Put required helpers, reference documents, templates, and runtime sources inside its directory. Put repository tooling in `scripts/`, developer documentation in `docs/`, and tests in `tests/<skill-name>/`.
- Read and write text as UTF-8. Use relative links and resolve runtime resources from the skill's location, independently of the current working directory.
- Update both README indexes when adding or renaming a skill. Update recipes and references when an API or recipe field changes.
- Preserve existing copyright and license notices. Document imported components and their license scope in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
- Do not commit generated artwork, virtual environments, Python caches, or Cargo build output.

## Local checks

Run from the repository root with Python 3.10+; a local virtual environment is recommended:

```sh
python -m pip install -r requirements-dev.txt
python scripts/validate_skills.py
```

The validator checks required metadata, naming, and the presence of instructions for all skills. Review the instructions and links as well; valid frontmatter alone does not establish useful behavior.

For changes to oy-pixel's paths, renderer, examples, or engine, also run:

```sh
python skills/oy-pixel/scripts/render.py --check
python tests/oy-pixel/test_render.py
cargo test --locked --manifest-path skills/oy-pixel/scripts/engine/Cargo.toml
cargo fmt --manifest-path skills/oy-pixel/scripts/engine/Cargo.toml -- --check
```

The rendering tests check pixels, transparency, timing, output formats, edits to existing images, operation from unrelated working directories, and a standalone skill copy without Cargo on `PATH`. CI runs these checks on Windows and Linux.

## Updating the bundled renderer

The Windows executable and `scripts/bin/source.sha256` ship with the skill. Keep the fingerprint paired with the engine source it was built from; do not update the fingerprint alone. After changing Rust sources or Cargo dependencies, run `render.py --check` on Windows to rebuild the executable, then run the rendering tests before including the new executable and fingerprint in a release. The renderer automatically builds from source on other platforms when no matching cached executable exists.

Engine documentation and further usage examples are linked from [docs/oy-pixel.md](docs/oy-pixel.md).
