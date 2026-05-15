# 新版本说明

## v1.0.18 — Tree-sitter AST 扫描器（2026-05-13）

### 概述


### Tree-sitter AST 扫描器

扫描器为每个 C/C++ 源文件构建完整的 tree-sitter 解析树，并通过类型化访问器函数遍历：

- **FixedSizeBuffer 精度** — 缓冲区规则区分固定大小栈数组与动态分配。
- **词法回退** — 解析失败的文件自动回退到词法扫描器。

### `--experiment-scan`

| 模式 | 发现数 | 误报率 |
|------|--------|--------|

```bash

```

### Phase 24 调优

17 项针对性规则变更减少误报 89,479 条：

注：cppcheck 已移除。`--cppcheck-path` 不再可用。

---

## v1.0.17 — 高级 C/C++ SAST 与 AUTOSAR 版本提取（2026-05-11）

### 概述

v1.0.17 在内部 SAST 扫描器中增加了参数值检测和 cppcheck 子进程集成，新增适用于 CI/IDE 流水线的 SARIF 2.1 输出，并修复了三个 AUTOSAR 准确性问题：完整的 arxml 依赖解析、从 `.epd` 文件和 Doxygen 头文件提取组件版本，以及对链接器标志发现的库进行正确的生态系统分类。

### 参数值匹配

词法扫描器现在在调用位置检查参数值，而不仅仅是函数名称，以检测无法从被调用函数名称推断的误用：

|-----|------|------|

### cppcheck 集成

安装 cppcheck 后，扫描器将其作为子进程调用，并将数据流支持的发现项与词法结果合并：

- cppcheck 输出从其 XML 报告格式解析
- 优雅降级：若 cppcheck 未在 `PATH` 或 `--cppcheck-path` 中找到，则使用仅词法结果并在报告中注明
- 使用 `--cppcheck-path <PATH>` 指定非默认二进制文件位置

### SARIF 2.1 输出

每次内部版本扫描现在都会在 Markdown 报告旁写出 `<project>_static_analysis.sarif` 文件，与 GitHub Code Scanning（`upload-sarif` action）、VS Code SARIF Viewer 及任何支持 SARIF 2.1 的 CI 流水线兼容。使用 `--sarif-output <PATH>` 覆盖默认输出路径。

### SARIF 基线对比

`--sarif-baseline <PATH>` 标志接受先前的 SARIF 运行结果并抑制其中已出现的发现项，仅报告**新**发现项，适用于 PR 级 CI 门控。匹配使用由规则 ID + 文件 URI + 起始行计算的 SHA-256 指纹。

### AUTOSAR arxml 依赖解析

`.arxml` 文件现已完整解析以提取组件间依赖关系，提取三种 AUTOSAR 元素类型：

| 元素 | 依赖类型 |
|------|---------|
| `SW-COMPONENT-PROTOTYPE` | SWC 间组合 |
| `BSW-MODULE-DESCRIPTION` | BSW 模块引用 |
| SWC 类型定义元素（`APPLICATION-SW-COMPONENT-TYPE` 等） | 组件类型声明 |

### AUTOSAR 版本提取

三个来源现在组合使用，替换 AUTOSAR 组件的 `unspecified` 版本字符串：

- **`.epd` 文件（BSW 模块）**：标准 ECUC 模块定义文件，包含 `ADMIN-DATA/DOC-REVISIONS` 下的 `ECUC-MODULE-DEF/REVISION-LABEL`
- **Doxygen 风格 C/H 头文件（SWC 目录）**：匹配 `^\s*\*\s+SW Version\s*:\s*(\S+)` 模式，按父目录名称分组
- **版本解析优先级**：`.epd` → Doxygen 头文件 → `"unspecified"`
- **生态系统升级**：链接器标志发现的 `system` 生态系统条目（如 `-lAdc`、`-lMcu`）在找到匹配的 epd/Doxygen 版本后升级为 `autosar` 生态系统



---


### 概述



|--------|------|

### 输出


### 回退模式

当未找到清单文件派生的组件目录但扫描根目录下存在 C/C++ 源文件时，扫描器自动插入合成的 `(project_name, "C/C++") → scan_root` 条目，使无清单仓库也能被扫描。

### 内部功能开关

扫描器及相关格式化器仅在 `--features internal` 构建时编译。`build-all.sh` 新增 `--internal` 标志，同时生成公开版本和内部版本（内部版本文件名添加 `-internal` 后缀）。

---


### 概述


### AUTOSAR 项目检测

扫描器在主目录遍历前运行 `detect_autosar()` 预扫描，按顺序检查三个信号（首次匹配即短路）：

| 信号 | 触发条件 |
|------|---------|
| DET-01 | 任意深度存在 `.arxml` 文件 |
| DET-02 | 存在名为 `BSW`、`MCAL`、`RTE`、`AUTOSAR` 或 `SWC` 的目录 |
| DET-03 | 根目录或一级子目录的 CMake 或 Makefile 中存在 `AUTOSAR_VERSION` 或 `AR_VERSION` 令牌 |

任一信号触发后，`ScanContext.is_autosar` 设为 `true`，AUTOSAR 分类流程自动启动。

### AUTOSAR 组件分类

`classify_autosar_components()` 将发现的依赖名称与内置 BSW 模块配置（可通过 `--bsw-config <path>` 覆盖）进行比对，匹配的组件将被标注：

- **`layer`** — 如 `BSW-Memory`、`BSW-SystemServices`、`BSW-Communication`
- **`platform`** — `Classic` 或 `Adaptive`

### AUTOSAR 输出 — CycloneDX

```json
{
  "name": "NvM",
  "properties": [
    { "name": "autosar:layer",    "value": "BSW-Memory" },
    { "name": "autosar:platform", "value": "Classic" },
    { "name": "autosar:supplier", "value": "Vector Informatik" }
  ]
}
```

### AUTOSAR 输出 — SPDX

```
ExternalRef OTHER autosar:layer BSW-Memory
ExternalRef OTHER autosar:platform Classic
ExternalRef OTHER autosar:supplier Vector-Informatik
```

### 供应商映射（`--supplier-config`）

```yaml
NvM: "Vector Informatik"
CanIf: "ETAS"
Det: "In-house"
```

已映射组件输出供应商字符串，未映射组件输出 `NOASSERTION`，非 AUTOSAR 组件不受影响。YAML 文件缺失或格式错误将导致硬性报错。



**SPDX — `SECURITY` ExternalRef：**
```
```

**CycloneDX — `cwes` 数组：**
```json
```


### 单一格式支持 `--output`

```bash
# 将 SPDX JSON 写入文件
radeis_sc2sbom --path ./project --format spdx-json --output ./out

# 省略 --output 则输出到标准输出（保持原有行为）
radeis_sc2sbom --path ./project --format cyclonedx-json
```

---

## v1.0.14 - 可靠性与 SBOM 质量（2026-04-24）

### 概述

v1.0.14 是一个由用户报告的真实 C/C++ 项目扫描缺陷驱动的可靠性与 SBOM 质量里程碑。四个阶段关闭了长期存在的缺口：扫描器不再因断开的符号链接而中止，Makefile 变量引用不再泄漏到 `versionInfo`，常见 C/C++ 库现在可以解析到真实的 SPDX 许可证标识符，Linux 发布二进制文件静态链接，使其可在所有支持的发行版上运行而不受 glibc 版本漂移影响。先前失败的扫描现在可端到端干净地完成。

### 断开符号链接容忍

此前扫描路径下任何一处断开的符号链接都会中止整个遍历。扫描器现在会发出警告并继续：

```
Warning: skipping /path/to/broken-link: No such file or directory (os error 2)
```

覆盖全部 5 处 WalkDir 遍历点——主扫描器、`main.rs` 中的回退导入扫描以及 C/C++ 解析器（`makefile.rs`、`mk_file.rs` 等）。`src/util/mod.rs` 中的共享 `warn_on_walkdir_err` 辅助函数替换了 5 个重复的 `filter_map` 闭包。

### Makefile `$(VAR)` 过滤

Makefile 片段经常包含未解析的变量引用，如 `DOPENSSL_VERSION := $(OPENSSL_VERSION)`。本次发布前,这些引用原样泄漏到 SBOM 输出中：

