# FantetySkills

[简体中文](./README.md) | **English**

Practical skills for product development, poster design, pixel art, and sound effects, drawn from everyday work with AI agents. Each skill turns recurring problems and their solutions into reusable instructions for agents that support skills, including Codex, Claude Code, and Trae.

## Installation

Install [Node.js](https://nodejs.org/), then run this command in the project where you want to use the skills:

```sh
npx skills add https://github.com/Fantety/FantetySkills
```

Follow the prompts to select skills and target agents. To install an individual skill, use the corresponding command:

```sh
npx skills add https://github.com/Fantety/FantetySkills --skill build-user-centered-products
npx skills add https://github.com/Fantety/FantetySkills --skill create-posters
npx skills add https://github.com/Fantety/FantetySkills --skill oy-pixel
npx skills add https://github.com/Fantety/FantetySkills --skill oy-audio
```

Installation defaults to the current project. Add `-g` to make skills available across projects and `--agent` to select an agent. For example, install all skills globally for Codex:

```sh
npx skills add https://github.com/Fantety/FantetySkills --skill '*' --agent codex -g
```

To list available skills before installing:

```sh
npx skills add https://github.com/Fantety/FantetySkills --list
```

See the [skills CLI documentation](https://github.com/vercel-labs/skills#readme) for more agents and installation options.

## Skills

| Skill | Purpose |
| --- | --- |
| [build-user-centered-products](./skills/build-user-centered-products/SKILL.md) | Design, implement, and review user-facing software around real user goals, primary tasks, and complete journeys. |
| [create-posters](./skills/create-posters/SKILL.md) | Create single-page posters from copy, images, logos, and QR codes, with guidance on hierarchy, layout, and export to fixed-canvas HTML or PNG/PDF. |
| [oy-pixel](./skills/oy-pixel/SKILL.md) | Create and edit pixel art, sprites, icons, tiles, effects, and frame animations; export PNG, GIF, sprite sheets, or PNG frame sequences. |
| [oy-audio](./skills/oy-audio/SKILL.md) | Procedurally generate UI sounds, game effects, stylized foley, ambience, and sound loops; deliver WAV files, editable recipes, and audio analysis. |

`build-user-centered-products` and `create-posters` have no additional runtime dependencies. The `oy-pixel` renderer requires Python 3.10+ and Pillow 10–12. A Windows executable is bundled; other platforms need Rust/Cargo and a C compiler for the first run. See the [oy-pixel guide](./docs/oy-pixel.md).

`oy-audio` requires Python 3.10+ and NumPy 2.x, with no model API, GPU, or original oy-audio application required. See the [oy-audio guide](./docs/oy-audio.md).

## Usage

After installation, describe the task and name the skill in your agent. In Codex, for example:

```text
$build-user-centered-products Review the signup flow and fix obstacles that prevent users from completing it.
$create-posters Create an event poster from the supplied copy, images, and QR code, and deliver a PNG.
$oy-pixel Create a 32×32 character walk cycle and export a transparent GIF and sprite sheet.
$oy-audio Create a short impact with metallic resonance; deliver WAV, an editable recipe, and analysis.
```

## Contributing

Issues and pull requests describing recurring problems and reusable solutions are welcome. See the [contribution guide](./CONTRIBUTING.md) for adding or changing skills and the [layout conventions](./docs/skill-layout.md) for file organization.

## License

This repository is licensed under the [MIT License](./LICENSE). See [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md) for third-party dependency licenses and distribution notices.
