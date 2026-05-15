# radeis_sc2sbom - 二进制文件分发

用于从源代码生成软件物料清单（SBOM）的预构建二进制文件。

## 可用二进制文件


### macOS（Apple Silicon - M1/M2/M3）
- **文件：** `radeis_sc2sbom-macos-arm64`
- **架构：** ARM64 (aarch64)
- **平台：** macOS 11.0 或更高版本

### macOS（Intel）
- **文件：** `radeis_sc2sbom-macos-x86_64`
- **架构：** x86_64
- **平台：** macOS 10.12 或更高版本

### Linux（x86_64 静态）
- **文件：** `radeis_sc2sbom-linux`
- **架构：** x86_64
- **平台：** 任何 Linux x86_64（静态二进制文件,无需 glibc——可在 Ubuntu 22.04+、24.04+、Alpine 等上运行）

### Windows（x86_64）
- **文件：** `radeis_sc2sbom-windows.exe`
- **架构：** x86_64
- **平台：** Windows 10 或更高版本

## 安装

### macOS / Linux

1. 下载适合您平台的二进制文件
2. 赋予可执行权限：
   ```bash
   chmod +x radeis_sc2sbom-macos-arm64
   ```
3. （可选）移动到 PATH 中的目录：
   ```bash
   sudo mv radeis_sc2sbom-macos-arm64 /usr/local/bin/radeis_sc2sbom
   ```

### Windows

1. 下载 `radeis_sc2sbom-windows.exe`
2. 在命令提示符或 PowerShell 中运行
3. （可选）添加到 PATH 以便于访问

## 快速入门

**注意：** 将 `<binary-name>` 替换为您平台对应的二进制文件名：
- macOS ARM：`radeis_sc2sbom-macos-arm64`
- macOS Intel：`radeis_sc2sbom-macos-x86_64`
- Linux：`radeis_sc2sbom-linux`
- Windows：`radeis_sc2sbom-windows.exe`

```bash
# 基本用法 - 扫描当前目录
./<binary-name> --path .

# 扫描指定项目
./<binary-name> --path /path/to/project

# 生成 SPDX 格式
./<binary-name> --path . --format spdx
```

## 常用选项

| 选项 | 说明 | 示例 |
|------|------|------|
| `--path <PATH>` | 要扫描的路径（默认：当前目录） | `--path ./my-project` |
| `--format <FORMAT>` | 输出格式：console、spdx、cyclonedx、all | `--format spdx` |
| `--output <DIR>` | 输出目录（默认：./out） | `--output ./sbom-reports` |
| `--tree-style <STYLE>` | 依赖树样式：classic、compact、flat | `--tree-style compact` |
| `--vendor` | 包含 vendor 目录 | `--vendor` |
| `--exclude <PATTERN>` | 排除模式（可多次使用） | `--exclude "test/*"` |
| `--bsw-config <PATH>` | 自定义 AUTOSAR BSW 模块配置（YAML） | `--bsw-config ./bsw.yaml` |
| `--supplier-config <PATH>` | AUTOSAR 组件到供应商的映射（YAML） | `--supplier-config ./suppliers.yaml` |

## 支持的生态系统

### 锁文件（含层级依赖树）
- **npm** - package-lock.json
- **Cargo**（Rust）- Cargo.lock
- **Poetry**（Python）- poetry.lock

### 清单文件
- **npm** - package.json
- **Cargo**（Rust）- Cargo.toml
- **Python** - requirements.txt、setup.py、pyproject.toml
- **Go** - go.mod
- **Maven**（Java）- pom.xml
- **ROS** - package.xml
- **JavaScript/TypeScript** - 源代码导入

### C / C++（构建系统与包管理器）
- **Makefile** - GNU Make 构建文件
- **Makefile.am** - Automake 构建文件
- **configure.ac** - Autotools 配置（pkg-config 检测）
- **.mk files** - 架构感知的 Make 片段文件
- **pkg-config (.pc files)** - 库依赖描述符
- **Shared libraries (.so scanner)** - 动态库依赖检测
- **Vendored 3rd-party** - `3rdparty/`、`3rd_party/`、`third_party/` 目录检测
- **Conan** - conanfile.txt / conanfile.py
- **vcpkg** - vcpkg.json
- **CMake** - CMakeLists.txt（通过 FetchContent 和 ExternalProject_Add）

### AUTOSAR
- **`.arxml` 文件** - 自动检测；BSW 组件按层级、平台和供应商分类
- **BSW 模块配置** - 内置默认值；可通过 `--bsw-config` 覆盖
- **供应商映射** - 通过 `--supplier-config` 指定可选 YAML 文件

## 示例

**注意：** 示例使用 `<binary-name>` 作为占位符，请替换为您平台对应的二进制文件名。

### 为 Node.js 项目生成完整 SBOM
```bash
./<binary-name> \
  --path ./my-node-app \
  --format all \
  --tree-style classic
```

### 扫描 Rust 项目
```bash
./<binary-name> \
  --path ./my-rust-app \
  --format cyclonedx
```

### 扫描 Python 项目并排除测试目录
```bash
./<binary-name> \
  --path ./my-python-app \
  --exclude "tests/*" \
  --exclude "venv/*" \
  --format spdx
```

### 仅生成控制台报告并使用紧凑树样式
```bash
./<binary-name> \
  --path . \
  --format console \
  --tree-style compact \
  --output ./reports
```

## 输出格式

### Console 格式
人类可读的 Markdown 报告，包含：
- 项目摘要
- 按生态系统划分的依赖统计
- 层级依赖树

### SPDX 格式（spdx.json）
行业标准 SBOM 格式，兼容：
- SPDX 工具和验证器
- 许可证合规工具
- 供应链安全平台

### CycloneDX 格式（cdx.json）
轻量级 SBOM 格式，包含：
- 组件清单
- 依赖关系


## 帮助

查看完整命令行参考：
```bash
./<binary-name> --help
```

## 源代码与问题反馈

- **仓库：** https://github.com/VicOne-OSS-CDC/radeis_sc2sbom
- **问题反馈：** https://github.com/VicOne-OSS-CDC/radeis_sc2sbom/issues
- **文档：** 详细文档请参阅主仓库 README

## 版本信息

查看二进制文件版本：
```bash
./<binary-name> --version
```

## 许可证

MIT License

Copyright (c) 2026 VicOne Inc.

特此免费授予任何获得本软件及相关文档文件（以下简称"软件"）副本的人不受限制地处理本软件的权利，包括但不限于使用、复制、修改、合并、发布、分发、再许可及销售本软件副本的权利，以及允许获得本软件的人员这样做，但须符合以下条件：

上述版权声明和本许可声明应包含在本软件的所有副本或重要部分中。

本软件按"原样"提供，不附任何明示或暗示的保证，包括但不限于对适销性、特定用途适用性及不侵权的保证。在任何情况下，作者或版权持有人均不对因本软件或本软件的使用或其他交易而产生的任何索赔、损害或其他责任承担责任，无论是合同诉讼、侵权行为还是其他原因。