**修复前（v1.0.13）：**
```json
{
  "name": "openssl",
  "versionInfo": "$(OPENSSL_VERSION)",
  "downloadLocation": "https://example.invalid/openssl-$(OPENSSL_VERSION).tar.gz"
}
```

**修复后（v1.0.14）：**
```json
{
  "name": "openssl",
  "versionInfo": "NOASSERTION",
  "downloadLocation": "NOASSERTION"
}
```

过滤器应用于 `mk_file.rs` 解析器层,同时在每个版本输出点（`version_info_or_noassertion`、`generate_cpe_identifier`、`create_download_location`、`create_package_url` 和 CycloneDX 组件版本字段）进行纵深防御守护。任何匹配 `$(...)` 的值都会被替换为 `NOASSERTION` 而不是被输出。

### C/C++ 许可证解析

原生 C/C++ 依赖项此前默认为 `NOASSERTION`,因为 `Makefile` 和 `pkg-config` 文件都没有一致地公开标准许可证字段。v1.0.14 增加两条解析路径：

1. **`.pc` 的 `License:` 解析** — 当 pkg-config 文件带有 `License:` 行时,会被解析并提升为 `licenseConcluded`。
2. **`known_licenses.rs` 表** — 针对 24 个常见系统库的精选查找表（openssl → Apache-2.0、zlib → Zlib、libcurl → curl、libssh2 → BSD-3-Clause、ncurses → X11 等）。在 `makefile.rs`、`mk_file.rs`、`pkgconfig.rs` 和 `pkgconfig_detector.rs` 中用作回退。

`openssl@3.0.7` 的示例输出现在如下所示：

```json
{
  "name": "openssl",
  "versionInfo": "3.0.7",
  "licenseConcluded": "Apache-2.0",
  "licenseDeclared": "Apache-2.0"
}
```

而不是两个许可证字段都为 `NOASSERTION`。

### 通过 musl 实现的静态 Linux 二进制文件

Linux 发布二进制文件现在以 `x86_64-unknown-linux-musl` 为目标,而非 `x86_64-unknown-linux-gnu`。此前该二进制文件携带来自构建主机的 GLIBC 2.39 依赖,在 Ubuntu 22.04 上会报 `version GLIBC_2.39 not found` 错误。musl 静态链接的二进制文件没有 glibc 依赖,可在任何 x86_64 Linux 上运行——Ubuntu 22.04+、24.04、Alpine、Debian、RHEL 等。向 `build-all.sh` 添加了 musl 交叉链接器工具链守卫,CI 的 Ubuntu runner 配置为提供 `musl-tools`。

### 迁移说明

无 CLI 或输出模式更改。在 v1.0.14 上重新生成的现有 SBOM 在 C/C++ 组件上可能会出现额外的许可证结论字段以及更少的 `$(VAR)` 风格版本字符串——二者都是严格意义上的改进。此前被 glibc 链接二进制文件卡住的 Linux 用户可直接替换为 v1.0.14 musl 二进制文件,无需其他改动。

### 测试目标

已针对最初暴露这四个缺陷的真实 C/C++ 项目进行验证。扫描现在可运行至完成,除了预期的断开符号链接通知外没有其他警告,且输出的 SPDX/CycloneDX 通过模式验证。

详见 [v1.0.14 计划](../plan/v1.0.14_user_reported_bugfixes.md) 了解完整设计细节。

---

## v1.0.13 - 多模态子模型组件（2026-04-14）

### 概述

v1.0.13 将多模态 AI 模型分解为各自的子模型组件。像 `google/gemma-4-E2B-it` 这样的模型现在被表示为一个父模型，包含独立的文本、视觉和音频子模型——每个子模型在 SBOM 中拥有自己的架构元数据。

### 子模型分解

`config.json` 中同时包含 `text_config` 和 `vision_config` 及/或 `audio_config` 的多模态模型会被自动分解：

```
gemma4 (parent — Gemma4ForConditionalGeneration)
├── gemma4_text  (35 layers, 1536 hidden, 8 heads, 262K vocab, 131K context)
├── gemma4_vision (16 layers, 768 hidden, 12 heads, patch_size=16)
└── gemma4_audio  (12 layers, 1024 hidden, 8 heads, conv_kernel=5)
```

### 输出格式支持

- **CycloneDX** — 在父组件内部嵌套 `components` 数组，每个子模型作为 `machine-learning-model`，带有 `radeis:ai:sub_model:*` 属性
- **SPDX** — 子包与父模型之间通过 `CONTAINS` 包含关系连接
- **控制台** — 子模型摘要表，显示模态、模型类型、层数、隐藏层大小、注意力头数、dtype 及模态特定扩展信息

### 守卫条件

子模型仅对真正的多模态模型生成——仅有 `text_config`（但没有 `vision_config` 或 `audio_config`）的纯文本模型不会生成多余的子模型条目。

### 测试覆盖

- 4 项新增 Safetensors 测试：多模态提取、纯文本守卫、无 text_config 守卫、仅视觉+文本（LLaVA 风格）
- 1 项新增 GGUF 测试：通过伴随 config.json 增强子模型

详见 [v1.0.13 计划](../plan/v1.0.13_multimodal_sub_model_components.md) 了解完整设计细节。

---

## v1.0.12 - Safetensors 丰富元数据（2026-04-14）

### 概述

v1.0.12 通过从所有 HuggingFace 伴随配置文件中提取丰富元数据，消除了 GGUF 与 Safetensors SBOM 质量之间的差距。Safetensors 和 GGUF 模型目录现在都能生成深度详细的 SBOM，涵盖架构深度、推理默认值、多模态能力、溯源和适配器检测。

### 伴随文件的丰富元数据提取

- **`config.json` 扩展** — model_type、text_config（hidden_layers、hidden_size、attention_heads、上下文窗口）、多模态检测（vision_config、audio_config）、dtype 回退链
- **`generation_config.json`** — temperature、top_k、top_p 推理参数
- **`tokenizer_config.json`** — processor_class、model_max_length（天文数字值安全截断）
- **`preprocessor_config.json`** — 图像、音频和视频处理器类型，含序列长度和采样率
- **`README.md` frontmatter** — base_model（支持字符串和列表）、license、model_creator、pipeline_tag、quantized_by、prompt_template、tags、languages、datasets
- **`adapter_config.json`** — LoRA/QLoRA 适配器检测，含基础模型引用

### GGUF 伴随文件增强

GGUF 仓库现在也受益于相同的伴随文件解析。二进制 KV 元数据始终优先；伴随文件仅填补空白。Tags、languages 和 datasets 使用来自二进制和 README 两个来源的去重联合合并。

### 输出增强

- **CycloneDX** — 约 25 个新的 `radeis:ai:*` 属性，涵盖架构、多模态、生成、处理器、溯源和适配器元数据
- **SPDX** — sourceInfo 扩展了 model_type、上下文窗口和模态摘要（保持简洁）
- **控制台** — AI Model Details 表格扩展了架构、多模态、生成参数和溯源行

### 安全与边界情况

- 所有伴随文件读取设置 1 MB 上限，防止内存问题
- README.md 文件名大小写不敏感匹配（README.md、readme.md、Readme.md）
- CRLF 换行符规范化，支持 Windows 生成的文件
- 天文数字的 model_max_length 值（如 Gemma-4 中的 1e30）安全丢弃
- prompt_template 限制为 512 字符（存储）和 80 字符（控制台显示）

### 测试覆盖

- `safetensors_tests.rs` 新增 8 项测试 — 伴随文件解析、多模态检测、dtype 回退链、model_max_length 截断
- `gguf_tests.rs` 新增 6 项测试 — config.json 提取、README frontmatter（字符串和列表 base_model、CRLF、小写文件名）、适配器检测、tags 联合合并

详见 [v1.0.12 计划](../plan/v1.0.12_safetensors_rich_metadata.md) 了解完整设计细节。

---

## v1.0.11 - Safetensors AI 模型 SBOM（2026-04-13）

### 概述

v1.0.11 新增 Safetensors AI 模型支持，使 `radeis_sc2sbom` 覆盖现代 Transformer 模型（LLaMA、Mistral、Falcon 等）的主流格式。模型在目录级别扫描——无论模型拆分为多少个分片，都只生成一条 Dependency 记录，并准确反映总大小、dtype 和架构元数据。

### Safetensors AI 模型支持

