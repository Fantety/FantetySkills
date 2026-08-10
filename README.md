# FantetySkills

> 在使用 Agent 进行软件开发时积累的实践经验、常见问题与对应的 Skill 解决方案。

**简体中文** | [English](./README.en.md)

本仓库是个人在使用 AI Agent(Codex / Trae 等)进行开发过程中形成的经验总结。每当在 Agent 协作中遇到反复出现的问题,我会将其抽象为一个可复用的 **Skill**,并把问题背景、解决思路与对应的 Skill 沉淀到这里,便于在后续项目中直接调用、避免重复踩坑。

## 为什么需要

Agent 在开发中能极大提升效率,但也常出现一些"看似完成了需求、实际偏离了用户真实目标"的问题:

- 只满足字面要求,忽视整体体验的连贯性
- 局部实现正确,却让整体流程变得脆弱、难以恢复
- 在不重要的细节上投入过多复杂度,关键路径反而处理粗糙
- 缺乏对用户状态、任务主次的判断,生成通用但无重点的方案

这类问题往往不是模型能力不足,而是缺少一套明确的"产品思考方式"作为引导。FantetySkills 试图通过 Skill 的形式,把这种思考方式固化下来,让 Agent 在每次任务中都能自觉套用。

## 仓库结构

```
FantetySkills/
├── skills/                          # 可复用的 Skill 集合
│   └── build-user-centered-products/
│       ├── SKILL.md                 # Skill 主体说明
│       └── agents/
│           └── openai.yaml          # Agent 接入配置
├── LICENSE                          # Apache 2.0
└── README.md
```

## 已收录的 Skills

### [build-user-centered-products](./skills/build-user-centered-products/SKILL.md)

**解决的问题**:Agent 容易把请求当作完整的产品规格来执行,只满足字面需求,而忽略用户真实目标与整体体验的连贯性。

**核心思路**:把每一次请求都视作用户需求的证据,而非最终规格。在做出重要决策前,先在内部建立产品模型——识别用户与目标、确定主要任务、建模状态与流转、评估完整旅程——然后再据此选择实现方式,而不是套用通用规则。

**适用场景**:面向用户的软件的设计、实现、修改与评审。当你希望 Agent 不只是"做完需求",而是产出一个连贯、可用、重点清晰的体验时,应使用此 Skill。

## 如何使用

### 在支持的 Agent 平台中接入

以 OpenAI 兼容的 Agent 配置为例,参考 [skills/build-user-centered-products/agents/openai.yaml](./skills/build-user-centered-products/agents/openai.yaml):

```yaml
interface:
  display_name: "用户中心产品开发"
  short_description: "从用户目标、任务主次、状态变化与恢复路径出发设计和实现软件产品"
  default_prompt: "Use $build-user-centered-products to turn this request into a coherent, user-centered product experience."
```

将对应 skill 目录放入你所用 Agent 的 skills 加载路径,或在任务中直接引用 `$build-user-centered-products`。

### 直接参考内容

即使不接入 Agent 平台,你也可以直接阅读 [SKILL.md](./skills/build-user-centered-products/SKILL.md),将其中的判断框架作为自己设计、评审软件时的 checklist。

## 贡献

这是一个持续积累的个人项目。随着在 Agent 开发中遇到更多典型问题,会继续补充新的 Skill。

## 许可证

[Apache License 2.0](./LICENSE)
