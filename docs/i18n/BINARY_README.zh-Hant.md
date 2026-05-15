# radeis_sc2sbom - 二進位檔案發佈

用於從原始碼產生軟體物料清單（SBOM）的預先建置二進位檔案。

## 可用二進位檔案


### macOS（Apple Silicon - M1/M2/M3）
- **檔案：** `radeis_sc2sbom-macos-arm64`
- **架構：** ARM64 (aarch64)
- **平台：** macOS 11.0 或更新版本

### macOS（Intel）
- **檔案：** `radeis_sc2sbom-macos-x86_64`
- **架構：** x86_64
- **平台：** macOS 10.12 或更新版本

### Linux（x86_64 靜態）
- **檔案：** `radeis_sc2sbom-linux`
- **架構：** x86_64
- **平台：** 任何 Linux x86_64（靜態二進位檔，無需 glibc——可於 Ubuntu 22.04+、24.04+、Alpine 等上執行）

### Windows（x86_64）
- **檔案：** `radeis_sc2sbom-windows.exe`
- **架構：** x86_64
- **平台：** Windows 10 或更新版本

## 安裝

### macOS / Linux

1. 下載適合您平台的二進位檔案
2. 賦予可執行權限：
   ```bash
   chmod +x radeis_sc2sbom-macos-arm64
   ```
3. （選用）移動至 PATH 中的目錄：
   ```bash
   sudo mv radeis_sc2sbom-macos-arm64 /usr/local/bin/radeis_sc2sbom
   ```

### Windows

1. 下載 `radeis_sc2sbom-windows.exe`
2. 在命令提示字元或 PowerShell 中執行
3. （選用）加入 PATH 以便於存取

## 快速入門

**注意：** 請將 `<binary-name>` 替換為您平台對應的二進位檔案名稱：
- macOS ARM：`radeis_sc2sbom-macos-arm64`
- macOS Intel：`radeis_sc2sbom-macos-x86_64`
- Linux：`radeis_sc2sbom-linux`
- Windows：`radeis_sc2sbom-windows.exe`

```bash
# 基本用法 - 掃描目前目錄
./<binary-name> --path .

# 掃描特定專案
./<binary-name> --path /path/to/project

# 產生 SPDX 格式
./<binary-name> --path . --format spdx
```

## 常用選項

| 選項 | 說明 | 範例 |
|------|------|------|
| `--path <PATH>` | 要掃描的路徑（預設：目前目錄） | `--path ./my-project` |
| `--format <FORMAT>` | 輸出格式：console、spdx、cyclonedx、all | `--format spdx` |
| `--output <DIR>` | 輸出目錄（預設：./out） | `--output ./sbom-reports` |
| `--tree-style <STYLE>` | 相依性樹狀結構樣式：classic、compact、flat | `--tree-style compact` |
| `--vendor` | 包含 vendor 目錄 | `--vendor` |
| `--exclude <PATTERN>` | 排除模式（可多次使用） | `--exclude "test/*"` |
| `--bsw-config <PATH>` | 自訂 AUTOSAR BSW 模組設定（YAML） | `--bsw-config ./bsw.yaml` |
| `--supplier-config <PATH>` | AUTOSAR 元件對供應商的對應（YAML） | `--supplier-config ./suppliers.yaml` |

## 支援的生態系統

### 鎖定檔案（含階層式相依性樹狀結構）
- **npm** - package-lock.json
- **Cargo**（Rust）- Cargo.lock
- **Poetry**（Python）- poetry.lock

### 資訊清單檔案
- **npm** - package.json
- **Cargo**（Rust）- Cargo.toml
- **Python** - requirements.txt、setup.py、pyproject.toml
- **Go** - go.mod
- **Maven**（Java）- pom.xml
- **ROS** - package.xml
- **JavaScript/TypeScript** - 原始碼匯入

### C / C++（建置系統與套件管理器）
- **Makefile** - GNU Make 建置檔案
- **Makefile.am** - Automake 建置檔案
- **configure.ac** - Autotools 設定（pkg-config 偵測）
- **.mk files** - 架構感知的 Make 片段檔案
- **pkg-config (.pc files)** - 函式庫相依性描述符
- **Shared libraries (.so scanner)** - 動態函式庫相依性偵測
- **Vendored 3rd-party** - `3rdparty/`、`3rd_party/`、`third_party/` 目錄偵測
- **Conan** - conanfile.txt / conanfile.py
- **vcpkg** - vcpkg.json
- **CMake** - CMakeLists.txt（透過 FetchContent 和 ExternalProject_Add）

### AUTOSAR
- **`.arxml` 檔案** - 自動偵測；BSW 元件依層級、平台和供應商分類
- **BSW 模組設定** - 內建預設值；可透過 `--bsw-config` 覆寫
- **供應商對應** - 透過 `--supplier-config` 指定選用 YAML 檔案

## 範例

**注意：** 範例使用 `<binary-name>` 作為占位符，請替換為您平台對應的二進位檔案名稱。

### 為 Node.js 專案產生完整 SBOM
```bash
./<binary-name> \
  --path ./my-node-app \
  --format all \
  --tree-style classic
```

### 掃描 Rust 專案
```bash
./<binary-name> \
  --path ./my-rust-app \
  --format cyclonedx
```

### 掃描 Python 專案並排除測試目錄
```bash
./<binary-name> \
  --path ./my-python-app \
  --exclude "tests/*" \
  --exclude "venv/*" \
  --format spdx
```

### 僅產生主控台報告並使用精簡樹狀結構樣式
```bash
./<binary-name> \
  --path . \
  --format console \
  --tree-style compact \
  --output ./reports
```

## 輸出格式

### Console 格式
人類可讀的 Markdown 報告，包含：
- 專案摘要
- 依生態系統分類的相依性統計
- 階層式相依性樹狀結構

### SPDX 格式（spdx.json）
業界標準 SBOM 格式，相容於：
- SPDX 工具與驗證器
- 授權合規工具
- 供應鏈安全平台

### CycloneDX 格式（cdx.json）
輕量級 SBOM 格式，包含：
- 元件清單
- 相依性關係


## 說明

查看完整命令列參考：
```bash
./<binary-name> --help
```

## 原始碼與問題回報

- **儲存庫：** https://github.com/VicOne-RD/radeis_sc2sbom
- **問題回報：** https://github.com/VicOne-RD/radeis_sc2sbom/issues
- **文件：** 詳細文件請參閱主儲存庫 README

## 版本資訊

查看二進位檔案版本：
```bash
./<binary-name> --version
```

## 授權條款

MIT License

Copyright (c) 2026 VicOne Inc.

特此免費授予任何取得本軟體及相關文件檔案（以下簡稱「軟體」）副本的人，不受限制地處理本軟體的權利，包括但不限於使用、複製、修改、合併、發布、散佈、再授權及販售本軟體副本的權利，以及允許取得本軟體的人員這樣做，但須符合以下條件：

上述版權聲明和本授權聲明應包含在本軟體的所有副本或重要部分中。

本軟體依「現狀」提供，不附任何明示或暗示的保證，包括但不限於對適銷性、特定用途適用性及不侵權的保證。在任何情況下，作者或版權持有人均不對因本軟體或本軟體的使用或其他交易而產生的任何索賠、損害或其他責任承擔責任，無論是合約訴訟、侵權行為還是其他原因。