- **文件检测** — 从 `.safetensors`、`model.safetensors.index.json` 和 `config.json` 扫描元数据
- **目录级去重** — 多分片模型（如 `model-00001-of-00002.safetensors`）按模型目录合并为一条 SBOM 记录
- **CycloneDX 输出** — 使用 `machine-learning-model` 组件类型，附带包含架构、dtype 和大小元数据的 `modelCard`
- **SPDX 输出** — 生成 `pkg:huggingface` PURL，用于在 HuggingFace 生态系统中标识模型
- **新增 `AIModelMetadata` 字段**：`safetensors_format`、`total_size_bytes`、`shard_count`、`torch_dtype`、`transformers_version`、`vocab_size`

### 测试覆盖

- `tests/parser_tests/safetensors_tests.rs` 新增 12 项测试，涵盖单分片、多分片、基于索引文件和 config.json 驱动等场景

---

## v1.0.10 - Java Complete（2026-04-13）

### 概述

v1.0.10 将 Gradle 从仅检测升级为完整的依赖解析，支持 Groovy DSL（`build.gradle`）和 Kotlin DSL（`build.gradle.kts`）。这对于使用基于 Android 的机器人控制器和 Android 设备上边缘 AI 推理的 Physical AI 项目至关重要。

### Gradle 依赖解析

- **Groovy DSL**（`build.gradle`）— 解析字符串记法（`'group:artifact:version'`）、映射记法（`group: 'g', name: 'a', version: 'v'`）、平台/BOM 声明
- **Kotlin DSL**（`build.gradle.kts`）— 解析函数记法（`implementation("group:artifact:version")`）和平台 BOM
- **作用域分类** — `testImplementation` → 测试、`compileOnly` → 已提供、`annotationProcessor`/`kapt`/`ksp` → 构建，置信度 1.0
- **Android 支持** — `androidTestImplementation` 和 `androidTestCompile` 正确分类为测试依赖
- **PURL 格式** — `pkg:maven/{group}/{artifact}@{version}`（与 Maven 相同）

详见 [v1.0.10 计划](../plan/v1.0.10_gradle_support.md) 了解完整设计细节。

---

## v1.0.9 - Physical AI Ready（2026-04-10）

### 概述

v1.0.9 新增 GGUF AI 模型支持，使 `radeis_sc2sbom` 成为首个原生支持 AI 模型二进制解析与完整性验证的 SBOM 生成器。本版本还通过合并 C/C++ 构建系统标志简化了 CLI。

### GGUF AI 模型支持

- **二进制解析器**直接从 `.gguf` 文件提取元数据 — 架构、量化类型、张量布局、上下文长度及嵌入的许可证信息
- **CycloneDX 输出**使用 `machine-learning-model` 组件类型，并附带包含训练参数和数据集引用的 `modelCard`
- **SPDX 输出**生成 `pkg:huggingface` PURL，用于在 Hugging Face 生态系统中标识模型
- 使用 `--scan-ai-models true` 启用

### AI 模型完整性验证

- **张量参数交叉验证** — 声明的张量数量和维度将与实际二进制布局进行验证，以检测截断或损坏的模型文件
- **SHA-256 哈希** — 为每个模型文件生成内容哈希，用于供应链真实性检查

### CLI 简化

- 5 个独立的 C/C++ 标志（`--scan-cmake`、`--scan-pkgconfig`、`--scan-autotools`、`--scan-makefiles`、`--scan-mk-files`）合并为 `--scan-c-build-systems`
- `--meson-parse-subprojects` 已移除（启用 `--scan-meson` 时始终生效）
- `--resolve-system-deps` 已移除（无效代码）
- `scan_directory()` 参数从 20 个精简至 13 个

详见 [v1.0.9 计划](../plan/v1.0.9_physical_ai_ready.md) 了解完整设计细节。

---

## v1.0.7 - 漏洞扫描改为可选启用（2026-03-12）

### 概述

漏洞扫描现已改为可选启用。默认情况下，`radeis_sc2sbom` 生成干净的 SBOM，不产生任何网络请求。这使工具运行更快，更适合离线/隔离网络环境以及只需要依赖清单的 CI/CD 流水线。

### 变更内容

  - 此前漏洞扫描会自动运行
- **默认输出更简洁**
  - 控制台输出中隐藏漏洞摘要行
  - Markdown 报告中省略风险评估章节

### 迁移指南

```bash
# v1.0.7 之前（漏洞扫描自动运行）
./radeis_sc2sbom --path .

# v1.0.7+ 扫描漏洞（显式启用）

# v1.0.7+ CI/CD：检测到高危/严重漏洞时终止
./radeis_sc2sbom --path . \
```

---

## v1.0.6 - 生产环境 SBOM 过滤与自动依赖范围分类（2026-03-04）

### 概述

新增自动依赖范围分类功能，可生成仅包含运行时实际所需软件包的生产就绪 SBOM。新增的 `--production` 标志和 `--scope-filter` 选项可在不损失准确性的前提下大幅缩减 SBOM 体积。

### 主要功能

#### 自动范围分类
- **6 种范围类型**：Runtime、Build、Test、Development、Optional、Provided
- **多启发式引擎**：结合基于生态系统、名称和目录的规则
- **置信度评分**：0.0–1.0 范围，每项分类附带可读的原因说明
- **支持 10+ 种生态系统**：npm、pip、cargo、SYSTEM、BUILD-CONFIG、GIT-SUBMODULE、MESON-WRAP 等

#### 生产环境过滤
- **`--production`** 标志：仅包含 Runtime 与 Optional 依赖
  - 示例：嵌入式项目从 106 个软件包缩减至 33 个（减少 68.9%）
- **`--scope-filter <SCOPE>`**：选择任意范围类型组合
  - 支持多值：`--scope-filter runtime --scope-filter optional`
- **范围统计**显示在控制台及 Markdown 报告中（各范围的数量及占比）

#### 增强的 SBOM 输出
- **SPDX 2.3**：`primaryPackagePurpose` 字段由范围分类填充
- **CycloneDX 1.5**：`scope` 字段由范围分类填充

#### 已验证的分类精度
| 分类 | 精度 | 示例 |
|---|---|---|
| 构建工具 | 100% | cmake、gcc、ninja、meson |
| 测试框架 | 100% | pytest、jest、gtest、unity |
| 开发工具 | 100% | pylint、black、eslint、prettier |
| 运行时库 | 高 | zlib、curl、openssl、protobuf |

### 测试覆盖
- **609 个测试全部通过**（203 lib + 203 bin + 200 集成 + 3 文档）
- **42 个新增集成测试**，覆盖范围过滤、生产模式和真实场景分类

### 向后兼容性
- 默认行为与 v1.0.5 保持一致——范围分类自动运行，但在未指定 `--production` 或 `--scope-filter` 时不过滤输出。

---

## v1.0.5 - 增强版本提取：双模式 .mk 文件扫描（2026-02-26）

### 概述

解决了从 Makefile 检测到的系统/运行时库出现"未指定版本"的问题，同时支持对构建系统仓库进行独立的 .mk 文件清单解析。采用**智能双模式架构**，无需配置即可自动适应不同的仓库类型。此版本新增了具备双工作模式的全面 .mk 文件解析，以及可选的 .so 二进制扫描（用于构建后版本提取）。

### 问题描述

**v1.0.5 之前：**
```json
{
  "name": "z",
  "version": "unspecified",  // ❌ 无版本信息
  "ecosystem": "system"
}
```

**v1.0.5 之后（启用 .mk 文件扫描）：**
```json
{
  "name": "z",
  "version": "1.3.1",  // ✅ 从 zlib.mk 提取
  "ecosystem": "system",
  "source_file": "... [version from .mk file: 1.3.1]"
}
```

### 主要功能

#### 双模式架构

.mk 文件扫描器根据仓库结构自动选择合适的模式：

**模式 1：版本解析**（应用项目）
- **触发条件：** 存在 Makefile 且通过 `-l` 标志检测到系统库
- **处理方式：** 从 .mk 文件中解析 Makefile 检测到的库的版本
- **生态系统：** "system"（保留 Makefile 检测上下文）
- **示例：** embedded-project 项目——16 个系统库从"unspecified"升级为精确版本
- **适用场景：** 链接系统库的应用项目

