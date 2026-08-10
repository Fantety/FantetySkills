# FantetySkills

> Practical experience gathered while developing with Agents — common problems encountered and the Skills that solve them.

[简体中文](./README.md) | **English**

This repository is a personal knowledge base built up through hands-on development with AI Agents (Codex / Trae, etc.). Whenever a recurring problem surfaces during Agent collaboration, I abstract it into a reusable **Skill** and document the problem background, the reasoning behind the solution, and the Skill itself — so it can be invoked directly in future projects and the same pitfalls can be avoided.

## Included Skills

### [build-user-centered-products](./skills/build-user-centered-products/SKILL.md)

**Problem it solves**: Agents tend to treat a request as a complete product specification and execute it literally, ignoring the user's real goal and the coherence of the overall experience.

**Core idea**: Treat every request as evidence of a user need, not as the final spec. Before making consequential decisions, build an internal product model — identify the user and their goal, determine the primary task, model the states and transitions, evaluate the full journey — and only then choose the implementation, rather than applying blanket rules.

**When to use**: Designing, implementing, modifying, or reviewing user-facing software. Use this Skill when you want the Agent to do more than "finish the requirement" and actually produce a coherent, usable, well-prioritized experience.

## Contributing

This is a continuously growing personal project. As I encounter more typical problems in Agent development, new Skills will be added. If you also develop with Agents and want to share your experience, feel free to open an issue or PR.

## License

[Apache License 2.0](./LICENSE)
