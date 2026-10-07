# oy-pixel: usage and development

The [oy-pixel skill](../skills/oy-pixel/SKILL.md) draws pixel artwork and frame animations from JSON recipes and Lua scripts. It exports PNG images, animated GIFs, sprite sheets, and PNG frame sequences.

## Installation and quick start

Copy the complete `skills/oy-pixel/` directory into your agent's skills directory; for Codex, this is normally `~/.codex/skills/oy-pixel/`. No repository-level files are needed at runtime. After installation, invoke `$oy-pixel` with the artwork request, dimensions, and desired output format.

Use Python 3.10+ and Pillow 10–12. The skill includes a Windows renderer executable. Other platforms build it on first use and need Cargo and a C compiler. From the repository root:

```sh
python -m pip install -r skills/oy-pixel/scripts/requirements.txt
python skills/oy-pixel/scripts/render.py --check
python skills/oy-pixel/scripts/render.py skills/oy-pixel/assets/examples/still.json --out output --name sprite
python skills/oy-pixel/scripts/render.py skills/oy-pixel/assets/examples/loop.json --out output --name sprite-loop --formats gif,sheet
```

For an installed copy, substitute the installed skill path. Recipes can live anywhere; script and base-image paths are relative to their recipe. The renderer writes finished artwork to the chosen output directory and preserves existing files unless `--overwrite` is explicitly supplied.

## References and source

- [Rendering and recipes](../skills/oy-pixel/references/workflow.md): setup, JSON fields, layers, timing, export options, and inspection.
- [Lua drawing API](../skills/oy-pixel/references/lua-api.md): available drawing functions and execution limits.
- [Pixel and animation craft](../skills/oy-pixel/references/craft.md): pixel construction, motion, and timing.
- [Runnable recipes](../skills/oy-pixel/assets/examples/): still and animation examples using the same Lua script.
- [Rust/Lua engine](../skills/oy-pixel/scripts/engine/README.md): embedded runtime and library interface.

The engine source and its cached binary live inside `scripts/` so a copied skill remains self-contained. Rust dependencies are pinned in [Cargo.lock](../skills/oy-pixel/scripts/engine/Cargo.lock).

## Verification

```sh
python tests/oy-pixel/test_render.py
cargo test --locked --manifest-path skills/oy-pixel/scripts/engine/Cargo.toml
cargo fmt --manifest-path skills/oy-pixel/scripts/engine/Cargo.toml -- --check
```

The Python tests verify PNG/GIF pixels, transparency, frame timing, sprite sheets, source-image edits, and an isolated copy of the skill. See [CONTRIBUTING.md](../CONTRIBUTING.md) for repository checks and updating the bundled Windows renderer.

## Origin and license

Source project: [Fantety/OY-pixel-skill](https://github.com/Fantety/OY-pixel-skill).

The imported skill and its tests retain the [MIT license](../skills/oy-pixel/LICENSE). Dependencies retain their own licenses; see the skill's [third-party notices](../skills/oy-pixel/THIRD_PARTY_NOTICES.md).