**模式 2：独立清单解析**（构建系统仓库）
- **触发条件：** 存在 .mk 文件但无 Makefile（或 Makefile 中未检测到库）
- **处理方式：** 直接从 .mk 文件中的所有 `*_VERSION` 变量创建依赖
- **生态系统：** "BUILD-CONFIG"（表示构建系统源代码）
- **示例：** embedded-toolchains 项目——检测到 35 个 BUILD-CONFIG 软件包，版本覆盖率 100%
- **适用场景：** 定义构建时依赖的构建系统仓库

**自动去重：**
- 当两种模式检测到同一个库时，模式 1（"system"）优先于模式 2（"BUILD-CONFIG"）
- 在保持准确依赖分类的同时防止重复条目
- `deduplicate_dependencies()` 函数中实现了生态系统感知逻辑

#### .mk 文件解析

从嵌入式系统中常见的构建配置文件中提取版本信息。

**发现策略：** 使用 glob 模式 `**/*.mk` 查找仓库**任意位置**的 .mk 文件，不对目录结构作任何假设。

**示例 .mk 文件：**
```makefile
# toolchains/3rd_party/curl/curl.mk（或任意位置）
CURL_VERSION ?= 8.15.0
CURL_NAME := curl-$(CURL_VERSION)
LIBCURL_SO := $(LIBCURL).4.8.0
```

**映射策略：**
1. 从 Makefile 检测库：`-lcurl` → 库名："curl"
2. 使用 glob 搜索 .mk 文件：`**/*.mk`（在仓库任意位置查找 curl.mk）
3. 解析所有 .mk 文件并提取 `CURL_VERSION ?= 8.15.0`
4. 将版本变量映射到库：`CURL_VERSION` → "curl" → "8.15.0"
5. 更新依赖版本："curl" @ "8.15.0"

**支持的模式：**
- `VAR_VERSION ?= value`（条件赋值）
- `VAR_VERSION := value`（简单赋值）
- `VAR_VERSION = value`（递归赋值）

**库名规范化：**（仅模式 1——用于解析 Makefile 的 `-l` 标志）
- `z` → `zlib`
- `ssl` → `openssl`
- `ssh2` → `libssh2`
- `pcap` → `libpcap`
- `xml2` → `libxml2`
- `pthread` → `pthreads`
- `m` → `libm`
- `dl` → `libdl`
- `rt` → `librt`
- `jpeg` → `libjpeg`
- `png` → `libpng`
- 通用规则：`foo` → `libfoo`（两种形式均尝试）

**构建工具过滤：**（仅模式 2——防止误报）
- 过滤构建工具：make、cmake、gcc、clang、python、perl、ruby、autoconf、automake、libtool、ninja、meson、bash、sh、awk、sed
- 防止 SBOM 中出现"make@4.3"或"cmake@3.25.0"等误报依赖

#### .so 二进制扫描（优先级 2）

从已构建的库二进制文件中提取版本（构建后方式）。

**技术手段：**
1. **解析 .so 文件名：** `libcurl.so.4.8.0` → 版本"4.8.0"
2. **读取 ELF soname：** `readelf -d libcurl.so | grep SONAME`（如 readelf 可用）
3. **提取版本字符串：** 在二进制内容中搜索版本模式

**搜索目录：**
- `lib/`
- `lib64/`（64 位库）
- `build/`
- `build/lib/`（CMake 外部构建）
- `toolchains/install/lib/`
- `usr/lib/`
- `usr/lib64/`（64 位系统库）
- `usr/local/lib/`（本地安装）
- `.libs/`（autotools）

**符号链接去重：**
- 自动解析符号链接链（例如 `libcurl.so` → `libcurl.so.4` → `libcurl.so.4.8.0`）
- 使用规范路径防止重复扫描同一库
- 确保跨符号链接库的版本报告一致

**限制：** 要求库已完成构建（不适用于纯源码仓库）。

### CLI 标志

```bash
# .mk 文件版本提取（默认启用）
--scan-mk-files=true/false     # 默认：true

# .so 二进制版本提取（默认禁用，要求已构建的库）
--scan-so-files=true/false     # 默认：false
```

**默认值说明：**
- `--scan-mk-files=true`：对纯源码仓库安全，开销低，适用于任意 .mk 文件位置
- `--scan-so-files=false`：要求已构建的库，CI/CD 中可能不存在

### 实际效果

**embedded-project 项目：**
- **v1.0.5 之前：** 32 个系统库版本为"unspecified"
- **v1.0.5 之后：** 32 个系统库从 .mk 文件中提取了精确版本
  - curl @ 8.15.0
  - elfutils @ 0.191
  - zlib @ 1.3.1
  - openssl @ 3.2.5
  - libssh2 @ 1.11.0
  - 以及另外 27 个……

### 向后兼容性

✅ **100% 向后兼容**
- 无 .mk 文件的现有项目：行为不变
- 默认的 `--scan-mk-files=true` 仅影响基于 Makefile 的项目
- 无 .mk 文件的项目继续显示"unspecified"版本
- 版本解析为增量式：不替换任何已有版本

### 测试

- **154 个单元测试通过**——包含全部现有测试及新增的 .mk 和 .so 解析器测试
- **3 个新增集成测试**——模式 1/模式 2 去重、构建工具过滤、多文件场景
- **全面代码审查**——所有问题已处理（去重逻辑、符号链接处理、目录遍历优化）
- **集成测试**覆盖 embedded-project 和 embedded-toolchains 项目结构
- **零回归**于现有解析器

### 修改的文件

**新增文件：**
- `src/parsers/c/mk_file.rs` - 带版本提取的 .mk 文件解析器
- `src/parsers/c/so_scanner.rs` - 带版本提取的 .so 二进制扫描器

**修改文件：**
- `src/parsers/c/makefile.rs` - 为模式 1 添加版本解析逻辑
- `src/parsers/mod.rs` - 为模式 1/模式 2 添加生态系统感知去重
- `src/cli.rs` - 添加 `--scan-mk-files` 和 `--scan-so-files` 标志
- `src/scanner/mod.rs` - 将新标志传递给 Makefile 解析器 + 模式 2 触发
- `src/main.rs` - 将 CLI 标志接入扫描器
- `Cargo.toml` - 添加 `glob` 依赖以发现 .mk 文件
- `tests/parser_tests/c_tests.rs` - 添加去重和过滤的集成测试

### 未来改进计划（v1.0.6+）

1. **智能 .mk 模式检测** - 从多个项目中学习 .mk 文件模式
2. **pkg-config .pc 生成** - 从 .mk 文件生成 .pc 文件供其他工具使用
3. **构建系统插件架构** - 通过插件支持自定义构建系统
4. **版本约束解析** - 将">=3.0"等约束解析为实际版本
5. **从 .mk 文件提取许可证** - 从构建配置中提取许可证信息

---

## v1.0.4 - Meson 与 Bazel 构建系统支持（2026-02-25）

### 概述

新增对现代 C/C++ 构建系统（Meson 和 Bazel）的支持，完善了 radeis_sc2sbom 对 C/C++ 生态系统的全面覆盖。结合 v1.0.0–1.0.3（vcpkg、Conan、CMake、Git 子模块、Autotools、pkg-config、Makefile），radeis 现已支持约 **95% 的 C/C++ 项目**。

### 主要功能

#### Meson 构建系统支持

解析 `meson.build` 文件中的 `dependency()` 声明：

```python
# meson.build 示例
project('myapp', 'cpp')

# 带版本约束的 dependency()
zlib_dep = dependency('zlib', version: '>=1.2.11')

# 不带版本的 dependency()
openssl_dep = dependency('openssl')

# 通过 find_library() 引用系统库
cc = meson.get_compiler('c')
math_dep = cc.find_library('m')

# 子项目引用
libfoo_proj = subproject('libfoo')

executable('myapp', 'src/main.cpp',
  dependencies: [zlib_dep, openssl_dep, math_dep])
```

**支持的功能：**
- 从 `dependency()` 声明中提取库名
- 从 `version:` 参数中提取版本约束（如有）（>=、==、>、<、!=）
- 从 `cc.find_library()` 调用中提取系统库
- 从 `subproject()` 调用中检测子项目引用（实际解析通过 `.wrap` 文件进行）
- PURL 格式：`pkg:generic/{name}@{version}?type=meson`（有版本时包含版本）

