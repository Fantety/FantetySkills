# Third-party and imported-component notices

FantetySkills uses [Apache License 2.0](LICENSE) by default. The following imported component retains its original license:

| Component | Scope | Origin | License |
| --- | --- | --- | --- |
| oy-pixel | `skills/oy-pixel/` and `tests/oy-pixel/` | [Fantety/OY-pixel-skill](https://github.com/Fantety/OY-pixel-skill), copyright (c) 2026 Fantety | [MIT](skills/oy-pixel/LICENSE) |

The original MIT license is included with the installable skill and covers that component and its tests.

The oy-pixel renderer bundles Lua and Rust dependencies, and uses Pillow as a separately installed Python dependency. Their notices are included in [skills/oy-pixel/THIRD_PARTY_NOTICES.md](skills/oy-pixel/THIRD_PARTY_NOTICES.md). Exact Rust dependency versions are recorded in [Cargo.lock](skills/oy-pixel/scripts/engine/Cargo.lock).

Keep the skill's `LICENSE` and `THIRD_PARTY_NOTICES.md` files when copying or redistributing it independently.
