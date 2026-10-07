# oy-audio 音效 Skill

独立安装的目录为 `skills/oy-audio/`。它使用 JSON 配方、Python 合成脚本和 NumPy，生成 16-bit PCM WAV，并输出音频质量报告。无需启动原 oy-audio 桌面应用，也无需模型服务、GPU、SoundFont 或 Rust 编译环境。

## 本地试用

```sh
python -m pip install -r skills/oy-audio/scripts/requirements.txt
python skills/oy-audio/scripts/render.py --check
python skills/oy-audio/scripts/render.py skills/oy-audio/assets/examples/impact.json --out output/audio --name impact
python skills/oy-audio/scripts/render.py skills/oy-audio/assets/examples/ambience.json --out output/audio --name ambience
python tests/oy-audio/test_render.py
```

安装后可调用：`$oy-audio Create a family of interaction sounds with a consistent material character; deliver WAV files and editable recipes.` Skill 的 [渲染指南](../skills/oy-audio/references/rendering.md) 定义配方及脚本 API，[声音设计](../skills/oy-audio/references/sound-design.md) 说明如何从需求构造声音。示例覆盖短反馈、冲击、重复拟音和环境循环，可修改为其他类别。

## 方案来源与实现边界

已阅读 oy-audio 项目工作副本中的相关实现与说明：`src-tauri/prompts/craft.md`、`prompts/skills/{ui,foley,nature}/SKILL.md`、`audio_script/src/{lib.rs,prelude.lua}`、`audio_analysis/src/lib.rs` 和相关 Lua 配方。参考时 HEAD 为 `0a6f3eb`，部分提示词含本地未提交修改；实际以读取到的工作副本为准。

提炼的原则是：把声音分为可调整的层；以采样时钟和固定种子实现确定性生成；保留配方与来源；用离线分析及试听形成迭代。这里的 Python 工具为本仓库新实现，不包含原应用的 Tauri、Agent 运行时、Lua 沙箱、事务模型或音轨工具；不兼容其 Lua/项目格式。没有复制原项目的源代码或依赖树。

本次还检查了用户指定的 musci_gen 目录。2026-10-07 时，其中 `main.py`、`PROMPT.md`、`pyproject.toml` 不是可读 UTF-8 文本，读取到的内容呈截断压缩数据；`musicgen/`、`tools/`、`.git/` 目录为空，因此未能核实或采用其实际实现。此 Skill 不声称已经整合该项目；获得可读副本后可继续补充对照。

## 验证范围

行为测试覆盖固定种子的可重复性和变化、采样率/声道/帧数、PCM 峰值、循环拼接时长和边界、尾音溢出、静音/非法输入、源采样导入、拒绝覆盖，以及无原项目依赖的独立目录运行。质量报告不包含 LUFS、真峰值或主观听感判定。示例需要实际试听才能评价审美与写实程度。

Python 脚本是普通可执行代码，不是沙箱。脚本上限检查控制音频配方尺寸，不构成 Python CPU、内存或文件访问隔离。