**说明：** 解析器当前提取依赖名称和版本约束。`modules:` 数组和 `required:` 标志在语法层面可识别，但目前未捕获到结构化输出中。

**真实项目验证：**
- OpenStudio 项目在 conan.lock 中将 meson 1.2.2 作为开发依赖
- 已成功在生产扫描中检测并验证
- 证明了对正在迁移至 Meson 的 C++ 项目的即时适用性

#### Bazel 构建系统支持

解析 `WORKSPACE`/`WORKSPACE.bazel` 和 `MODULE.bazel` 文件中的外部依赖：

```python
# WORKSPACE 示例
http_archive(
    name = "com_google_googletest",
    urls = ["https://github.com/google/googletest/archive/release-1.12.1.tar.gz"],
    strip_prefix = "googletest-release-1.12.1",
)

git_repository(
    name = "com_google_absl",
    remote = "https://github.com/abseil/abseil-cpp.git",
    tag = "20230802.1",
)

# MODULE.bazel 示例（Bazel 6.0+ bzlmod）
module(name = "myproject")

bazel_dep(name = "abseil-cpp", version = "20230802.1")
bazel_dep(name = "googletest", version = "1.14.0")
```

**支持的功能：**
- 解析 `WORKSPACE`/`WORKSPACE.bazel` 中的 `http_archive`、`git_repository` 和本地仓库
- 解析 `MODULE.bazel`（Bazel 6.0+ bzlmod）中的 `bazel_dep()` 声明
- 从外部依赖声明中提取 URL 和版本
- 支持多行依赖声明
- PURL 格式：`pkg:generic/{name}@{version}?type=bazel`，或对基于 git 的依赖使用 `pkg:github/{owner}/{repo}@{version}?type=bazel`

### CLI 标志

```bash
# 启用/禁用 Meson 和 Bazel 支持（默认均启用）
--scan-meson=true/false        # meson.build 和 .wrap 文件
--scan-bazel=true/false        # WORKSPACE、WORKSPACE.bazel、MODULE.bazel 文件
```

### 覆盖范围影响

**v1.0.4 之前：**
- 现代 C++（vcpkg、Conan、CMake）：约 70–80%
- 遗留 C（Autotools、pkg-config、Makefile）：约 80–90%
- 综合覆盖率：约 **90% 的 C/C++ 项目**

**v1.0.4 之后：**
- 现代 C++（vcpkg、Conan、CMake、Meson、Bazel）：约 75–85%
- 遗留 C（Autotools、pkg-config、Makefile）：约 80–90%
- 综合覆盖率：约 **95% 的 C/C++ 项目**

### 真实项目支持

已成功测试的项目：
- **OpenStudio**（Conan）——检测到 meson 1.2.2 作为开发依赖
- **单元测试**——105 个测试通过（Meson 和 Bazel 解析器）

### 生产环境验证

**OpenStudio 扫描结果（v1.0.4）：**
- 共 49 个软件包（48 个 Conan + 1 个 Python）
- 在 conan.lock 中检测到 meson 1.2.2 作为开发依赖
- 安全扫描结果干净（0 个漏洞）
- 完整解析 conan.lock，含开发依赖分类

这验证了 v1.0.4 的 Meson 支持对使用现代构建系统的真实 C++ 项目立即适用。

### 向后兼容性

✅ **100% 保持兼容** - 现有生态系统解析器无回归：
- 全部 105 个单元测试通过
- curl 结果与 v1.0.3 完全一致（已验证）
- npm、Python、ROS、Conan、Git 子模块、CMake、Autotools 均保持稳定
- Meson 和 Bazel 解析器仅为新增功能

### 综合对比报告

作为 v1.0.4 验证的一部分，我们已完成针对 6 个多样化仓库的综合对比报告（每份 375–596 行）：

1. **curl**（C 库）- 446 行
2. **nodejs-service**（Node.js）- 444 行
3. **nodejs-project**（多云）- 375 行
4. **OpenStudio**（C++ Conan）- 398 行
5. **mrpt**（机器人 C++）- 596 行
6. **ros2cli**（ROS 2）- 590 行

**合计：** 2,897 行综合分析

**主要发现：**
- **跨所有项目共追踪 2,561 个依赖**
- **4 项其他工具不具备的独特能力：**
  - C/C++ Autotools（curl：29 个库）
  - ROS 2（ros2cli：223 个组件）
  - Git 子模块（mrpt：8 个含 SHA 的子模块）
  - CMake ExternalProject（mrpt：3 个依赖）
- **相比 BlackDuck 3 年节省 22 万–165 万美元**

详见 [scan_reports/COMPARISON_REPORTS_INDEX.md](../scan_reports/COMPARISON_REPORTS_INDEX.md)。

---

## v1.0.3 - C 遗留支持（pkg-config + Autotools + Makefile）（2026-02-24）

### 概述

新增对传统 C 项目构建系统的全面支持，支持使用 GNU Autotools、pkg-config 和纯 Makefile 的遗留 C/C++ 项目生成 SBOM。填补了现代包管理器支持（vcpkg、Conan、CMake）留下的关键空缺，实现了对约 90% C/C++ 项目的覆盖。

### 主要功能

#### pkg-config（.pc 文件）支持

解析 `.pc`（pkg-config）文件以提取系统库依赖：

```
Name: OpenSSL
Version: 3.0.2
Description: Secure Sockets Layer and cryptography libraries
Requires: libcrypto libssl
```

**支持的功能：**
- 从 .pc 文件中提取软件包名称、版本和描述
- 检测 configure.ac 中的 PKG_CHECK_MODULES() 调用
- 检测 Makefile 中的 pkg-config shell 调用
- PURL 格式：`pkg:generic/{name}@{version}?type=pkg-config`

#### Autotools（configure.ac/Makefile.am）支持

解析 GNU Autotools 配置文件中的库依赖：

**configure.ac：**
```bash
AC_CHECK_LIB([pthread], [pthread_create])
AC_SEARCH_LIBS([sqrt], [m])
PKG_CHECK_MODULES([GLIB], [glib-2.0 >= 2.50])
```

**Makefile.am：**
```makefile
myapp_LDADD = -lssl -lcrypto -lpthread
libfoo_la_LIBADD = -lz
```

**支持的功能：**
- 从 AC_CHECK_LIB、AC_SEARCH_LIBS、PKG_CHECK_MODULES 中提取依赖
- 从 Makefile.am 的 LDADD/LIBADD 变量中提取 -l 标志
- 保留 PKG_CHECK_MODULES 中的版本约束
- PURL 格式：`pkg:generic/{name}@{version}?type=autotools`

#### 纯 Makefile 启发式解析器

对手写 Makefile 进行尽力解析：

```makefile
LDFLAGS = -lssl -lcrypto -lpthread -lz
OPENSSL_CFLAGS = $(shell pkg-config --cflags openssl)
```

**支持的功能：**
- 使用正则表达式提取 -l 标志（系统库）
- 检测 pkg-config 调用
- 按库名去重
- PURL 格式：`pkg:generic/{name}@{version}?type=makefile`

**限制：**
- 不支持变量展开（`$(FOO)`）
- 不支持条件块（`ifeq`）
- 在 Autotools 项目中跳过 Makefile 解析

### CLI 标志

```bash
# 启用/禁用 C 遗留支持（默认全部启用）
--scan-pkgconfig=true/false       # .pc 文件和 PKG_CHECK_MODULES
--scan-autotools=true/false       # configure.ac 和 Makefile.am
--scan-makefiles=true/false       # 纯 Makefile（启发式）
--resolve-system-deps=false       # 系统 pkg-config 解析（默认禁用）
```

### 覆盖范围影响

**v1.0.3 之前：**
- 现代 C++（vcpkg、Conan、CMake）：约 70–80%
- 遗留 C（Autotools）：<1%
- 系统库：<1%

**v1.0.3 之后：**
- 现代 C++（vcpkg、Conan、CMake）：约 70–80%
- 遗留 C++（Makefile）：约 40%
- 纯 C（Autotools）：约 60%
- 系统库依赖：约 80%

**综合覆盖率：约 90% 的 C/C++ 项目**

### 真实项目支持

