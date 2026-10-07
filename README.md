# FantetySkills

**简体中文** | [English](./README.en.md)

在使用 AI Agent 开发和创作时积累的实用 skills，涵盖产品体验、海报设计和像素绘画。将反复遇到的问题与解决方法整理为可复用的指令，供 Codex、Claude Code、Trae 等支持 skills 的 Agent 使用。

## 安装

安装 [Node.js](https://nodejs.org/) 后，在要使用 skill 的项目目录中运行：

```sh
npx skills add https://github.com/Fantety/FantetySkills
```

按照提示选择需要的 skill 和目标 Agent。也可以直接安装单个 skill，选用下面对应的命令：

```sh
npx skills add https://github.com/Fantety/FantetySkills --skill build-user-centered-products
npx skills add https://github.com/Fantety/FantetySkills --skill create-posters
npx skills add https://github.com/Fantety/FantetySkills --skill oy-pixel
```

默认安装到当前项目。添加 `-g` 可在多个项目中使用，添加 `--agent` 可指定 Agent，例如为 Codex 全局安装全部 skills：

```sh
npx skills add https://github.com/Fantety/FantetySkills --skill '*' --agent codex -g
```

安装前查看可用 skills：

```sh
npx skills add https://github.com/Fantety/FantetySkills --list
```

更多 Agent 与安装选项见 [skills CLI 文档](https://github.com/vercel-labs/skills#readme)。

## Skills

| Skill | 用途 |
| --- | --- |
| [build-user-centered-products](./skills/build-user-centered-products/SKILL.md) | 设计、实现和评审面向用户的软件，从真实目标、主要任务和完整流程出发，改善产品体验。 |
| [create-posters](./skills/create-posters/SKILL.md) | 根据文案、图片、Logo 和二维码制作单页海报，处理信息层级、版式与导出，支持固定画布 HTML 和 PNG/PDF 交付。 |
| [oy-pixel](./skills/oy-pixel/SKILL.md) | 创建和编辑像素画、精灵、图标、瓦片、特效及逐帧动画，输出 PNG、GIF、精灵图集或 PNG 帧序列。 |

`build-user-centered-products` 和 `create-posters` 无额外运行依赖。`oy-pixel` 的渲染器需要 Python 3.10+ 和 Pillow 10–12；附带 Windows 预编译程序，其他平台首次运行需要 Rust/Cargo 和 C 编译器。详见 [oy-pixel 使用说明](./docs/oy-pixel.md)。

## 使用

安装后，在 Agent 中描述任务并指定 skill。Codex 中可以这样调用：

```text
$build-user-centered-products 评审当前注册流程，找出妨碍用户完成注册的问题并改进。
$create-posters 根据提供的文案、图片和二维码制作一张活动海报，交付 PNG。
$oy-pixel 制作一个 32×32 的角色行走动画，导出透明背景 GIF 和精灵图集。
```

## 贡献

欢迎通过 Issue 或 PR 分享遇到的问题与可复用的解决方式。添加或修改 skill 请阅读 [贡献指南](./CONTRIBUTING.md)；文件组织约定见 [目录规范](./docs/skill-layout.md)。

## 许可证

本仓库采用 [MIT 许可证](./LICENSE)。第三方依赖的许可证及分发说明见 [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md)。
