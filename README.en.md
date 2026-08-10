# FantetySkills

> Practical experience gathered while developing with Agents — common problems encountered and the Skills that solve them.

[简体中文](./README.md) | **English**

This repository is a personal knowledge base built up through hands-on development with AI Agents (Codex / Trae, etc.). Whenever a recurring problem surfaces during Agent collaboration, I abstract it into a reusable **Skill** and document the problem background, the reasoning behind the solution, and the Skill itself — so it can be invoked directly in future projects and the same pitfalls can be avoided.

## Why It Exists

Agents can dramatically boost development productivity, but they also tend to produce work that "seems to meet the requirement yet drifts from the user's real goal":

- Satisfying the literal request while ignoring the coherence of the overall experience
- Getting the local implementation right while making the overall flow fragile or hard to recover
- Overspending complexity on unimportant details while leaving the critical path rough
- Lacking judgment about user state and task priority, producing generic, unfocused solutions

These issues rarely stem from model capability limits. They stem from the absence of an explicit "product reasoning" framework to guide the Agent. FantetySkills tries to codify that reasoning in the form of Skills, so the Agent applies it consistently in every task.

## Repository Structure

```
FantetySkills/
├── skills/                          # Reusable Skill collection
│   └── build-user-centered-products/
│       ├── SKILL.md                 # Skill definition
│       └── agents/
│           └── openai.yaml          # Agent integration config
├── LICENSE                          # Apache 2.0
└── README.md
```

## Included Skills

### [build-user-centered-products](./skills/build-user-centered-products/SKILL.md)

**Problem it solves**: Agents tend to treat a request as a complete product specification and execute it literally, ignoring the user's real goal and the coherence of the overall experience.

**Core idea**: Treat every request as evidence of a user need, not as the final spec. Before making consequential decisions, build an internal product model — identify the user and their goal, determine the primary task, model the states and transitions, evaluate the full journey — and only then choose the implementation, rather than applying blanket rules.

**When to use**: Designing, implementing, modifying, or reviewing user-facing software. Use this Skill when you want the Agent to do more than "finish the requirement" and actually produce a coherent, usable, well-prioritized experience.

## How to Use

### Integrate into a Supported Agent Platform

For OpenAI-compatible Agent configs, see [skills/build-user-centered-products/agents/openai.yaml](./skills/build-user-centered-products/agents/openai.yaml):

```yaml
interface:
  display_name: "用户中心产品开发"
  short_description: "从用户目标、任务主次、状态变化与恢复路径出发设计和实现软件产品"
  default_prompt: "Use $build-user-centered-products to turn this request into a coherent, user-centered product experience."
```

Drop the corresponding skill directory into your Agent's skills load path, or reference `$build-user-centered-products` directly in a task.

### Read It Directly

Even without an Agent platform, you can read [SKILL.md](./skills/build-user-centered-products/SKILL.md) directly and use its reasoning framework as a checklist when designing or reviewing software.

## Contributing

This is a continuously growing personal project. As I encounter more typical problems in Agent development, new Skills will be added. If you also develop with Agents and want to share your experience, feel free to open an issue or PR.

## License

[Apache License 2.0](./LICENSE)