已成功测试的项目：
- **curl**（Autotools）——检测到 openssl、zlib、nghttp2
- **nginx**（Makefile）——检测到 openssl、pcre、zlib
- 标准 C 库（pthread、m、ssl、crypto、z）

### 输出示例

```
PKG-CONFIG (3 packages)
├── openssl @ >=3.0 [direct]
├── OpenSSL @ 3.0.2 [direct]
└── glib-2.0 @ >=2.50 [direct]

AUTOTOOLS (5 packages)
├── ssl @ unspecified [direct]
├── crypto @ unspecified [direct]
├── z @ unspecified [direct]
├── m @ unspecified [direct]
└── pthread @ unspecified [direct]
```

### PURL 示例

- **pkg-config**：`pkg:generic/openssl@3.0.2?type=pkg-config`
- **Autotools**：`pkg:generic/pthread@unspecified?type=autotools`
- **Makefile**：`pkg:generic/ssl@unspecified?type=makefile`

---

## v1.0.2 - Conan C++ 包管理器支持（2026-02-24）

### 概述

新增对 Conan C/C++ 包管理器的完整支持，支持使用 Conan 2.x 的项目生成 SBOM。支持解析锁文件、INI 格式清单和 Python 格式清单，涵盖运行时、构建、工具和测试依赖。

### 主要功能

#### Conan 锁文件解析（conan.lock）

解析 Conan 2.x 锁文件，包含精确版本和配方修订：

```json
{
  "version": "0.5",
  "requires": [
    "zlib/1.2.13#416618fa04d433c6bd94279ed2e93638%1680515805"
  ],
  "build_requires": ["cmake/3.27.0"],
  "tool_requires": ["ninja/1.11.1"],
  "test_requires": ["gtest/1.14.0"]
}
```

**支持的功能：**
- 从引用中提取软件包名称和版本
- 将配方修订哈希作为校验和存储
- 区分运行时、构建、工具和测试依赖
- 同目录下锁文件优先于清单文件

#### Conan 清单解析（conanfile.txt）

解析 INI 格式的 Conan 清单：

```ini
[requires]
zlib/1.2.13
openssl/[>=3.0]
boost/1.82.0

[build_requires]
cmake/3.27.0

[tool_requires]
ninja/1.11.1

[test_requires]
gtest/1.14.0
```

**支持的功能：**
- 版本约束：`[>=3.0]`、`[>1.0 <2.0]`、`[~=1.82]`、`[^1.0]`
- 用户/频道表示：`package/version@user/channel`
- 构建、工具和测试依赖标记为开发依赖

#### Conan Python 清单解析（conanfile.py）

使用正则提取解析 Python 格式的 Conan 清单：

```python
from conan import ConanFile

class MyProjectConan(ConanFile):
    requires = ["zlib/1.2.13", "openssl/3.1.2"]
    build_requires = ["cmake/3.27.0"]

    def requirements(self):
        self.requires("boost/1.82.0")

    def build_requirements(self):
        self.build_requires("doxygen/1.9.8")
```

**支持的模式：**
- 列表格式：`requires = ["dep1", "dep2"]`
- 方法调用：`self.requires("dep")`
- 构建依赖：`build_requires`、`self.build_requires()`
- 工具依赖：`tool_requires`、`self.tool_requires()`
- 测试依赖：`test_requires`、`self.test_requires()`

#### SBOM 输出

```json
{
  "name": "zlib",
  "versionInfo": "1.2.13",
  "externalRefs": [{
    "referenceType": "purl",
    "referenceLocator": "pkg:conan/zlib@1.2.13"
  }],
  "checksums": [{
    "algorithm": "SHA256",
    "checksumValue": "416618fa04d433c6bd94279ed2e93638"
  }],
  "sourceInfo": "conan/lock extractor from conan.lock"
}
```

### 技术实现

**新增文件：**
- [src/parsers/cpp/conan.rs](../src/parsers/cpp/conan.rs) - conan.lock 解析器
- [src/parsers/cpp/conan_manifest.rs](../src/parsers/cpp/conan_manifest.rs) - conanfile.txt/py 解析器
- [tests/parser_tests/conan_tests.rs](../tests/parser_tests/conan_tests.rs) - 10 个测试用例

**集成：**
- 扫描器检测 `conan.lock`、`conanfile.txt`、`conanfile.py`
- 锁文件优先：若存在 `conan.lock` 则跳过清单
- SPDX purl 格式：`pkg:conan/{name}@{version}`
- 支持 CycloneDX 组件

### 测试覆盖

```bash
cargo test conan_tests
```

**10 个测试用例覆盖：**
- 带配方修订的锁文件解析
- 带版本约束的 INI 清单解析
- Python 清单解析（列表和方法格式）
- 版本范围处理
- 用户/频道表示
- 格式错误输入处理
- 空清单处理
- purl 格式生成

### 示例

```bash
# 扫描含 Conan 依赖的项目
./target/release/radeis_sc2sbom --path /path/to/conan-project --format spdx-json

# 输出示例
{
  "packages": [
    {
      "name": "zlib",
      "versionInfo": "1.2.13",
      "externalRefs": [{"referenceLocator": "pkg:conan/zlib@1.2.13"}]
    },
    {
      "name": "cmake",
      "versionInfo": "3.27.0",
      "externalRefs": [{"referenceLocator": "pkg:conan/cmake@3.27.0"}],
      "properties": [{"name": "dev-dependency", "value": "true"}]
    }
  ]
}
```

### 设计决策

1. **锁文件优先**：`conan.lock` 提供精确版本，优先于清单文件
2. **配方修订**：存储在 `checksum_sha256` 字段以供溯源
3. **开发依赖**：构建、工具和测试依赖通过 `is_dev: true` 标记
4. **版本约束**：从清单中原样保留（如 `>=3.0`）
5. **生态系统标识符**：所有 Conan 依赖使用 `"conan"`

---

## v1.0.1 - CMake 支持与递归子模块扫描（2026-02-23）

### 概述

新增静态 CMake 依赖检测（FetchContent/ExternalProject）及 Git 子模块内的递归依赖扫描。无需执行 CMake 构建即可完整发现所有依赖。

### 主要功能

#### CMake 依赖解析

静态解析 CMakeLists.txt 文件以提取 FetchContent_Declare 和 ExternalProject_Add 依赖：

```cmake
# FetchContent_Declare（现代 CMake 3.11+）
FetchContent_Declare(
  json
  GIT_REPOSITORY https://github.com/nlohmann/json.git
  GIT_TAG        v3.11.2
)

# ExternalProject_Add（遗留模式）
ExternalProject_Add(
  zlib
  URL https://zlib.net/zlib-1.2.13.tar.gz
  URL_HASH SHA256=abc123...
)
```

**支持的功能：**
- GIT_REPOSITORY + GIT_TAG 提取
- 基于 URL 的依赖及从 URL 提取版本
- URL_HASH（SHA256）校验和提取
- 跳过含 CMake 变量 `${VAR}` 的条目（无法静态解析）
- 多托管平台 Git URL 解析（GitHub、GitLab、Bitbucket、通用）

**SBOM 输出：**
```json
{
  "name": "nlohmann/json",
  "versionInfo": "v3.11.2",
  "externalRefs": [{
    "referenceType": "purl",
    "referenceLocator": "pkg:github/nlohmann/json@v3.11.2"
  }],
  "sourceInfo": "cmake/fetchcontent extractor from CMakeLists.txt"
}
```

#### 递归子模块扫描

自动扫描 Git 子模块内部的依赖（package.json、Cargo.toml、CMakeLists.txt 等），支持深度限制：

**示例：** 含子模块的项目（子模块内有 npm 依赖）
```
.
├── .gitmodules
├── libs/
│   └── json/          # Git 子模块
│       ├── package.json   # 递归扫描
│       └── CMakeLists.txt # 同样扫描
```

**功能：**
- 检测子模块内所有清单类型（npm、Cargo、vcpkg、CMake 等）
- 支持嵌套子模块（子模块中的子模块）
- 通过 `--submodule-depth` 配置深度限制（默认：3 层）
- 来源标注显示子模块来源

**嵌套依赖的 SBOM 输出：**
```json
{
  "name": "typescript",
  "versionInfo": "5.0.0",
  "ecosystem": "npm",
  "sourceInfo": "javascript/packagejson extractor from libs/json/package.json (submodule: libs/json)"
}
```

#### CLI 增强

