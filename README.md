# FantetySkills

> 在使用 Agent 进行软件开发时积累的实践经验、常见问题与对应的 Skill 解决方案。

**简体中文** | [English](./README.en.md)

本仓库是个人在使用 AI Agent(Codex / Trae 等)进行开发过程中形成的经验总结。每当在 Agent 协作中遇到反复出现的问题,我会将其抽象为一个可复用的 **Skill**,并把问题背景、解决思路与对应的 Skill 沉淀到这里,便于在后续项目中直接调用、避免重复踩坑。

## 已收录的 Skills

### [build-user-centered-products](./skills/build-user-centered-products/SKILL.md)

**解决的问题**:Agent 容易把请求当作完整的产品规格来执行,只满足字面需求,而忽略用户真实目标与整体体验的连贯性。

**核心思路**:把每一次请求都视作用户需求的证据,而非最终规格。在做出重要决策前,先在内部建立产品模型——识别用户与目标、确定主要任务、建模状态与流转、评估完整旅程——然后再据此选择实现方式,而不是套用通用规则。

**适用场景**:面向用户的软件的设计、实现、修改与评审。当你希望 Agent 不只是"做完需求",而是产出一个连贯、可用、重点清晰的体验时,应使用此 Skill。

### [create-posters](./skills/create-posters/SKILL.md)

根据文案、图片、Logo、二维码和交付要求创建或修改单页海报，关注信息层级、可读性、素材完整性和导出质量。适用于活动海报、宣传图、信息海报，以及固定画布 HTML 和 PNG/PDF 交付。

### [oy-pixel](./skills/oy-pixel/SKILL.md)

创建和编辑像素画、精灵、图标、瓦片、特效与逐帧动画。通过 JSON 配方和 Lua 绘图脚本，输出 PNG、GIF、精灵图集或 PNG 帧序列；包含 Python 渲染入口、Rust/Lua 引擎和 Windows 预编译程序。

环境要求、渲染示例和验证命令见 [oy-pixel 使用与开发说明](./docs/oy-pixel.md)。

## 安装与使用

每个 `skills/<skill-name>/` 都是一个可独立安装的 skill。将所需 skill 的**完整目录**复制到 Agent 支持的 skills 目录；Codex 默认使用 `~/.codex/skills/`（Windows 为 `%USERPROFILE%\.codex\skills\`；设置了 `CODEX_HOME` 时使用其下的 `skills/`）。

例如，安装 oy-pixel 后应存在 `~/.codex/skills/oy-pixel/SKILL.md`，并保留同目录下的 `scripts/`、`references/`、`assets/` 和许可证文件。在支持显式调用的 Agent 中，可以使用 `$oy-pixel`、`$create-posters` 或 `$build-user-centered-products`。

两个指导型 skill 无额外运行依赖。oy-pixel 需要 Python 3.10+ 和 Pillow 10–12；仓库附带 Windows 渲染程序，其他平台首次运行需要 Cargo 和 C 编译器。

## 目录结构

```text
FantetySkills/
├── README.md / README.en.md      仓库说明与 skill 索引
├── LICENSE                      默认 Apache-2.0 许可证
├── THIRD_PARTY_NOTICES.md        引入组件与许可证范围
├── CONTRIBUTING.md              贡献与验证指南
├── .github/workflows/           持续集成
├── docs/                        仓库级使用与开发文档
├── scripts/                     仓库维护工具
├── tests/<skill-name>/          自动化测试
├── requirements-dev.txt         开发与验证依赖
└── skills/
    ├── build-user-centered-products/
    ├── create-posters/
    └── oy-pixel/
```

## 贡献

这是一个持续积累的个人项目。欢迎通过 Issue 或 PR 分享遇到的问题与可复用的解决方式。添加或修改 skill 前，请阅读 [贡献指南](./CONTRIBUTING.md)。

```sh
python -m pip install -r requirements-dev.txt
python scripts/validate_skills.py
```

涉及渲染行为或引擎的修改，还需要运行贡献指南中的 Python 与 Rust 测试。

## 许可证

除另有声明外，本仓库采用 [Apache License 2.0](./LICENSE)。`skills/oy-pixel/` 及其渲染测试 `tests/oy-pixel/` 保留原项目的 [MIT 许可证](./skills/oy-pixel/LICENSE)。依赖及分发说明见 [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md)。
