<a href="../../README.md">English</a> | <a href="README.zh-Hans.md">简体中文</a> | 繁體中文 | <a href="README.ja.md">日本語</a>

<p align="center">
  <img src="../../docs/images/icon.png" alt="Sourcecode to SBOM" width="80">
</p>

# radeis_sc2sbom

**快速 SBOM 產生器，內建漏洞偵測。** 多生態系統支援，具備獨特能力：**唯一**支援 C/C++ Autotools、ROS 2、Git 子模組、CMake ExternalProject 和 AUTOSAR BSW 的工具。

## 借助 VicOne xZETA 充分發揮 SBOM 的價值

`radeis_sc2sbom` 產生的 SBOM 完全符合 **SPDX 2.3** 和 **CycloneDX 1.5** 標準，可直接匯入 **[VicOne xZETA](https://vicone.com/products/xzeta)**——業界領先的車用漏洞與 SBOM 管理平台。


**[了解更多 VicOne xZETA →](https://vicone.com/products/xzeta)**

---

## 為什麼選擇 radeis_sc2sbom？

- **最全面** — 根據專案類型，比競爭對手多偵測 2.1%–58.8% 的套件
- **5+ 項獨特能力** — 唯一支援 Autotools、ROS 2、Git 子模組、CMake ExternalProject 和 AUTOSAR BSW 的工具
- **AI 模型支援** — GGUF 和 Safetensors 解析，支援 CycloneDX `machine-learning-model` 和 `pkg:huggingface` PURL
- **符合標準** — SPDX 2.3（JSON + Tag-Value）和 CycloneDX 1.5

## 快速開始

```bash
# 建置
git clone <repository-url>
cd radeis_sc2sbom
cargo build --release

# 掃描並產生所有格式
./target/release/radeis_sc2sbom --path . --output ./out

# 單一格式輸出到檔案
./target/release/radeis_sc2sbom --path . --format spdx-json --output ./out

# 單一格式輸出到標準輸出
./target/release/radeis_sc2sbom --path . --format cyclonedx-json
```

## 支援的生態系統

| 類別 | 生態系統 | 關鍵檔案 |
|------|---------|---------|
| **AI/ML 模型** | Safetensors | `*.safetensors`, `config.json`, `generation_config.json`, `tokenizer_config.json` |
| **AI 模型** | GGUF | `*.gguf`, `config.json`, `README.md` |
| **AUTOSAR** | BSW 模組 + SWC 元件 | `*.arxml`、`*.epd`、BSW 目錄、CMake/Makefile 令牌；版本來源：REVISION-LABEL + Doxygen 標頭 — **唯一工具** |
| **機器人** | ROS/ROS2 | `package.xml`, `setup.py` — **唯一 SBOM 工具** |
| **C/C++** | Autotools | `configure.ac`, `Makefile.am` — **唯一工具** |
| **C/C++** | CMake | `CMakeLists.txt`, `*.cmake` — ExternalProject **唯一工具** |
| **C/C++** | Conan / vcpkg / pkg-config | 鎖定檔、`.pc`、`conanfile.*` |
| **C/C++** | Meson / Bazel | `meson.build`, `WORKSPACE`, `MODULE.bazel` |
| **C/C++** | Makefile / .mk 檔案 | 啟發式解析含變數解析 |
| **版本控制** | Git 子模組 | `.gitmodules` 含 commit SHA — **唯一工具** |
| **Python** | pip / Poetry / Pipenv | `requirements.txt`, `poetry.lock`, `pyproject.toml` |
| **JavaScript** | npm | `package.json`, `package-lock.json`, `yarn.lock` |
| **Rust** | Cargo | `Cargo.toml`, `Cargo.lock` |
| **Java** | Gradle / Maven | `build.gradle`, `build.gradle.kts`, `pom.xml` |
| **Go / Ruby / PHP** | 標準 | `go.mod`, `Gemfile`, `composer.json` |

所有生態系統均包括授權提取、供應商元資料和 CPE 識別碼。

## 常用指令

```bash
# 完整掃描，產生所有格式
./target/release/radeis_sc2sbom --path . --output ./out

# 生產 SBOM（僅執行時 + 可選相依性）
./target/release/radeis_sc2sbom --path . \
  --production \
  --format spdx-json

# AUTOSAR 專案含供應商對應
./target/release/radeis_sc2sbom --path . \
  --supplier-config suppliers.yaml \
  --format cyclonedx-json --output ./out
```


## 主要選項

```
--path <PATH>                  要掃描的目錄（必要）
--format <FORMAT>              console | spdx-json | spdx-tag-value | cyclonedx-json | all（預設：all）
--output <DIR>                 輸出目錄；單一格式時省略則輸出到標準輸出
--production                   僅包含執行時 + 可選相依性
--scope-filter <SCOPE>         runtime | build | test | dev | optional | provided
--supplier-config <PATH>       YAML 檔案，將 AUTOSAR 元件名稱對應到供應商字串
--bsw-config <PATH>            覆蓋內建 AUTOSAR BSW 模組設定
--ros-distro <DISTRO>          jazzy | iron | humble（預設：jazzy）
--target-arch <ARCH>           .mk 條件解析的目標架構
--scan-submodules              Git 子模組掃描（預設：true）
--scan-c-build-systems         CMake、pkg-config、Autotools、Makefile、.mk（預設：true）
--scan-ai-models               GGUF + Safetensors AI 模型掃描（預設：true）
--compact-spdx                 SPDX 輸出減少約 30%
```


完整參考請參閱 [docs/CLI.md](../CLI.md)。

## 新版本特性



- **獨立 C 專案支援** — 無套件清單檔案的專案（如 NIST Juliet 測試套件）可透過回退模式自動偵測並掃描
- **內部功能開關** — 掃描器僅在使用 `--features internal` 建置時編譯；公開版本不受影響
<!-- END_INTERNAL -->


- **AUTOSAR 偵測** — 自動偵測 AUTOSAR 專案（透過 `.arxml` 檔案、BSW/MCAL/RTE 目錄名稱或建置檔案中的 `AUTOSAR_VERSION` 令牌）
- **AUTOSAR 分類** — BSW 元件在 CycloneDX 屬性和 SPDX ExternalRef 中標注 `autosar:layer`、`autosar:platform`
- **供應商對應** — `--supplier-config <yaml>` 將元件名稱對應到供應商字串；輸出 `autosar:supplier`（未對應時回退為 `NOASSERTION`）
- **單一格式支援 `--output`** — `--format spdx-json --output ./out` 現在會寫入檔案；省略 `--output` 則列印到標準輸出

### v1.0.14 — 可靠性與 SBOM 品質

- 斷開符號連結容忍 — 掃描器遇到斷開的符號連結不再中止
- Makefile `$(VAR)` 過濾 — 變數引用不再洩漏到 `versionInfo`
- C/C++ 授權解析 — `.pc` `License:` 欄位 + 24 項已知函式庫查找表
- 靜態 Linux 二進位檔案（musl）— 無 glibc 相依性，可在 Ubuntu 22.04+、Alpine 及任何 x86_64 Linux 上執行

### 歷史版本

完整歷史記錄請參閱 [CHANGELOG.md](../../CHANGELOG.md) 和 [docs/WHATS_NEW.md](../WHATS_NEW.md)。

## 基準測試摘要

| 儲存庫 | radeis 套件數 | 競爭對手最佳 | 優勢 |
|--------|-------------|------------|------|
| curl（C） | 44 | 41（Syft） | +3 — 唯一 Autotools 支援 |

詳見 [docs/BENCHMARKS.md](../BENCHMARKS.md)。

## 輸出檔案

預設位置：`./out/`

```
<project>_report.md          # 控制台報告（Markdown）
<project>_spdx.json          # SPDX 2.3 JSON
<project>_spdx.spdx          # SPDX 2.3 Tag-Value
<project>_cyclonedx.json     # CycloneDX 1.5 JSON
```

## 文件

- [CLI 參考](../CLI.md)
- [使用指南](../USAGE.md)
- [新版本特性](../WHATS_NEW.md)
- [範圍分類](../SCOPE_CLASSIFICATION.md)
- [架構說明](../ARCHITECTURE.md)
- [基準測試](../BENCHMARKS.md)

## 環境需求

- Rust 1.70+

## 授權

MIT — 詳見 [LICENSE](../../LICENSE)

## 作者

Amean Lin · William Chang

## 支援

- **問題回報與功能請求** — [提交 issue](../../issues)
- **企業與產品諮詢** — [聯繫 VicOne](https://vicone.com/lab_r7/contact-us/)

---

<p align="center">
  Made with ❤️ for supply chain security
</p>