**新增标志：**
- `--scan-cmake=<true|false>` - 启用/禁用 CMake 扫描（默认：true）

**现有标志（现已完整实现）：**
- `--submodule-depth=<N>` - 嵌套子模块最大递归深度（默认：3）

**示例：**
```bash
# 启用 CMake 和递归子模块扫描（默认）
./target/release/radeis_sc2sbom /path/to/project

# 禁用 CMake 扫描
./target/release/radeis_sc2sbom /path/to/project --scan-cmake=false

# 将子模块递归限制为 1 层
./target/release/radeis_sc2sbom /path/to/project --submodule-depth=1
```

### 技术细节

#### CMake 解析器实现
- **文件：** [src/parsers/cmake/mod.rs](../src/parsers/cmake/mod.rs)、[src/parsers/cmake/fetchcontent.rs](../src/parsers/cmake/fetchcontent.rs)、[src/parsers/cmake/external_project.rs](../src/parsers/cmake/external_project.rs)
- **模式：** 基于正则的解析，使用 `(?is)` 标志（大小写不敏感 + 多行）
- **限制：** 无法解析 CMake 变量——仅支持静态值

#### 递归扫描实现
- **函数：** [src/scanner/mod.rs](../src/scanner/mod.rs) 中的 `scan_submodule_recursively()`
- **安全性：** 深度限制防止循环引用导致的无限循环
- **覆盖：** 所有现有解析器（npm、Cargo、Python、Go、vcpkg、CMake 等）

### 竞争优势

|---------|----------------|-------------|-----------|
| **CMake FetchContent** | ✅ 静态解析 | ❌ | ✅ 仅构建捕获 |
| **CMake ExternalProject** | ✅ 静态解析 | ❌ | ✅ 仅构建捕获 |
| **子模块内嵌套依赖** | ✅ 递归扫描 | ❌ | ❌ |
| **无需构建** | ✅ 全静态 | ✅ | ❌ CMake 需要构建 |

### 迁移说明

**API 变更：**
- `scan_directory()` 签名新增 `scan_cmake` 参数（影响自定义集成）
- 测试已更新以包含 `scan_cmake` 参数

**破坏性变更：** 对 CLI 用户无破坏性变更（向后兼容）

---

## v1.0.0 - C++ 支持（2026-02-20）

### 概述

首次支持 C++ 生态系统，包含 vcpkg 清单解析和 Git 子模块检测。支持使用现代包管理器的 C/C++ 项目生成 SBOM。

### 主要功能

#### vcpkg 清单解析器

完整支持 vcpkg.json，涵盖所有版本约束格式：

```json
{
  "name": "my-project",
  "dependencies": [
    "zlib",
    { "name": "openssl", "version>=": "3.0" },
    { "name": "boost", "version-semver": "1.82.0", "features": ["filesystem"] },
    { "name": "fmt", "version-date": "2023-01-15" }
  ],
  "overrides": [
    { "name": "zlib", "version": "1.2.13" }
  ]
}
```

**支持的版本格式：**
- 简单字符串依赖：`"zlib"`
- `version>=`：最低版本约束
- `version>`：大于版本
- `version=`：精确版本
- `version-semver`：语义版本
- `version-date`：基于日期的版本
- `port-version`：端口修订号
- overrides 段用于版本固定
- features 存储在 source_file 元数据中

**SBOM 输出：**
```json
{
  "name": "boost",
  "versionInfo": "1.82.0",
  "externalRefs": [{
    "referenceType": "purl",
    "referenceLocator": "pkg:vcpkg/boost@1.82.0"
  }],
  "sourceInfo": "cpp/vcpkg extractor from vcpkg.json [filesystem]"
}
```

#### Git 子模块检测

自动解析 `.gitmodules` 文件并解析提交 SHA：

```ini
[submodule "libs/json"]
    path = libs/json
    url = https://github.com/nlohmann/json.git
    branch = master
```

**功能：**
- 解析子模块名称、路径、URL 和分支
- 通过 `git ls-tree HEAD` 解析提交 SHA
- 支持 HTTPS 和 SSH URL
- 支持多托管平台：GitHub、GitLab、Bitbucket、自托管

**SBOM 输出：**
```json
{
  "name": "nlohmann/json",
  "versionInfo": "bc889af",
  "externalRefs": [{
    "referenceType": "purl",
    "referenceLocator": "pkg:github/nlohmann/json@bc889af"
  }],
  "sourceInfo": "git-submodule extractor from .gitmodules"
}
```

#### 潜在漏洞标记

通过导入扫描（version="detected"）发现的软件包，其漏洞现标记为**潜在**：

```
   Package version 'detected' is unknown (detected via import scanning).
   Actual version may not be affected.
```

这有助于区分：
- **已确认漏洞** - 版本已知的软件包（如 `urllib3@2.6.0`）
- **潜在漏洞** - 通过导入扫描检测到但无版本信息的软件包

#### 版本格式修复

- **修复前：** `"==2.6.0"`（无效的 purl 格式）
- **修复后：** `"2.6.0"`（正确的 purl 格式）

此修复显著减少了误报漏洞数量（37 → 8，以 MRPT 对比为例）。

#### Python 检测方法

radeis 扫描来自清单文件（requirements.txt、pyproject.toml）的**声明依赖**，比扫描已安装环境的工具生成更准确的 SBOM：

| 方式 | radeis | 环境扫描工具 |
|----------|--------|---------------------|
| 检测方法 | 声明依赖 | 已安装软件包 |
| 捕获内容 | 项目需求 | 特定环境的软件包 |
| 准确性 | 更准确 | 可能含误报 |

**关键洞察：** 环境扫描工具可能包含 `importlib-metadata` 和 `zipp`（Python < 3.10 的回溯移植包）等并非实际项目需求的软件包。

### 新增 CLI 选项

```bash
--scan-submodules <BOOL>   # 启用 Git 子模块扫描（默认：true）
--submodule-depth <N>      # 最大递归深度（默认：3）
```

**示例：**
```bash
# 扫描含 vcpkg 的 C++ 项目
radeis_sc2sbom --path ./cpp_project --format spdx-json

# 禁用子模块扫描
radeis_sc2sbom --path . --scan-submodules false

# 限制子模块递归深度
radeis_sc2sbom --path . --submodule-depth 1
```

### purl 格式支持

| 生态系统 | purl 格式 | 示例 |
|-----------|-------------|---------|
| vcpkg | `pkg:vcpkg/{name}@{version}` | `pkg:vcpkg/zlib@1.2.13` |
| GitHub | `pkg:github/{owner}/{repo}@{commit}` | `pkg:github/nlohmann/json@bc889af` |
| GitLab | `pkg:gitlab/{owner}/{repo}@{commit}` | `pkg:gitlab/owner/repo@abc123` |
| Bitbucket | `pkg:bitbucket/{owner}/{repo}@{commit}` | `pkg:bitbucket/owner/repo@def456` |
| 通用 | `pkg:generic/{name}@{version}` | `pkg:generic/custom-lib@1.0.0` |

### 技术变更

**新增文件：**
- `src/parsers/cpp/mod.rs` - C++ 解析器模块入口
- `src/parsers/cpp/vcpkg.rs` - vcpkg.json 解析器
- `src/parsers/git/mod.rs` - Git 解析器模块入口
- `src/parsers/git/submodules.rs` - .gitmodules 解析器
- `src/parsers/git/commit_resolver.rs` - Git 提交 SHA 解析器
- `src/parsers/git/url_parser.rs` - Git URL 解析器

**修改文件：**
- `src/cli.rs` - 新增 CLI 选项
- `src/scanner/mod.rs` - vcpkg.json 和 .gitmodules 检测
- `src/formats/spdx.rs` - vcpkg/git-submodule purl 支持
- `src/formats/console.rs` - 潜在漏洞展示
- `src/parsers/python.rs` - 版本格式修复（去除操作符）
- `src/parsers/mod.rs` - 将 `__future__` 添加至 Python 标准库
- `src/main.rs` - 将新参数传递给扫描器

### 从 v0.9.3 迁移

**无破坏性变更。** 所有现有功能正常工作。

新功能在以下情况自动激活：
- 发现 `vcpkg.json` → 运行 vcpkg 解析器
- 发现 `.gitmodules` → 运行子模块检测（需要 git）

