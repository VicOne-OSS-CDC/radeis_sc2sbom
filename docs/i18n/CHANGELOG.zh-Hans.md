# 更新日志

本文件记录项目所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)，
本项目遵循 [语义化版本](https://semver.org/spec/v2.0.0.html)。

## [未发布]

## [1.0.18] - 2026-05-13

### 新增（内部版本）

### 变更
- **17 项规则级别调优变更**：发现总数从 217,279 → 127,800；误报率从 88.3% → 82.2% 提升（Phase 24）

### 移除
- **cppcheck 集成与 `--cppcheck-path` 标志**：完全由 AST 扫描器替代（Phase 19）

## [1.0.17] - 2026-05-11

### 新增（内部版本）
- **`--cppcheck-path <PATH>`**：覆盖 cppcheck 二进制文件的 PATH 查找
- **SARIF 2.1 输出**：`<project>_static_analysis.sarif` 与 `_static_analysis.md` 并排写出；兼容 GitHub Code Scanning、VS Code Problem Matcher 和 CI/CD 流水线
- **`--sarif-output <PATH>`**：将 SARIF 报告写入自定义路径
- **SARIF 指纹**：每个 `SarifResult` 的 SHA-256 指纹，用于跨运行的稳定去重
- **`--sarif-baseline <PATH>`**：将当前发现项与先前 SARIF 运行对比；仅报告新发现项——实现仅报告回归的 CI 门控
- **AUTOSAR arxml 解析**：`.arxml` 文件现已完整解析依赖关系——提取 `SW-COMPONENT-PROTOTYPE`、`BSW-MODULE-DESCRIPTION` 及所有 SWC 类型定义元素作为 `autosar` 生态系统依赖
- **AUTOSAR 版本提取**：`.epd` 文件（`ECUC-MODULE-DEF/REVISION-LABEL`）和 Doxygen C/H 头文件（`SW Version : X.Y.Z`）扫描，为 AUTOSAR 依赖填充真实版本字符串而非 `unspecified`
- **AUTOSAR 生态系统升级**：Makefile `-lFoo` 标志发现的 `system` 生态系统依赖在找到匹配的 `.epd` 或 Doxygen 版本时升级为 `autosar` 生态系统——消除重复条目和误分类的链接器依赖

### 修复
- AUTOSAR 项目中深度 > 3 的 C/C++ 文件未被 SAST 扫描——`has_c_cpp_files` 最大深度从 3 提升至 6
- 相同 `(name, ecosystem)` 的 AUTOSAR 依赖有版本化和 `unspecified` 条目时现已去重——版本化条目优先
- 存在同名 `autosar` 生态系统条目时 `system` 生态系统条目被抑制

## [1.0.16] - 2026-05-10

### 新增
- **回退模式**：当未找到清单文件派生的组件目录但扫描根目录下存在 C/C++ 源文件时，自动插入合成的 `(project_name, "C/C++") → scan_root` 条目——可扫描无清单仓库（如 NIST Juliet 测试套件）
- **`has_c_cpp_files` 辅助函数**：浅层 `WalkDir`（最大深度 3）检查，复用 `is_c_cpp_source` 谓词；供回退模式使用以避免对非 C 仓库的误判
- **`resolve_component_dir` 辅助函数**：通过三种策略（精确名称、`lib` 前缀、大小写不敏感扫描）将清单声明的 C/C++ 依赖项映射到供应商源码子目录；对无匹配供应商子目录的依赖项返回 `None`，防止来自外部/系统依赖的误判发现项

### 内部
- `build-all.sh` 增加 `--internal` 标志：同时构建公开版本和内部版本，内部版本二进制文件名添加 `-internal` 后缀

## [1.0.15] - 2026-05-09

### 新增
- **AUTOSAR 检测**：预扫描 `detect_autosar()` 在扫描前运行；通过 `.arxml` 文件（DET-01）、BSW/MCAL/RTE/AUTOSAR/SWC 目录名（DET-02）和构建文件中的 `AUTOSAR_VERSION`/`AR_VERSION` 令牌（DET-03）检测项目
- **AUTOSAR 分类**：`classify_autosar_components()` 将依赖名称与内置 BSW 模块配置比对；将匹配组件升级为 `ecosystem="autosar"`，附带 `AutosarMetadata`（module_name、layer、platform）
- **AUTOSAR 输出 — CycloneDX**：AUTOSAR 组件输出 `autosar:layer` 和 `autosar:platform` 属性（如 `"BSW-Memory"`、`"Classic"`）
- **AUTOSAR 输出 — SPDX**：AUTOSAR 组件以 `ExternalRef OTHER` 条目输出 `autosar:layer` 和 `autosar:platform`
- **供应商配置**：`--supplier-config <path>` 接受 YAML 文件，将 AUTOSAR 组件名映射到供应商字符串；已映射组件在 CycloneDX 属性和 SPDX ExternalRef 中输出 `autosar:supplier`；未映射组件输出 `NOASSERTION`
- **BSW 配置覆盖**：`--bsw-config <path>` 用自定义 YAML 文件覆盖内置 AUTOSAR BSW 模块配置

### 变更
- `--output` 现在对所有单一格式（`spdx-json`、`spdx-tag-value`、`cyclonedx-json`、`console`）生效，而不仅限于 `--format all`；单一格式省略 `--output` 时输出到标准输出（保持原有默认行为）

### 内部

## [1.0.14] - 2026-04-24

### 修复
- 扫描器遇到断开的符号链接不再中止——发出 `Warning: skipping` 并继续（覆盖全部 5 处 WalkDir 遍历点：scanner、回退导入扫描、C 解析器）
- `$(OPENSSL_VERSION)` 等 Makefile 变量引用不再泄漏到 SPDX `versionInfo`——在解析器层及每个 SPDX/CycloneDX 版本输出点进行过滤，改为输出 `NOASSERTION`
- C/C++ 库许可证现从 `.pc` 的 `License:` 字段和已知库查找表（24 个常见系统库）解析，不再全部为 `NOASSERTION`

### 变更
- Linux 发布二进制文件现通过 musl（`x86_64-unknown-linux-musl`）静态链接——消除 glibc 版本依赖；可在 Ubuntu 22.04+、24.04、Alpine 及任何 x86_64 Linux 上运行
- 为 `build-all.sh` 添加 musl 交叉链接器工具链守卫

### 内部
- 将 `warn_on_walkdir_err` 辅助函数提取到 `src/util/mod.rs`,合并了 5 处相同的 `filter_map` 闭包
- 使用 Unix 专属 API 的跨平台测试现由 `#[cfg(unix)]` 守卫

## [1.0.13] - 2026-04-14

### 新增
- **AI 模型**：多模态子模型分解 — 复合模型（Gemma-4、LLaVA、Qwen-VL）分解为文本、视觉和音频子模型组件
- **AI 模型**：新增 `SubModelInfo` 结构体，捕获每个子模型的架构信息：model_type、layers、hidden_size、heads、dtype、vocab_size、上下文窗口及模态特定字段（patch_size、conv_kernel_size 等）
- **AI 模型**：守卫条件 — 仅对真正的多模态模型（同时存在 text_config + vision/audio_config）生成子模型
- **CycloneDX**：在父 AI 模型组件内嵌套 `components` 数组，每个子模型带有 `radeis:ai:sub_model:*` 属性
- **SPDX**：子包与父模型之间通过 `CONTAINS` 包含关系连接
- **控制台**：子模型摘要表，显示模态、模型类型、层数、隐藏层大小、注意力头数、dtype 及模态特定扩展信息
- **测试**：新增 5 项测试（4 项 Safetensors + 1 项 GGUF），涵盖多模态提取、纯文本守卫、仅视觉+文本及 GGUF 增强

## [1.0.12] - 2026-04-14

### 新增
- **AI 模型**：从 HuggingFace 伴随文件为 Safetensors 和 GGUF 仓库提取丰富元数据
- **AI 模型**：解析 `generation_config.json` — temperature、top_k、top_p 推理默认值
- **AI 模型**：解析 `tokenizer_config.json` — processor_class、model_max_length（含天文数字值截断）
- **AI 模型**：解析 `preprocessor_config.json` — 图像、音频和视频处理器类型及参数
- **AI 模型**：解析 `README.md` YAML frontmatter — base_model（字符串或列表）、license、pipeline_tag、quantized_by、prompt_template、tags、languages、datasets
- **AI 模型**：扩展 `config.json` 提取 — model_type、text_config（hidden_layers、hidden_size、attention_heads、max_position_embeddings）、多模态检测（vision_config、audio_config）
- **AI 模型**：Dtype 回退链 — `torch_dtype` > `dtype` > `text_config.dtype`
- **AI 模型**：通过 `adapter_config.json` 检测 LoRA/QLoRA 适配器
- **AI 模型**：GGUF 伴随文件增强 — 二进制元数据始终优先，伴随文件填补空白
- **AI 模型**：来自二进制和 README 来源的 tags、languages 和 datasets 去重联合合并
- **AI 模型**：所有伴随文件读取设置 1 MB 安全上限
- **AI 模型**：README.md 文件名大小写不敏感匹配及 CRLF 换行符支持
- **CycloneDX**：约 25 个新的 `radeis:ai:*` 属性，涵盖丰富的 AI 模型元数据
- **SPDX**：sourceInfo 扩展了 model_type、上下文窗口和模态摘要
- **控制台**：AI Model Details 表格扩展了架构、多模态、生成和溯源部分
- **测试**：`safetensors_tests.rs` 新增 8 项测试，`gguf_tests.rs` 新增 6 项测试

## [1.0.11] - 2026-04-13

### 新增
- **AI 模型**：Safetensors AI 模型 SBOM 解析 — 支持 `.safetensors`、`model.safetensors.index.json` 和 `config.json`
- **AI 模型**：目录级扫描 — 无论分片数量多少，每个模型仅生成一条 Dependency 记录
- **AI 模型**：HuggingFace Safetensors 模型的 `pkg:huggingface` PURL
- **AI 模型**：Safetensors 模型的 CycloneDX `machine-learning-model` 组件类型及 `modelCard`
- **AI 模型**：分片去重 — 多分片模型（如 `model-00001-of-00002.safetensors`）合并为单条 SBOM 记录
- **AI 模型**：新增 `AIModelMetadata` 字段：`safetensors_format`、`total_size_bytes`、`shard_count`、`torch_dtype`、`transformers_version`、`vocab_size`
- **测试**：`tests/parser_tests/safetensors_tests.rs` 新增 12 项测试

## [1.0.10] - 2026-04-13

### 新增
- **Java/Gradle**：完整解析 `build.gradle`（Groovy DSL）和 `build.gradle.kts`（Kotlin DSL）的依赖关系 — 此前仅支持检测
- **Java/Gradle**：字符串记法（`'group:artifact:version'`）、映射记法（`group: 'g', name: 'a', version: 'v'`）、平台/BOM 支持
- **Java/Gradle**：作用域分类 — `testImplementation` → 测试、`compileOnly` → 已提供、`annotationProcessor`/`kapt`/`ksp` → 构建
- **Java/Gradle**：Android 项目支持（`androidTestImplementation`、`androidTestCompile`）

### 变更
- **Java**：Gradle 状态在生态系统表中从"仅检测"升级为"生产就绪"

## [1.0.9] - 2026-04-10

### 新增
- **AI 模型**：GGUF 二进制解析器，支持元数据提取（架构、量化类型、张量信息、上下文长度）
- **AI 模型**：CycloneDX `machine-learning-model` 组件类型，附带 `modelCard`（训练参数、数据集）
- **AI 模型**：SPDX `pkg:huggingface` PURL，用于 AI 模型依赖标识
- **AI 模型**：完整性验证 — 张量参数交叉验证及 SHA-256 哈希，用于模型文件真实性校验
- **AI 模型**：常见 AI 模型许可证的 SPDX 格式归一化
- **CLI**：`--scan-ai-models` 标志，用于启用 GGUF 模型扫描（默认值：true）

### 变更
- **CLI**：将 5 个 C/C++ 构建系统标志（`--scan-cmake`、`--scan-pkgconfig`、`--scan-autotools`、`--scan-makefiles`、`--scan-mk-files`）合并为单一的 `--scan-c-build-systems` 标志
- **CLI**：将 `--meson-parse-subprojects` 合并至 `--scan-meson`（启用 Meson 扫描时始终生效）
- **核心**：`scan_directory()` 从 20 个参数精简至 13 个

### 移除
- **CLI**：`--resolve-system-deps` 标志（无效代码 — 从未连接到任何实现）
- **CLI**：`--meson-parse-subprojects` 标志（启用 `--scan-meson` 时子项目解析现已始终启用）

## [1.0.7] - 2026-03-12

### 变更 - 漏洞扫描改为可选启用

- **默认输出更简洁** — 禁用扫描时，漏洞摘要行、风险评估章节及各软件包详情均不再显示

## [1.0.6] - 2026-03-04

### 新增 - 生产环境 SBOM 过滤与自动化依赖范围分类

**第 1-3 阶段完成：核心分类系统**
- **自动范围分类**，适用于所有依赖（v1.0.6 第 1-3 阶段）
  - 6 种范围类型：运行时、构建、测试、开发、可选、已提供
  - 多策略分类：生态系统、名称模式、目录分析
  - 置信度评分（0.0-1.0），附详细推理说明
  - 支持 10 种以上生态系统（npm、PIP、cargo、SYSTEM、BUILD-CONFIG 等）

**第 4 阶段完成：全面测试与验证**
- **42 项新集成测试**，验证范围过滤与分类准确性
  - 14 项范围过滤集成测试
  - 11 项端到端生产模式测试
  - 17 项真实场景分类验证测试
- **全部 609 项测试通过**（203 个库测试 + 203 个二进制测试 + 200 个集成测试 + 3 个文档测试）
- **已使用真实依赖进行验证**：
  - 常见 C/C++ 运行时库（zlib、curl、openssl、protobuf）
  - 构建工具（cmake、gcc、ninja、meson）
  - 测试框架（pytest、jest、gtest、unity）
  - 开发工具（pylint、black、eslint、prettier）
  - Web 框架（django、flask、express、react）

**第 5 阶段完成：文档**
- **v1.0.6 功能的完整文档**
  - 更新 [README.md](README.md)，添加生产模式示例
  - 新增 [SCOPE_CLASSIFICATION.md](docs/SCOPE_CLASSIFICATION.md) - 完整的范围过滤指南（300 行以上）
  - 更新 [CLI.md](docs/CLI.md)，添加范围过滤选项
  - CHANGELOG.md 添加第 5 阶段完成说明
- **文档覆盖内容**：
  - 快速上手示例
  - 依赖范围说明（6 种类型）
  - 分类方法（4 种策略）
  - 过滤选项及示例
  - 置信度评分解读
  - 故障排查指南
  - 生产环境、安全合规的最佳实践
  - 真实场景验证指标

**范围过滤 CLI：**
- **`--scope-filter <SCOPE>`** - 按范围过滤依赖
  - 支持多值：runtime、build、test、development、optional、provided
  - 示例：`--scope-filter runtime --scope-filter optional`
- **`--production`** - 生产模式（仅运行时 + 可选）
  - 显著缩减 SBOM 体积（例如，典型项目从 67 个软件包缩减至 11 个）
  - 等价于 `--scope-filter runtime --scope-filter optional`

**输出与报告增强：**
- **范围统计**，显示于控制台及 Markdown 报告
  - 各范围计数及百分比
  - 平均分类置信度
  - 依赖总数
- **依赖创建的构建者模式**
  - `Dependency::new().with_scope(scope, confidence, reason)`
  - 更简洁的测试代码与更好的 API 使用体验

**分类功能：**
- **生态系统感知分类**：
  - SYSTEM 库 → 运行时（置信度 0.8+）
  - BUILD-CONFIG → 构建（待链接分析优化）
  - GIT-SUBMODULE → 已提供
  - MESON-WRAP/SUBPROJECT → 构建
  - PIP/npm/cargo → 视上下文而定
- **基于名称的启发式规则**：
  - 已知工具的精确匹配（置信度 1.0）
  - 大小写不敏感匹配
  - 基于模式的分类
- **详尽推理说明**：所有分类均附详细解释

**按类别的测试覆盖：**
- **范围过滤**（14 项测试）：
  - 默认行为（无过滤）
  - 生产模式过滤
  - 自定义单范围/多范围过滤
  - 边界情况（无效过滤器、空结果）
- **端到端工作流**（11 项测试）：
  - 完整分类流水线
  - 生产 SBOM 生成
  - CycloneDX 输出集成
  - SBOM 体积缩减验证
  - 生态系统多样性验证
- **真实场景分类**（17 项测试）：
  - 常用库检测准确率
  - 构建工具识别
  - 测试框架识别
  - 开发工具分类
  - 置信度分布
  - 分类推理验证

### 变更
- **Dependency 结构体**新增范围字段
  - `scope: DependencyScope` - 分类结果
  - `scope_confidence: f32` - 置信度评分（0.0-1.0）
  - `scope_reason: String` - 人类可读的说明
- **主流水线**现包含自动分类（步骤 3.5/5）
- **SBOM 结构体**包含 `scope_statistics: Option<ScopeStatistics>`
- **控制台输出**在摘要中显示范围分布

### 第 4 阶段验证结果
- **测试通过率**：100%（609/609 项测试通过）
- **分类准确率**：
  - 构建工具：100% 准确（cmake、gcc、ninja、meson）
  - 测试框架：100% 准确（pytest、jest、junit、mocha）
  - 开发工具：100% 准确（pylint、black、eslint、prettier）
  - 运行时库：SYSTEM 生态系统高准确率
  - BUILD-CONFIG：默认归为构建（待链接分析优化）
- **置信度分布**：
  - 精确名称匹配：0.95-1.0（pytest、cmake 等）
  - 基于生态系统：0.7-0.9（SYSTEM、GIT-SUBMODULE）
  - 基于启发式：0.5-0.8（回退情况）
- **生态系统覆盖**：已验证 10 种以上生态系统
- **生产 SBOM 体积缩减**：典型情况缩减 50-80%（如从 67 个缩减至 11 个软件包）

## [1.0.5] - 2026-03-02

### 新增 - GitHub Actions 与 CI/CD 基础设施

**多平台构建系统：**
- **GitHub Actions 工作流**，用于自动化跨平台构建与发布
  - macOS（ARM64 与 Intel x86_64），通过 osxcross 交叉编译
  - Linux（x86_64 glibc）
  - Windows（x86_64），通过 MinGW 交叉编译
  - 带缓存的并行构建，加速 CI/CD 流水线
  - 自动化发布，包含二进制资产与校验和

**发布自动化：**
- **自动提取发布说明**，来源为 CHANGELOG.md
- **版本一致性验证**，检查 git 标签与 Cargo.toml 是否匹配
- **SHA256 校验和**生成，适用于所有发布二进制文件
- **PDF 生成**，使用 xnexus-md2pdf-tool 生成二进制发行指南
- 带封面页与专业排版的 VicOne 风格 README.pdf

**构建改进：**
- 自托管运行器支持，集成企业级 GitHub
- 用于私有子模块访问的 GIT_TOKEN 认证
- 产物保留（90 天），用于构建可追溯性
- 平台专属构建过滤（仅构建所需平台）
- 可配置的正式版/预发布版标志

### 变更 - 代码质量与死代码清理

**API 简化（约删除 155 行）：**
- **移除所有格式函数的 `_with_mode` 后缀**：
  - `print_spdx_json_with_mode` → `print_spdx_json`
  - `print_spdx_tag_value_with_mode` → `print_spdx_tag_value`
  - `save_spdx_json_with_mode` → `save_spdx_json`
  - `save_spdx_tag_value_with_mode` → `save_spdx_tag_value`
  - `convert_to_spdx_with_mode` → `convert_to_spdx`
  - `convert_to_cyclonedx_with_mode` → `convert_to_cyclonedx`
  - `print_cyclonedx_json_with_mode` → `print_cyclonedx_json`
  - `save_cyclonedx_json_with_mode` → `save_cyclonedx_json`

**消除编译器警告：**
- 移除格式模块中未使用的包装函数
- 清理所有解析器模块中未使用的导入
- 移除解析器函数中的死代码
- 修复测试引用，使用新的简化 API

**文档：**
- 更新 BINARY_README.md，添加完整的 MIT 许可证文本（VicOne Inc. 版权）
- 为面向客户的文档移除内部发行章节
- 将 LICENSE 中的版权持有人从"William Chang"更新为"VicOne Inc."

### 迁移说明

**仅影响库调用方**（二进制用户不受影响）：

格式 API 已简化，函数名更清晰：

```rust
// 旧 API（v1.0.5 之前）
save_spdx_json_with_mode(sbom, path, &SbomMode::Complete, false)
convert_to_cyclonedx_with_mode(sbom, &SbomMode::Complete)

// 新 API（v1.0.5+）
save_spdx_json(sbom, path, &SbomMode::Complete, false)
convert_to_cyclonedx(sbom, &SbomMode::Complete)
```

### 基础设施

**发布资产：**
- macOS（ARM64/Intel）、Linux（x86_64 glibc）、Windows（x86_64）预构建二进制文件
- README.md（Markdown 二进制发行指南）
- README.pdf（VicOne 风格 PDF 指南）- 新增！
- checksums.txt（SHA256 验证）

所有二进制文件均通过 GitHub Actions 构建，并经过自动化测试与验证。

## [1.0.4] - 2026-02-25

### 新增 - Meson 与 Bazel 构建系统

**现代 C/C++ 构建系统支持：**
- **Meson 构建系统解析器** - 现代元构建系统支持
  - 解析 `meson.build` 文件中的 dependency() 和 subproject() 声明
  - 支持 wrap 文件（`*.wrap`）用于外部依赖管理
  - 处理版本约束与构建选项
  - 与 Conan 锁文件集成（已在 conanfile.lock 中检测到 meson 1.2.2）
  - 额外覆盖约 2.5% 的 C/C++ 项目

- **Bazel 构建系统解析器** - Google 构建系统支持
  - 解析 `BUILD`、`BUILD.bazel` 与 `WORKSPACE` 文件
  - 支持 http_archive、git_repository 与 maven_jar 规则
  - 从 URL 和 commit SHA 提取版本
  - 处理外部仓库引用（@repo//:target）
  - 额外覆盖约 2.5% 的 C/C++ 项目

**CLI 增强：**
- `--scan-meson` 标志，启用/禁用 Meson 扫描（默认：true）
- `--scan-bazel` 标志，启用/禁用 Bazel 扫描（默认：true）
- 所有 C/C++ 生态系统标志的完整帮助文本

### 新增 - 全面的比较报告

**竞品分析：**
- **6 份完整比较报告**，共 2,897 行
  - OpenStudio、UD Trucks 生产环境、VDL Bus 生产环境
  - ROS 2 Humble Desktop、scikit-learn、Python 测试夹具
- **2,561 个依赖**横跨所有项目
- **比竞品多检测 2.1%-58.8% 的软件包**
- **4 项竞品商业工具所不具备的独特能力**：
  - Autotools/pkg-config 支持（遗留 C 项目）
  - ROS 2 package.xml 解析
  - Git 子模块递归扫描
  - CMake FetchContent/ExternalProject

**成本节省分析：**
- 相比 BlackDuck 授权费用节省 **$220K-$1.65M**
- 单次扫描费用：$0（radeis_sc2sbom）vs $50-$200（SaaS 竞品）
- 比较文档总计：7 份报告 + 索引

### 变更

**覆盖率影响：**
- v1.0.4 之前：约 90% 的 C/C++ 项目
- v1.0.4 之后：**约 95% 的 C/C++ 项目**

**Bug 修复：**
- 修复 Bazel 解析器在 BUILD 文件表达式中的括号匹配问题
- 将系统软件包 purl 类型更正为 `pkg:generic/` 格式
- 更新扫描器测试以适配新的签名变更

**文档：**
- 更新 README.md，添加 Meson 和 Bazel 生态系统支持
- 完善 BENCHMARKS.md，添加详细的比较方法论
- 在 WHATS_NEW.md 中添加 v1.0.4 发布说明

### 基础设施

**测试：**
- 105 项单元测试通过（新增 8 项 Meson/Bazel 测试）
- 使用真实 BUILD 和 meson.build 文件进行集成测试
- 向后兼容：与 v1.0.0-1.0.3 100% 兼容

## [1.0.3] - 2026-02-24

### 新增 - C 遗留构建系统支持

**传统 C/C++ 构建系统解析器：**
- **pkg-config 解析器** - 系统库依赖检测
  - 解析 `.pc` 文件（pkg-config 元数据文件）
  - 从 configure.ac 提取 `PKG_CHECK_MODULES` 声明
  - 版本与依赖链解析
  - 处理 Requires/Requires.private 字段
  - 系统库依赖约 80% 的覆盖率

- **Autotools 解析器** - GNU 构建系统支持
  - 解析 `configure.ac` 文件
    - `AC_CHECK_LIB(library, function)` 声明
    - `AC_SEARCH_LIBS(function, [libs...])` 声明
    - `PKG_CHECK_MODULES(PREFIX, packages)` 宏
  - 解析 `Makefile.am` 文件
    - `LDADD` 与 `LIBADD` 链接器标志提取
    - `-l` 标志解析用于库依赖
  - 纯 C 项目约 60% 的覆盖率

- **Makefile 启发式解析器** - 纯 Makefile 支持
  - 从 LDFLAGS/LIBS 中基于模式提取 `-l` 标志
  - 检测带库名的 `pkg-config --libs` 调用
  - 尽力解析，无需完整 Make 求值
  - 遗留 C++ 项目约 40% 的覆盖率

**CLI 增强：**
- `--scan-pkgconfig` 标志，启用/禁用 pkg-config 扫描（默认：true）
- `--scan-autotools` 标志，启用/禁用 Autotools 扫描（默认：true）
- `--scan-makefiles` 标志，启用/禁用 Makefile 扫描（默认：true）
- `--resolve-system-deps` 标志，尝试解析系统库版本（默认：true）

### 变更

**覆盖率影响：**
- 结合 v1.0.0-1.0.2：**约 90% 的所有 C/C++ 项目**
- 成功扫描遗留项目（curl、nginx、openssl 模式）
- 填补无现代构建系统项目的缺口

**SPDX purl 支持：**
- 新增 `pkg:generic/{name}@{version}?type=pkg-config` 格式
- 新增 `pkg:generic/{name}@{version}?type=autotools` 格式
- 新增 `pkg:generic/{name}@{version}?type=makefile` 格式

**文档：**
- 更新 README.md，添加 C 遗留构建系统支持
- 在 WHATS_NEW.md 中添加 v1.0.3 详细发布说明
- 记录启发式解析的限制与最佳实践

### 基础设施

**测试：**
- 所有 5 个 C 解析器的单元测试，使用 tempfile 夹具
- 使用真实 configure.ac、Makefile.am、Makefile 样本进行集成测试
- 测试夹具包含 openssl.pc、curl 风格的 configure.ac 模式

## [1.0.2] - 2026-02-24

### 新增 - Conan 包管理器支持

**Conan C++ 包管理器：**
- **Conan 锁文件解析器**（`conan.lock`）
  - 解析 Conan v1 锁文件格式（JSON）
  - 提取带完整版本和修订信息的软件包引用
  - 支持直接依赖与传递依赖图
  - 处理远程仓库元数据
- **Conan 清单解析器**（`conanfile.txt`、`conanfile.py`）
  - 锁文件不可用时的回退方案
  - 版本约束解析（>=、~、==等）
  - 选项与生成器检测

**SPDX purl 支持：**
- 新增 `pkg:conan/{name}@{version}` 格式
- 可用时包含修订与远程元数据

**CLI 增强：**
- 自动检测 `conan.lock`、`conanfile.txt`、`conanfile.py`
- 集成至现有 C/C++ 扫描工作流

### 变更

**扫描器改进：**
- 通过深度验证优化子模块扫描
- 提升 CMake 解析鲁棒性（*.cmake 模块文件）
- 修复递归扫描中冗余的深度检查
- 改善对格式错误 Conan 文件的错误处理

**文档：**
- 重组 README.md，改善结构
- 在 `docs/` 目录创建完整文档
- 更新 WHATS_NEW.md，添加 v1.0.2 发布说明
- 在支持的生态系统表中添加 Conan

### 基础设施

**测试：**
- 使用真实锁文件样本进行 Conan 解析器单元测试
- conanfile.txt 和 conanfile.py 的集成测试
- 全部测试通过（共 124 项）

**CLI：**
- 新增 `--output` 标志，用于自定义 SBOM 输出目录
- 改善帮助文本与示例

## [1.0.1] - 2026-02-23

### 新增 - CMake 支持与递归子模块扫描

**CMake 依赖解析器：**
- **FetchContent 解析器** - 现代 CMake 外部依赖
  - 从 CMakeLists.txt 解析 `FetchContent_Declare()` 块
  - 支持 GIT_REPOSITORY、GIT_TAG、URL、URL_HASH
  - 从 URL_HASH 提取 SHA256 校验和，用于供应链安全
  - 使用 Git URL 解析器生成正确的 github/gitlab/bitbucket purl
  - 从 GIT_TAG 或 URL 路径提取版本
- **ExternalProject 解析器** - 遗留 CMake 外部项目支持
  - 解析 `ExternalProject_Add()` 块
  - 处理与 FetchContent 相同的 Git 与 URL 模式
  - 静态解析（无需执行 CMake）

**递归子模块扫描：**
- 递归扫描 Git 子模块内的依赖
  - package.json、Cargo.toml、CMakeLists.txt 及所有支持的清单文件
  - 深度限制，防止无限递归
  - 可配置最大深度（默认：3）
- 扫描器模块中新增 `scan_submodule_recursively()` 函数

**CLI 增强：**
- `--scan-cmake` 标志，启用/禁用 CMake 依赖扫描（默认：true）

### 变更

**扫描器签名：**
- 更新 `scan_directory()`，新增 `scan_cmake` 参数
- 所有调用点已更新以传递新参数
- 测试已更新以适配新扫描器签名

**SPDX purl 支持：**
- 新增 `pkg:cmake/{name}@{version}` 格式
- 非 Git 来源回退至 `pkg:generic/{name}@{version}`

**文档：**
- 更新 README.md，添加 CMake 支持与新 CLI 标志
- 在 WHATS_NEW.md 中添加 v1.0.1 详细发布说明
- 记录 CMake 变量处理限制（`${VAR}` 将被跳过）

### 基础设施

**测试：**
- 创建 `tests/parser_tests/cmake_tests.rs`，包含 8 项全面测试
  - FetchContent 解析（Git 和 URL 来源）
  - ExternalProject 解析
  - CMake 变量处理（对无法解析的变量发出警告并跳过）
  - 校验和提取验证
- 在 `tests/fixtures/cmake/` 中创建测试夹具
  - CMakeLists_fetchcontent.txt
  - CMakeLists_externalproject.txt
  - CMakeLists_with_variables.txt
- 全部 124 项测试通过（原有 116 项 + 8 项新 CMake 测试）

## [1.0.0] - 2026-02-20

### 新增 - C++ 生态系统支持

**首个 C++ 生态系统支持：**
此主要版本（1.0.0）增加了完整的 C/C++ 项目 SBOM 生成能力。

**vcpkg 包管理器：**
- **vcpkg 清单解析器**（`vcpkg.json`）
  - 所有版本约束格式：`version>=`、`version>`、`version=`、`version-semver`、`version-date`
  - 用于版本固定的覆盖章节
  - 特性元数据存储于 source_file 字段
  - 生成 `pkg:vcpkg/{name}@{version}` purl 格式
- 已集成至扫描器，支持自动检测 vcpkg.json

**Git 子模块检测：**
- **Git 子模块解析器**（`.gitmodules`）
  - 解析 INI 格式的子模块定义
  - 通过 `git ls-tree HEAD` 解析 commit SHA
  - 支持 HTTPS 与 SSH URL 格式
  - 多主机支持：GitHub、GitLab、Bitbucket、自托管 Git 服务器
  - 根据主机类型生成相应 purl 格式：
    - GitHub：`pkg:github/owner/repo@sha`
    - GitLab：`pkg:gitlab/owner/repo@sha`
    - Bitbucket：`pkg:bitbucket/owner/repo@sha`
    - 自托管：`pkg:generic/repo@sha`

**CLI 增强：**
- `--scan-submodules` 标志，启用/禁用子模块扫描（默认：true）
- `--submodule-depth` 标志，设置子模块的最大递归深度（默认：3）

### 新模块

**解析器模块：**
- `src/parsers/cpp/mod.rs` - C++ 解析器模块入口点
- `src/parsers/cpp/vcpkg.rs` - vcpkg 清单解析器（458 行）
- `src/parsers/git/mod.rs` - Git 解析器模块入口点
- `src/parsers/git/submodules.rs` - .gitmodules 解析器（292 行）
- `src/parsers/git/commit_resolver.rs` - Git commit SHA 解析器（140 行）
- `src/parsers/git/url_parser.rs` - 支持多主机的 Git URL 解析器（299 行）

**新增代码总计：** 1,511 行（19 个文件变更）

### 变更

**扫描器集成：**
- 在 `scan_directory()` 中添加 vcpkg.json 和 .gitmodules 检测
- 集成 commit SHA 解析，用于精确的子模块版本
- 对无法解析的 Git 引用发出警告

**SPDX 格式：**
- 为 vcpkg 软件包添加 purl 生成
- 为 Git 子模块添加带主机特定格式的 purl 生成

**文档：**
- 更新 README.md，添加 C++ 生态系统支持表
- 在 WHATS_NEW.md 中添加 v1.0.0 详细发布说明
- 记录 vcpkg 版本约束格式
- 记录 Git URL 解析与多主机支持

### 基础设施

**测试：**
- 带版本约束验证的 vcpkg 解析器测试
- 带 URL 解析的 Git 子模块解析器测试
- C++ 项目集成测试
- 全部测试通过

**Bug 修复（1.0.0 预发布完善）：**
- 修复 Python 版本运算符剥离（>=、==、~=）
- 为 Python 添加传递依赖解析
- 修复 Python 解析器中 `__future__` 误报
- 改善 Git 操作中的错误处理

## [0.9.3] - 2026-02-10

### 新增 - Pipfile/Pipfile.lock 与 pyproject.toml 解析器支持

**Pipfile/Pipfile.lock 支持（第 1 阶段）：**
- **Pipfile.lock 解析器** - 为 Pipenv 项目提供完整的锁文件支持
  - 使用 serde_json 反序列化解析 JSON 格式
  - 从 `default`（生产）和 `develop`（开发）章节提取所有软件包
  - **SHA256 校验和提取**，来自 hashes 数组，用于供应链安全
  - 通过 `index` 字段的存在来检测直接依赖
  - 使用 rayon 并行批量从 PyPI 获取元数据
  - **提升 487%**：从 8 个软件包（v0.9.1）提升至 47 个（v0.9.3）
  - 基于 Pipenv 项目与 Black Duck **100% 对等**

- **Pipfile 清单解析器** - 锁文件不可用时的回退方案
  - 使用现有 toml crate 解析 TOML 格式
  - 处理版本规范：`*`、`==`、`>=`、`~=`、复杂约束
  - 区分 `[packages]`（生产）和 `[dev-packages]`（开发）

**pyproject.toml 支持（第 2 阶段）：**
- **多格式 pyproject.toml 解析器** - 现代 Python 打包标准（PEP 517/518）
  - **PEP 621 格式** - `[project]` 章节，含 `dependencies` 和 `optional-dependencies`
  - **Poetry 格式** - `[tool.poetry]` 章节，含 `dependencies` 和 `dev-dependencies`
  - **Poetry 1.2+ 组** - `[tool.poetry.group.*.dependencies]` 格式
  - **PDM 格式** - `[tool.pdm]` 章节，含 `dependencies` 和 `dev-dependencies`
  - 使用正则表达式解析复杂的依赖规范
  - 从可选依赖组（dev、test、tests、testing）检测开发依赖
  - 处理 Poetry 版本约束：插入符（`^`）、波浪符（`~`）、比较运算符
  - 自动过滤 Python 版本约束

**poetry.lock 校验和提取（第 3 阶段）：**
- **从 `[[package.files]]` 章节提取 SHA256 校验和**
  - 从 files 数组中提取第一个文件哈希
  - 格式：`sha256:abc123...` → `abc123...`
  - 增强 Poetry 项目的供应链安全
  - 零额外网络开销

### 变更
- assessment-service 仓库的 Python 软件包检测从 8 个提升至 47 个
- 消除 Pipenv 项目所有 `@detected` 版本占位符
- 软件包名称使用规范 PyPI 格式（例如 `repoze.lru` 而非 `repoze-lru`）
- 从 poetry.lock 和 Pipfile.lock 提取 SHA256 校验和（存储于 Dependency 结构体，供未来 SPDX 输出集成使用）
- 集成加载动画："parsing Pipfile.lock..." 消息在扫描期间显示
- 进度指示器自动显示 Python 软件包计数

### 测试
- 已验证从 assessment-service Pipfile.lock 检测到 47 个 Python 软件包
- 100% 版本准确率（无 "@detected" 占位符残留）
- 与 Black Duck 的软件包名称完全匹配（47/47 个软件包）
- 已验证 Pipfile.lock 和 poetry.lock 的校验和提取

### 性能
- 使用 rayon 并行批量获取 PyPI 元数据（沿用现有模式）
- 使用 serde 反序列化进行单遍 JSON/TOML 解析
- 相比 v0.9.1 无性能下降
- 与 v0.9.2 进度指示器无缝集成

### 技术细节
- **修改的文件：**
  - `src/parsers/python.rs` - 新增 3 个解析器函数（+219 行）
    - `parse_pipfile_lock_with_relationships()` - 带校验和的 Pipfile.lock
    - `parse_pipfile()` - Pipfile 清单
    - `parse_pyproject_toml()` - pyproject.toml 多格式
  - `src/parsers/mod.rs` - 导出新的解析器函数
  - `src/scanner/mod.rs` - 注册解析器并集成加载动画（+8 行）
  - `Cargo.toml` - 版本升级至 0.9.3

- **依赖：**
  - 无新增依赖（使用现有的 serde_json、toml、regex、rayon）

- **解析器优先级顺序：**
  1. Pipfile.lock（最高 - 带校验和的精确版本）
  2. poetry.lock（高 - 带校验和的精确版本）
  3. requirements.txt（中 - 版本规范）
  4. setup.py（中 - 版本规范）
  5. Pipfile（中 - 版本规范）
  6. pyproject.toml（中 - 版本规范）
  7. 导入扫描（最低 - 无版本）

### 竞争定位
- **与 Black Duck 对等**，Python 软件包检测（47/47 个软件包）
- **更优的软件包命名** - 使用规范 PyPI 名称
- **全面的 Python 支持** - Pipfile、Poetry、pip、setuptools、PDM、PEP 621
- **供应链安全** - 来自锁文件的 SHA256 校验和
- **现代标准** - 完整的 pyproject.toml 支持

### 迁移说明
- **库用户的破坏性变更**：`ScanContext.poetry_relationships` 更名为 `python_lockfile_relationships`（CLI 用法不受影响）
- 现有 Pipenv 项目自动受益于改进的检测
- Poetry 项目现在在 SBOM 输出中包含校验和
- pyproject.toml 项目现在可生成准确的 SBOM

## [0.9.1.1] - 2026-01-28（热修复）

### 修复 - Markdown 报告显示 Bug

**问题：**
- ROS 多软件包 Markdown 报告显示空的或不完整的代码块
- 示例："PIP（1 个软件包）"标题下代码块为空，"ROS（8 个软件包）"只显示 2 个软件包
- 由于软件包计数与显示的软件包不匹配，导致读者困惑

**根本原因：**
- 控制台报告生成使用 `render_dependency_list()`，该函数仅显示**生产**直接依赖
- 标题统计了所有软件包（包括开发依赖）
- 这导致开发依赖从显示中被过滤掉，但仍计入标题数量

**解决方案：**
- 修改 ROS 多软件包章节，包含所有直接依赖（生产 + 开发）
- 使用扩展过滤的 `render_tree_classic()` 显示所有直接依赖
- 保持带有正确分支字符（`├──`、`└──`）的树形结构
- 现在所有计入的软件包均在代码块中以树形可视化显示

**影响：**
- 修复 ROS 多软件包报告中的空代码块
- 显示所有直接依赖（生产和开发），使用正确的树形结构
- 保持标题与显示软件包之间计数一致
- 不影响常规（非 ROS）项目报告

**测试结果：**
- ros2run PIP 章节现在显示：`└── pytest @ unspecified [direct, dev]`
- ros2run ROS 章节现在显示全部 8 个软件包的树形结构（├──、└──）
- 修复前：空 PIP 块，不完整的 ROS 列表（仅显示 8 个中的 2 个）

**修改的文件：**
- `src/formats/console.rs`（第 1292-1324 行）

## [0.9.1] - 2026-01-28

### 新增 - ROS/rosdistro 版本解析与仓库 URL 丰富

**ROS 软件包版本解析：**
- **自动 ROS 发行版检测** - 与 rosdistro GitHub API 集成，解析 ROS 软件包版本
  - 从 ros/rosdistro 仓库获取 distribution.yaml
  - 支持 ROS 2 发行版：jazzy、iron、humble、galactic、foxy
  - 支持 ROS 1 发行版：noetic、melodic
- **手动覆盖的 CLI 标志** - `--ros-distro <distro>`，用于显式指定 ROS 发行版
  - 优先级顺序：CLI 标志 > ROS_DISTRO 环境变量 > 默认值（"jazzy"）
  - 示例：`--ros-distro humble`、`--ros-distro iron`
- **软件包名称变体解析** - 处理多种命名约定
  - 基础名称：`rclpy`
  - Python 前缀：`python3-rclpy`
  - 发行版前缀：`ros-jazzy-rclpy`
  - 下划线变体：`ament-index-python`、`python3-ament-index-python`
- **全局缓存** - 每次扫描会话每个发行版仅获取一次 rosdistro
  - 10 秒超时，优雅回退至"unspecified"
  - 使用 rayon 并行解析（与现有元数据丰富模式一致）

**仓库 URL 丰富：**
- **GitHub URL 提取** - 为 ROS 软件包填充 SPDX `downloadLocation` 字段
  - 从 rosdistro distribution.yaml 提取 `source.url`
  - 47 个软件包带有 GitHub 仓库 URL（ros2cli 项目基准）
  - 完整的源代码可追溯性，用于安全审计
  - 零性能开销（使用现有 rosdistro 获取）

### 变更
- ROS 依赖现在显示解析后的版本，而非"unspecified"
  - 修复前：`rclpy @ unspecified, downloadLocation: NOASSERTION`
  - 修复后：`rclpy @ 7.1.9, downloadLocation: https://github.com/ros2/rclpy.git`（jazzy）
- SPDX `downloadLocation` 字段已为 ROS 软件包填充（47 个软件包带 URL）
- 更新 `scan_directory()` 签名，接受可选的 `ros_distro` 参数
- 完善 `detect_ros_distribution()`，采用三层优先级系统
- 在 `RosPackageInfo` 结构体中新增 `repository_url` 字段
- 将 `lookup_package_version()` 重命名为 `lookup_package_info()`（返回版本 + URL）

### 测试
- 新增仓库 URL 丰富单元测试（`test_resolve_ros_dependency_versions_with_repository_url`）
- 新增 5 项 rosdistro 函数单元测试（版本解析、软件包变体、非 ROS 软件包）
- 新增 2 项 ros2cli 扫描集成测试，使用不同发行版
- 全部 97 项测试通过

### 性能
- 每个 ROS 发行版单次网络获取（会话期间缓存）
- 10 秒超时，支持优雅降级
- 不影响非 ROS 项目的性能
- 使用 rayon 并行解析
- 仓库 URL 提取无额外网络开销

### 技术细节
- 新增依赖：`serde_yaml` v0.9、`lazy_static` v1.4
- 修改的文件：`src/cli.rs`、`src/parsers/ros.rs`、`src/scanner/mod.rs`、`src/main.rs`、`src/formats/spdx.rs`
- 新函数：`fetch_rosdistro_database()`、`detect_ros_distribution()`、`lookup_package_info()`、`resolve_ros_dependency_versions()`
- 在 `RosPackageInfo` 结构体中新增 `repository_url: Option<String>` 字段
- 更新 SPDX 格式化器中的 `create_download_location()`，使用 `dependency.repository_url`

### 竞争定位
- **ROS 支持**：首个通过 rosdistro 实现自动化 ROS 软件包版本解析的 SBOM 工具
- **仓库 URL**：首个为 ROS 软件包填充 downloadLocation 的 SBOM 工具
- **对比 BlackDuck**：radeis 检测到 94 个唯一依赖（是 BlackDuck 4 个的 23.5 倍）
  - 15 个独立 ROS 软件包 vs 3 个仓库（粒度高 5 倍）
  - 47 个软件包带 GitHub URL vs BlackDuck 的 0 个
  - 62 个软件包带已解析版本 vs BlackDuck 的 3 个（多 21 倍）

### 基准测试结果（ros2cli 项目）
- **94 个唯一依赖**
- **15 个 ros2cli 仓库内的独立 ROS 软件包**
- **47 个软件包带 GitHub 仓库 URL**
- **62 个软件包带已解析版本**（覆盖率 66%）
- **223 条 SPDX 层级条目**（含关联关系）
- **检测到 5 个漏洞**并嵌入 SPDX 输出

## [0.9.0] - 2026-01-28

### 新增 - 校验和、自动化与多生态系统元数据

**软件包校验和：**
- **所有软件包的 SHA-1 校验和** - 支持完整性验证与可复现构建
  - 格式：40 字符小写十六进制 SHA-1 哈希
  - 添加至每个软件包的 SPDX `filesAnalyzed` 字段
  - 支持供应链安全与 SBOM 验证工作流

**自动化修复建议：**
- Markdown 输出中的机器可读修复建议
- 用于自动漏洞修复的结构化格式
- 与现有漏洞报告集成

**多生态系统元数据提取（网络模式）：**
- **所有生态系统的混合元数据提取** - 优先使用本地文件，回退至注册表 API
  - npm：package.json + npm 注册表 API（registry.npmjs.org）
  - Python：用于 poetry.lock 软件包的 PyPI API（pypi.org/pypi）
  - Cargo：用于 Cargo.lock 软件包的 crates.io API
  - PHP：用于 composer.json 软件包的 Packagist API（repo.packagist.org/p2）
  - Ruby：用于 Gemfile 软件包的 RubyGems API（rubygems.org/api/v2）
- **使用 rayon 并行批量获取**，性能提升 10-27 倍
  - npm：689 个软件包，从 10 分钟以上 → 22.6 秒（提速 27 倍）
  - Python：100-500 个软件包，从 10-20 分钟 → 30 秒（提速 20-40 倍）
  - Cargo：100-500 个软件包，从 7-15 分钟 → 25 秒（提速 17-36 倍）
  - PHP：10-200 个软件包，从 2-7 分钟 → 20 秒（提速 6-21 倍）
  - Ruby：5-50 个 gem，从 1-2 分钟 → 10 秒（提速 6-12 倍）
- **API 超时从 5 秒缩短至 3 秒**，加速失败处理

### 变更
- 文件大小从 955KB（v0.8.0）优化至 899KB（缩减 6%）
- 效率从 1.384 KB/包提升至 1.303 KB/包（改善 6%）
- 保留 v0.8.0 的全部 690 个软件包和 689 个 CPE 标识符

### 性能
- **软件包数量**：690 个（与 v0.8.0 一致）
- **文件大小**：899 KB（比 v0.8.0 的 955 KB 小 6%）
- **CPE 标识符**：689 个（与 v0.8.0 一致）
- **文件效率**：1.303 KB/包（优于 v0.8.0 的 1.384 KB/包）

### 竞争定位
- **独特功能**：唯一在 SPDX 输出中同时嵌入漏洞 + CPE 标识符 + SHA-1 校验和的工具
- **相比 v0.8.0 的改进**：文件大小缩减 6%，同时保留所有功能

## [0.8.0] - 2026-01-27

### 新增 - 丰富元数据与安全功能

**元数据提取（各生态系统覆盖率 95% 以上）：**
- **许可证信息提取** - 为 npm、Cargo、Python、ROS、PHP、Ruby 生态系统提取并规范化许可证标识符（符合 SPDX 规范）
  - 许可证覆盖率达 95% 以上（650+/690 个软件包，而 v0.7.0 为 0%）
  - 将 SPDX 的"NOASSERTION"替换为实际许可证标识符
- **供应商与来源方跟踪** - 供应链透明度所需的作者、维护者与组织元数据（覆盖率 90% 以上，620+/690 个软件包）
  - 映射至带"Person:"和"Organization:"前缀的 SPDX supplier/originator 字段
- **下载地址 URL** - 各生态系统软件包注册表 URL，用于验证与可复现构建
  - npm：`https://registry.npmjs.org/{package}/-/{file}.tgz`
  - PyPI：`https://pypi.org/project/{package}/{version}/`
  - Cargo：`https://crates.io/api/v1/crates/{package}/{version}/download`
  - 完整支持 Composer、RubyGems 和 Go 生态系统

**增强 SPDX 输出：**
- **基于 UUID 的 SPDX ID** - 比顺序 ID 唯一性更强，无命名空间冲突
  - 格式：`SPDXRef-Package-{sanitized-name}-{uuid}`
  - 合成"main"根软件包，确保文档结构一致
- **源文件跟踪** - 完整审计跟踪，显示哪个提取器和清单文件检测到每个软件包（覆盖率 98% 以上，685+/690 个软件包）
  - 模式："Identified by the {extractor_type} extractor from {absolute_path}"
- **CPE 标识符** - 通用平台枚举（CPE 2.3），用于安全漏洞关联
  - 格式：`cpe:2.3:a:vendor:product:version:*:*:*:*:*:*:*`
  - 生态系统特定的供应商提取（npm 范围包、Composer、Go 模块）

**测试基础设施：**
- **模块化测试结构** - 将所有 64 项测试从单体 main.rs 迁移至有组织的测试模块
  - 7 个类别（解析器、格式、扫描器、模型、错误、工具、集成）共 84 项测试
  - main.rs 从 2,233 行减少至 268 行（缩减 88%，删除 1,965 行）
  - 18 个独立测试文件，便于组织与维护
- **源文件跟踪测试** - 11 项新测试，覆盖所有解析器（npm、Cargo、Python、ROS、PHP、Ruby、Go）
- **多生态系统集成测试** - 2 项全面测试，验证不同解析器间的源文件跟踪
- **UUID 和 CPE 测试** - 7 项新测试，用于 SPDX ID 唯一性和 CPE 标识符生成

### 变更
- 在 `Dependency` 结构体中新增可选元数据字段（license、author、maintainers、repository_url、homepage_url、source_file）
- 更新 SPDX 软件包创建，填充 license、supplier、originator 和 sourceInfo 字段
- SPDX ID 生成从顺序式（`SPDXRef-Package-npm-1`）改为基于 UUID（`SPDXRef-Package-axios-{uuid}`）
- 关联关系结构从扁平式（v0.7.0 的 699 个 DESCRIBES）改为层级式（1 个 DESCRIBES + 689 个 CONTAINS）
- 所有解析器现在使用绝对路径跟踪源文件路径
- CycloneDX 格式现在包含许可证和供应商信息
- 创建 `src/lib.rs`，支持集成测试（将模块公开为库）
- 公开 SPDX 结构体以便测试（SPDXDocument、SPDXPackage、SPDXRelationship、SPDXExternalRef 字段）

### 改进
- 通过丰富元数据增强合规报告能力
- 更丰富的元数据，用于供应链安全与透明度
- 改善测试覆盖率与组织，提升长期可维护性
- 增强去重逻辑，正确处理 ImportScan 与 Manifest 优先级

### 修复
- **去重 Bug 修复** - ImportScan 条目现在当 LockFile/Manifest 版本存在时能正确被过滤
  - v0.7.0 错误保留了带占位符"detected"版本的 ImportScan 重复项
  - 移除 10 个重复软件包（9 个唯一软件包被计算了两次：axios、uuid、6 个 AWS SDK 客户端、serverless-sentry-lib、strftime）
  - 软件包计数从 699 更正为 690（689 个真实软件包 + 1 个合成"main"）
- 去重逻辑现在正确优先排序：LockFile > Manifest > ImportScan
- 测试结构现在已正确组织，用于 Rust 集成测试
- SPDX 外部引用现在以正确格式同时包含 PURL 和 CPE 标识符

### 性能
- **软件包数量**：690 个（Bug 修复：移除 v0.7.0 的 10 个重复 ImportScan 条目）
- **文件大小**：955 KB（回归：比 v0.7.0 的 454 KB 增加 110%）
- **CPE 标识符**：689 个（v0.8.0 新功能）
- **文件效率**：1.384 KB/包（回归：比 v0.7.0 的 0.649 KB/包 差 113%）
- 所有 84 项测试的执行时间在 90 秒以内

### 技术细节
- 修改 [src/models/dependency.rs](src/models/dependency.rs) - 新增可选元数据字段
- 修改 [src/formats/spdx.rs](src/formats/spdx.rs) - 基于 UUID 的 ID、CPE 生成、层级关联关系、元数据填充
- 修改 [src/formats/cyclonedx.rs](src/formats/cyclonedx.rs) - 许可证和供应商支持
- 修改 [src/parsers/mod.rs](src/parsers/mod.rs) - 带 ImportScan 优先级修复的增强去重
- 创建 [src/lib.rs](src/lib.rs) - 用于集成测试的库入口点
- 创建 [tests/all_tests.rs](tests/all_tests.rs) - 集成测试模块入口点
- 在 tests/ 目录下创建 18 个有组织结构的测试模块文件

### 竞争定位
- **元数据丰富度**：现已与企业级工具相当（95% 以上许可证、90% 以上供应商，对比 BlackDuck 99.9%）
- **独特功能**：唯一在 SPDX 输出中同时嵌入漏洞 + CPE 标识符的工具
- **多生态系统领先**：完整支持 npm、Cargo、Python、ROS、PHP、Ruby 生态系统的元数据
- **注意**：v0.8.0 修复了 ImportScan 去重 Bug（移除 10 个重复项）并添加了 CPE 元数据（增大了文件大小），两个问题均在 v0.9.0 中得到解决

## [0.7.0] - 2026-01-27

**⚠️ 已知问题**：本版本存在去重 Bug，错误保留了 10 个带"detected"版本的 ImportScan 重复软件包。详见 [PACKAGE_COUNT_ANALYSIS.md](scan_reports/PACKAGE_COUNT_ANALYSIS.md)。已在 v0.8.0 中修复。

### 新增
- **软件包检测** - 现可检测 **699 个软件包**（原为 563 个），但包含 10 个重复 ImportScan 条目（v0.8.0 更正为 690 个）
- **双 SBOM 模式**，适用于不同使用场景：
  - `--sbom-mode complete` - 所有软件包（699 个，含 10 个重复项，454KB），用于合规与资产清单
- 智能清单过滤 - 当精确的锁文件版本存在时，自动移除冗余的 package.json 版本范围
- `docs/` 文件夹中的完整文档：
  - [docs/sbom_modes_guide.md](docs/sbom_modes_guide.md) - 双 SBOM 模式完整指南，含 CI/CD 示例
  - [docs/WHATS_NEW.md](docs/WHATS_NEW.md) - v0.7.0 详细变更与迁移指南
  - [docs/plan/improvement_plan.md](docs/plan/improvement_plan.md) - 技术设计文档
  - [docs/plan/implementation_summary.md](docs/plan/implementation_summary.md) - 含指标的实现结果

### 变更
- **软件包检测提升 24.2%**（563 → 699 个，但含 10 个 ImportScan 重复项）
- 去重算法现在使用 `(name, version, ecosystem)` 元组代替 `(name, ecosystem)` 进行版本感知
- NPM 解析器使用 HashSet 防止对同一 `package@version` 组合的重复处理
- README.md 全面修订，更加简洁清晰，聚焦于关键改进
- 更新所有 SPDX 和 CycloneDX 生成器，支持基于模式的过滤

### 修复
- 同一软件包的多个版本现在能正确保留（例如 `@aws-sdk/client-sso@3.632.0` 和 `@aws-sdk/client-sso@3.848.0` 均保留）
- 当锁文件版本存在时，清单版本（如版本范围 `^3.215.0`）自动过滤
- 约 126 个缺失的 AWS SDK 子软件包现在能正确在嵌套 node_modules 中检测到

### 性能
- 仅漏洞模式实现文件大小缩减 98%（11 KB vs 454 KB）
- ⚠️ **699 个软件包含 10 个 ImportScan 重复项**（v0.8.0 更正为 690 个）

### 技术细节
- 修改 [src/parsers/mod.rs](src/parsers/mod.rs) - 带清单过滤的版本感知去重
- 修改 [src/parsers/npm.rs](src/parsers/npm.rs) - 基于 HashSet 的重复预防
- 修改 [src/cli.rs](src/cli.rs) - 新增 SbomMode 枚举
- 修改 [src/formats/spdx.rs](src/formats/spdx.rs) - SPDX 输出的基于模式的过滤
- 修改 [src/formats/cyclonedx.rs](src/formats/cyclonedx.rs) - CycloneDX 输出的基于模式的过滤
- 修改 [src/main.rs](src/main.rs) - 向所有格式生成器传递模式参数

### 竞争定位
- 双 SBOM 模式为合规和安全工作流提供灵活性
- **已知问题**：去重 Bug 允许带"detected"版本的 ImportScan 条目在 LockFile 版本存在时存活（v0.8.0 已修复）

## [0.6.0] - 2026-01-26

### 新增
- 从锁文件为 npm、Cargo 和 Poetry 生成真实的层级依赖树
  - 第 1 阶段：npm（package-lock.json）- 完整的父子关联关系
  - 第 2 阶段：Cargo（Cargo.lock）- 解析 Rust 项目的 dependencies 数组
  - 第 3 阶段：Poetry（poetry.lock）- 解析 Python 项目的 [package.dependencies] 表
- 使用基于图的分析精确标记 [direct] 与传递依赖
- 重组报告结构，区分软件包列表与附录章节
- 循环依赖检测与处理
- 完整依赖链跟踪，显示从根到漏洞软件包的所有路径
- 模块化架构重构 - 将 main.rs（6,561 行）拆分为 24 个文件中的 7 个模块

### 变更
- 报告结构现在先显示直接生产依赖，然后是独立列表，开发/传递依赖放入附录
- 附录放在漏洞章节之后，便于更好地组织
- Main.rs 从 6,561 行减少至 2,130 行（缩减 67.6%）

### 修复
- 基于实际父子关联关系修正 is_direct 标志，而非文件路径
- 控制台摘要计数现在使用依赖图中修正的标志
- VendorMode::Only Bug，该 Bug 阻止扫描 vendor 目录内的文件
- 合并重复的 create_package_url 函数
- 跨报告章节标准化直接依赖计数
- 漏洞树现在使用关联关系确保一致的 is_direct 标志

### 测试
- 全部 59 项综合单元测试通过
- 新增 Cargo 和 Poetry 关联关系解析测试
- 零编译警告

## [0.5.0] - 2026-01-23

### 破坏性变更
- 树形可视化现在默认启用（使用 --tree-style flat 恢复旧格式）

### 新增
- 三种模式的树形依赖可视化（flat、tree、compact）
- 以严重性优先组织的层级漏洞显示
- 一览式概览的摘要统计章节
- Emoji 严重性指示器（🔴 严重、🟠 高、🟡 中、🟢 低）
- 每个漏洞的依赖链显示
- 使用 --max-vulns-per-severity 的可折叠漏洞显示
- 摘要章节中的风险评估

### 变更
- 使用 Unicode 框绘字符增强控制台输出
- 使用一致分隔符改善视觉层级

## [0.4.0] - 2026-01-22

### 破坏性变更
- 默认启用漏洞检查
- 默认启用 vendor 目录扫描
- 默认启用导入回退扫描

### 新增
- 增强 Markdown 报告，含依赖来源跟踪（直接 vs 传递）

### 变更
- 改善 npm 软件包的传递依赖检测
- 控制台报告中默认输出详细漏洞信息

## [0.3.1] - 2026-01-21

### 新增
- 多平台构建系统（Windows + Linux）
- 交叉编译自动化
- 增强构建文档

## [0.3.0] - 2026-01-20

### 新增
- ROS/ROS2 多软件包支持
- 层级树形输出
- SPDX 关联关系（DESCRIBES、DEPENDS_ON）
- Python、JS/TS、Go 的导入扫描回退
- 51 项综合单元测试

## [0.2.0] - 2026-01-19

### 新增
- SPDX 2.3 支持（JSON + Tag-Value）
- 软件包 URL（purl）实现
- 多格式输出

## [0.1.0] - 2026-01-16

### 新增
- 初始版本
- 支持 8 种生态系统（npm、Cargo、pip、Go、RubyGems、Composer、Maven、ROS）
- 控制台输出
- 18 项单元测试

[0.9.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/VicOne-RD/radeis_sc2sbom/releases/tag/v0.1.0
