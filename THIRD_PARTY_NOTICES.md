# Third-party and imported-component notices

FantetySkills code, skills, documentation, and tests are licensed under [MIT](LICENSE). Third-party dependencies retain their own licenses. The following component is also distributed under MIT:

| Component | Scope | Origin | License |
| --- | --- | --- | --- |
| oy-pixel | `skills/oy-pixel/` and `tests/oy-pixel/` | [Fantety/OY-pixel-skill](https://github.com/Fantety/OY-pixel-skill), copyright (c) 2026 Fantety | [MIT](skills/oy-pixel/LICENSE) |

Each installable skill includes a copy of the MIT license.

The oy-pixel renderer bundles Lua and Rust dependencies, and uses Pillow as a separately installed Python dependency. Their notices are included in [skills/oy-pixel/THIRD_PARTY_NOTICES.md](skills/oy-pixel/THIRD_PARTY_NOTICES.md). Exact Rust dependency versions are recorded in [Cargo.lock](skills/oy-pixel/scripts/engine/Cargo.lock).

Keep the skill's `LICENSE` and `THIRD_PARTY_NOTICES.md` files when copying or redistributing it independently.