### 未来路线图（v1.0.x）

- v1.0.1：CMake FetchContent/ExternalProject 解析
- v1.0.2：Conan 包管理器支持
- 子模块内递归扫描

---

## v0.9.3 - Python 全面优化（2026-02-10）

### 概述

完整支持 Pipenv 和 pyproject.toml，Python 软件包检测量提升 **487%**。

### 主要改进

#### Pipfile/Pipfile.lock 解析器
完整集成 Pipenv，支持锁文件解析。

**改进前：**
```json
{
  "name": "azure-identity",
  "versionInfo": "detected",
  "sourceInfo": "Import scanner"
}
```

**改进后：**
```json
{
  "name": "azure-identity",
  "versionInfo": "1.12.0",
  "downloadLocation": "https://pypi.org/project/azure-identity/1.12.0/",
  "sourceInfo": "python/pipfilelock extractor"
}
```

**结果（nodejs-service）：**
- 从 Pipfile.lock 检测到 47 个软件包（之前为 8 个）
- 100% 版本准确率（零"@detected"）
- 与 Black Duck 完全一致（47/47）
- 提取 SHA256 校验和

#### pyproject.toml 多格式解析器
自动支持三种格式：

**PEP 621（标准格式）：**
```toml
[project]
dependencies = ["requests>=2.28.0", "click>=8.0.0"]

[project.optional-dependencies]
dev = ["pytest>=7.0"]
```

**Poetry：**
```toml
[tool.poetry.dependencies]
python = "^3.8"
requests = "^2.28.0"

[tool.poetry.dev-dependencies]
pytest = "^7.0"
```

**PDM：**
```toml
[tool.pdm]
dependencies = ["requests>=2.28.0"]
```

#### SHA256 校验和提取
从锁文件中提取校验和以保障供应链安全：
- Pipfile.lock：hashes 数组中的第一个 SHA256
- poetry.lock：files[0].hash 字段中的哈希
- 内部存储（尚未输出到 SPDX/CycloneDX）

#### 重复防止
锁文件现在跳过对应的清单文件：
- 存在 Pipfile.lock → 跳过 Pipfile
- 存在 poetry.lock → 跳过 pyproject.toml

### 性能指标

| 指标 | v0.9.3 | v0.9.2 | 提升幅度 |
|--------|--------|--------|-------------|
| Python 软件包 | 47 | 8 | +487% |
| 真实版本 | 47 | 0 | +47 |
| "@detected"版本 | 0 | 8 | -100% |
| Black Duck 一致率 | 100% | 17% | +83% |

### 竞争定位

| 工具 | 软件包数 | 版本准确率 | 校验和 | 格式 |
|------|----------|------------------|-----------|---------|
| **radeis v0.9.3** | 47 🏆 | 100% 🏆 | ✅ SHA256 🏆 | Pipfile、poetry、pyproject 🏆 |
| Black Duck | 47 | 100% | 未知 | Pipfile、poetry |

### 新增文件支持

**Pipfile.lock（锁文件）**
- 精确版本（`==1.12.0`）
- SHA256 哈希
- 直接依赖检测（`index` 字段）
- 开发依赖（`develop` 段）

**Pipfile（清单）**
- 版本规格（`*`、`>=1.0`、`==1.2.3`）
- 开发依赖
- 多源支持

**pyproject.toml（多格式）**
- PEP 621、Poetry、PDM 格式
- 自动检测
- 版本规格与扩展功能

### 技术变更

**修改文件：**
- `src/parsers/python.rs`（+219 行）- 新增解析器
- `src/scanner/mod.rs`（+30 行）- 解析器注册
- `Cargo.toml` - 版本升至 0.9.3

**关键算法：**
```rust
// Pipfile.lock 结构
#[derive(Deserialize)]
struct PipfileLock {
    default: HashMap<String, PipfilePackage>,
    develop: Option<HashMap<String, PipfilePackage>>,
}

// 校验和提取
fn extract_first_sha256(hashes: &[String]) -> Option<String> {
    hashes.iter()
        .find(|h| h.starts_with("sha256:"))
        .map(|h| h.trim_start_matches("sha256:").to_string())
}

// pyproject.toml 多格式处理
pub fn parse_pyproject_toml(path: &Path) -> Result<Vec<Dependency>> {
    // 依次尝试 PEP 621、Poetry、PDM
    if let Some(project) = pyproject.get("project") { ... }
    if let Some(poetry) = pyproject.get("tool").and_then(|t| t.get("poetry")) { ... }
    if let Some(pdm) = pyproject.get("tool").and_then(|t| t.get("pdm")) { ... }
}
```

### 使用场景

**Pipenv 项目：**
```bash
radeis_sc2sbom --path ./python_project --format spdx-json

# 输出：47 个软件包，100% 版本准确率
```

**现代 Python 项目：**
```bash
radeis_sc2sbom --path ./pyproject_project --format all

# 自动检测 PEP 621、Poetry 或 PDM 格式
```

**供应链审计：**
```bash
radeis_sc2sbom --path ./workspace --format spdx-json

# 从锁文件提取 SHA256 校验和
```

### 从 v0.9.2 迁移

**无破坏性变更。** 所有功能自动生效：

```bash
# v0.9.2：8 个软件包，全为"@detected"
radeis_sc2sbom --path ./nodejs-service --format spdx-json

# v0.9.3：47 个软件包，全为真实版本
# （命令相同，自动改善）
```

**变更内容：**
- ✅ 自动解析 Pipfile/Pipfile.lock
- ✅ 解析 pyproject.toml（所有格式）
- ✅ 提取 SHA256 校验和
- ✅ 100% 版本准确率
- ✅ 零性能影响

### 缺陷修复

1. Python 版本检测——通过锁文件消除"@detected"
2. 校验和提取——新增 SHA256 支持
3. 解析器优先级——锁文件覆盖清单
4. 重复防止——存在锁文件时跳过清单

---

## v0.9.2 - 用户体验优化（2026-02-08）

### 进度指示器

扫描过程中提供实时反馈：

```
[1/5] Walking directory tree... 47 entries scanned
[2/5] Parsing complete... 54 dependencies discovered
[3/5] Deduplicating dependencies... 54 → 47 unique
[5/5] Scan complete
```

### 功能

- 带百分比进度的进度条
- 长时间操作的旋转动画
- 5 阶段流水线可见性
- 预计完成时间

---

## v0.9.1 - ROS 集成（2026-01-28）

### 自动版本解析

ROS 软件包在 `package.xml` 中缺少版本信息。v0.9.1 通过 rosdistro API 实现自动解析。

**改进前：**
```
rclpy @ unspecified
downloadLocation: NOASSERTION
```

**改进后：**
```
rclpy @ 7.1.9 (from rosdistro)
downloadLocation: https://github.com/ros2/rclpy.git
```

### ros2cli 基准测试

- 94 个唯一依赖（对比 BlackDuck 的 4 个）
- 检测到 15 个 ROS 软件包
- 47 个软件包带 GitHub URL
- 62 个软件包有版本号（66%）
- 发现 5 个漏洞

### 功能

- 并行 API 获取（速度提升 10–27 倍）
- SHA-1 校验和
- 自动修复建议
- 紧凑 SPDX 模式（体积减小 30%）

**支持的发行版：** jazzy、iron、humble、rolling、noetic、melodic

---

## v0.9.1.1 - 热修复（2026-01-28）

### 修复：Markdown 报告显示问题

**问题：** ROS 多包报告中出现空代码块

**修复：** 在 ROS 报告章节中展示所有直接依赖

---

## v0.8.0 - 元数据与安全性（2026-01-22）

### 丰富元数据

- 跨生态系统许可证覆盖率 95%+
- 供应商/来源追踪 90%+
- 基于 UUID 的 SPDX ID
- 安全工具使用的 CPE 标识符

### 改进内容

- 并行元数据获取
- 增强的许可证检测
- 改进的供应商追踪
- 更好的 CPE 生成

---

## 未来路线图

### v0.9.4+
- 在 SPDX/CycloneDX 输出中显示校验和
- 增强 PyPI 元数据缓存
- pdm.lock 支持
- conda environment.yml 支持

---

**最新版本：** v1.0.7（2026 年 3 月 12 日）
**状态：** 生产就绪
**Python 支持：** 业界领先 🏆
