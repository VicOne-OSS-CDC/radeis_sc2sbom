<a href="README.md">English</a> | 简体中文 | <a href="docs/i18n/README.zh-Hant.md">繁體中文</a> | <a href="docs/i18n/README.ja.md">日本語</a>

<p align="center">
  <img src="docs/images/icon.png" alt="Sourcecode to SBOM" width="80">
</p>

# radeis_sc2sbom

**快速 SBOM 生成器，内置漏洞检测。** 多生态系统支持，具备独特能力：**唯一**支持 C/C++ Autotools、ROS 2、Git 子模块、CMake ExternalProject 和 AUTOSAR BSW 的工具。

## 借助 VicOne xZETA 充分发挥 SBOM 的价值

`radeis_sc2sbom` 生成的 SBOM 完全符合 **SPDX 2.3** 和 **CycloneDX 1.5** 标准，可直接导入 **[VicOne xZETA](https://vicone.com/products/xzeta)**——业界领先的汽车漏洞与 SBOM 管理平台。


**[了解更多 VicOne xZETA →](https://vicone.com/products/xzeta)**

---

## 为什么选择 radeis_sc2sbom？

- **最全面** — 根据项目类型，比竞争对手多检测 2.1%–58.8% 的包
- **5+ 项独特能力** — 唯一支持 Autotools、ROS 2、Git 子模块、CMake ExternalProject 和 AUTOSAR BSW 的工具
- **AI 模型支持** — GGUF 和 Safetensors 解析，支持 CycloneDX `machine-learning-model` 和 `pkg:huggingface` PURL
- **符合标准** — SPDX 2.3（JSON + Tag-Value）和 CycloneDX 1.5

## 快速开始

```bash
# 构建
git clone <repository-url>
cd radeis_sc2sbom
cargo build --release

# 扫描并生成所有格式
./target/release/radeis_sc2sbom --path . --output ./out

# 单一格式输出到文件
./target/release/radeis_sc2sbom --path . --format spdx-json --output ./out

# 单一格式输出到标准输出
./target/release/radeis_sc2sbom --path . --format cyclonedx-json
```

## 支持的生态系统

| 类别 | 生态系统 | 关键文件 |
|------|---------|---------|
| **AI/ML 模型** | Safetensors | `*.safetensors`, `config.json`, `generation_config.json`, `tokenizer_config.json` |
| **AI 模型** | GGUF | `*.gguf`, `config.json`, `README.md` |
| **AUTOSAR** | BSW 模块 + SWC 组件 | `*.arxml`、`*.epd`、BSW 目录、CMake/Makefile 令牌；版本来源：REVISION-LABEL + Doxygen 头文件 — **唯一工具** |
| **机器人** | ROS/ROS2 | `package.xml`, `setup.py` — **唯一 SBOM 工具** |
| **C/C++** | Autotools | `configure.ac`, `Makefile.am` — **唯一工具** |
| **C/C++** | CMake | `CMakeLists.txt`, `*.cmake` — ExternalProject **唯一工具** |
| **C/C++** | Conan / vcpkg / pkg-config | 锁文件、`.pc`、`conanfile.*` |
| **C/C++** | Meson / Bazel | `meson.build`, `WORKSPACE`, `MODULE.bazel` |
| **C/C++** | Makefile / .mk 文件 | 启发式解析含变量解析 |
| **版本控制** | Git 子模块 | `.gitmodules` 含 commit SHA — **唯一工具** |
| **Python** | pip / Poetry / Pipenv | `requirements.txt`, `poetry.lock`, `pyproject.toml` |
| **JavaScript** | npm | `package.json`, `package-lock.json`, `yarn.lock` |
| **Rust** | Cargo | `Cargo.toml`, `Cargo.lock` |
| **Java** | Gradle / Maven | `build.gradle`, `build.gradle.kts`, `pom.xml` |
| **Go / Ruby / PHP** | 标准 | `go.mod`, `Gemfile`, `composer.json` |

所有生态系统均包括许可证提取、供应商元数据和 CPE 标识符。

## 常用命令

```bash
# 完整扫描，生成所有格式
./target/release/radeis_sc2sbom --path . --output ./out

# 生产 SBOM（仅运行时 + 可选依赖）
./target/release/radeis_sc2sbom --path . \
  --production \
  --format spdx-json

# AUTOSAR 项目含供应商映射
./target/release/radeis_sc2sbom --path . \
  --supplier-config suppliers.yaml \
  --format cyclonedx-json --output ./out
```


## 主要选项

```
--path <PATH>                  要扫描的目录（必需）
--format <FORMAT>              console | spdx-json | spdx-tag-value | cyclonedx-json | all（默认：all）
--output <DIR>                 输出目录；单一格式时省略则输出到标准输出
--production                   仅包含运行时 + 可选依赖
--scope-filter <SCOPE>         runtime | build | test | dev | optional | provided
--supplier-config <PATH>       YAML 文件，将 AUTOSAR 组件名映射到供应商字符串
--bsw-config <PATH>            覆盖内置 AUTOSAR BSW 模块配置
--ros-distro <DISTRO>          jazzy | iron | humble（默认：jazzy）
--target-arch <ARCH>           .mk 条件解析的目标架构
--scan-submodules              Git 子模块扫描（默认：true）
--scan-c-build-systems         CMake、pkg-config、Autotools、Makefile、.mk（默认：true）
--scan-ai-models               GGUF + Safetensors AI 模型扫描（默认：true）
--compact-spdx                 SPDX 输出减小约 30%
```


完整参考请参阅 [docs/CLI.md](docs/CLI.md)。

## 新版本特性



- **独立 C 项目支持** — 无包清单文件的项目（如 NIST Juliet 测试套件）可通过回退模式自动检测并扫描
- **内部功能开关** — 扫描器仅在使用 `--features internal` 构建时编译；公开版本不受影响
<!-- END_INTERNAL -->


- **AUTOSAR 检测** — 自动检测 AUTOSAR 项目（通过 `.arxml` 文件、BSW/MCAL/RTE 目录名或构建文件中的 `AUTOSAR_VERSION` 令牌）
- **AUTOSAR 分类** — BSW 组件在 CycloneDX 属性和 SPDX ExternalRef 中标注 `autosar:layer`、`autosar:platform`
- **供应商映射** — `--supplier-config <yaml>` 将组件名映射到供应商字符串；输出 `autosar:supplier`（未映射时回退为 `NOASSERTION`）
- **单一格式支持 `--output`** — `--format spdx-json --output ./out` 现在会写入文件；省略 `--output` 则打印到标准输出

### v1.0.14 — 可靠性与 SBOM 质量

- 断开符号链接容忍 — 扫描器遇到断开的符号链接不再中止
- Makefile `$(VAR)` 过滤 — 变量引用不再泄漏到 `versionInfo`
- C/C++ 许可证解析 — `.pc` `License:` 字段 + 24 项已知库查找表
- 静态 Linux 二进制文件（musl）— 无 glibc 依赖，可在 Ubuntu 22.04+、Alpine 及任何 x86_64 Linux 上运行

### 历史版本

完整历史记录请参阅 [CHANGELOG.md](CHANGELOG.md) 和 [docs/WHATS_NEW.md](docs/WHATS_NEW.md)。

## 基准测试摘要

| 仓库 | radeis 包数 | 竞争对手最佳 | 优势 |
|------|------------|------------|------|
| curl（C） | 44 | 41（Syft） | +3 — 唯一 Autotools 支持 |

详见 [docs/BENCHMARKS.md](docs/BENCHMARKS.md)。

## 输出文件

默认位置：`./out/`

```
<project>_report.md          # 控制台报告（Markdown）
<project>_spdx.json          # SPDX 2.3 JSON
<project>_spdx.spdx          # SPDX 2.3 Tag-Value
<project>_cyclonedx.json     # CycloneDX 1.5 JSON
```

## 文档

- [CLI 参考](docs/CLI.md)
- [使用指南](docs/USAGE.md)
- [新版本特性](docs/WHATS_NEW.md)
- [范围分类](docs/SCOPE_CLASSIFICATION.md)
- [架构说明](docs/ARCHITECTURE.md)
- [基准测试](docs/BENCHMARKS.md)

## 环境要求

- Rust 1.70+

## 许可证

MIT — 详见 [LICENSE](LICENSE)

## 作者

Amean Lin · William Chang

## 支持

- **问题反馈与功能请求** — [提交 issue](../../issues)
- **企业与产品咨询** — [联系 VicOne](https://vicone.com/lab_r7/contact-us/)

---

<p align="center">
  Made with ❤️ for supply chain security
</p>
