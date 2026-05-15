# 更新日誌

本文件記錄專案所有重要變更。

格式基於 [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)，
本專案遵循 [語意化版本](https://semver.org/spec/v2.0.0.html)。

## [Unreleased]

## [1.0.18] - 2026-05-13

### 新增

### 變更
- Phase 24 調校：17 項規則層級變更相較 v1.0.18 前基線減少 89,479 個誤判

### 移除
- `--cppcheck-path` CLI 選項（cppcheck 整合已由 AST 掃描器取代）
- `check_plaintext_password` 死碼函式

## [1.0.17] - 2026-05-11

### 新增（內部版本）
- **`--cppcheck-path <PATH>`**：覆寫 cppcheck 二進位檔案的 PATH 查找
- **SARIF 2.1 輸出**：`<project>_static_analysis.sarif` 與 `_static_analysis.md` 並排寫出；相容 GitHub Code Scanning、VS Code Problem Matcher 和 CI/CD 流水線
- **`--sarif-output <PATH>`**：將 SARIF 報告寫入自訂路徑
- **SARIF 指紋**：每個 `SarifResult` 的 SHA-256 指紋，用於跨執行的穩定去重
- **`--sarif-baseline <PATH>`**：將目前發現項與先前 SARIF 執行比對；僅回報新發現項——實現僅回報回歸的 CI 閘控
- **AUTOSAR arxml 解析**：`.arxml` 檔案現已完整解析相依性——提取 `SW-COMPONENT-PROTOTYPE`、`BSW-MODULE-DESCRIPTION` 及所有 SWC 型別定義元素作為 `autosar` 生態系統相依性
- **AUTOSAR 版本提取**：`.epd` 檔案（`ECUC-MODULE-DEF/REVISION-LABEL`）和 Doxygen C/H 標頭（`SW Version : X.Y.Z`）掃描，為 AUTOSAR 相依性填充真實版本字串而非 `unspecified`
- **AUTOSAR 生態系統升級**：Makefile `-lFoo` 旗標發現的 `system` 生態系統相依性在找到符合的 `.epd` 或 Doxygen 版本時升級為 `autosar` 生態系統——消除重複條目和誤分類的連結器相依性

### 修復
- AUTOSAR 專案中深度 > 3 的 C/C++ 檔案未被 SAST 掃描——`has_c_cpp_files` 最大深度從 3 提升至 6
- 相同 `(name, ecosystem)` 的 AUTOSAR 相依性有版本化和 `unspecified` 條目時現已去重——版本化條目優先
- 存在同名 `autosar` 生態系統條目時 `system` 生態系統條目被抑制

## [1.0.16] - 2026-05-10

### 新增
- **回退模式**：當未找到清單檔案派生的元件目錄但掃描根目錄下存在 C/C++ 原始檔案時，自動插入合成的 `(project_name, "C/C++") → scan_root` 條目——可掃描無清單儲存庫（如 NIST Juliet 測試套件）
- **`has_c_cpp_files` 輔助函式**：淺層 `WalkDir`（最大深度 3）檢查，複用 `is_c_cpp_source` 謂詞；供回退模式使用以避免對非 C 儲存庫的誤判
- **`resolve_component_dir` 輔助函式**：透過三種策略（精確名稱、`lib` 前綴、大小寫不敏感掃描）將清單宣告的 C/C++ 相依性映射到供應商原始碼子目錄；對無匹配供應商子目錄的相依性傳回 `None`，防止來自外部/系統相依性的誤判發現項

### 內部
- `build-all.sh` 增加 `--internal` 旗標：同時建置公開版本和內部版本，內部版本二進位檔案名稱新增 `-internal` 後綴

## [1.0.15] - 2026-05-09

### 新增
- **AUTOSAR 偵測**：預掃描 `detect_autosar()` 在掃描前執行；透過 `.arxml` 檔案（DET-01）、BSW/MCAL/RTE/AUTOSAR/SWC 目錄名稱（DET-02）和建置檔案中的 `AUTOSAR_VERSION`/`AR_VERSION` 令牌（DET-03）偵測專案
- **AUTOSAR 分類**：`classify_autosar_components()` 將相依性名稱與內建 BSW 模組設定比對；將符合的元件升級為 `ecosystem="autosar"`，附帶 `AutosarMetadata`（module_name、layer、platform）
- **AUTOSAR 輸出 — CycloneDX**：AUTOSAR 元件輸出 `autosar:layer` 和 `autosar:platform` 屬性（如 `"BSW-Memory"`、`"Classic"`）
- **AUTOSAR 輸出 — SPDX**：AUTOSAR 元件以 `ExternalRef OTHER` 條目輸出 `autosar:layer` 和 `autosar:platform`
- **供應商設定**：`--supplier-config <path>` 接受 YAML 檔案，將 AUTOSAR 元件名稱對應到供應商字串；已對應元件在 CycloneDX 屬性和 SPDX ExternalRef 中輸出 `autosar:supplier`；未對應元件輸出 `NOASSERTION`
- **BSW 設定覆蓋**：`--bsw-config <path>` 用自訂 YAML 檔案覆蓋內建 AUTOSAR BSW 模組設定

### 變更
- `--output` 現在對所有單一格式（`spdx-json`、`spdx-tag-value`、`cyclonedx-json`、`console`）生效；單一格式省略 `--output` 時輸出到標準輸出

### 內部

## [1.0.14] - 2026-04-24

### 修正
- 掃描器遇到斷開的符號連結不再中止——發出 `Warning: skipping` 並繼續（涵蓋全部 5 處 WalkDir 走訪點：scanner、回退匯入掃描、C 解析器）
- `$(OPENSSL_VERSION)` 等 Makefile 變數參照不再洩漏至 SPDX `versionInfo`——在解析器層及每個 SPDX/CycloneDX 版本輸出點進行過濾，改為輸出 `NOASSERTION`
- C/C++ 程式庫授權現從 `.pc` 的 `License:` 欄位和已知程式庫查找表（24 個常見系統程式庫）解析，不再全部為 `NOASSERTION`

### 變更
- Linux 發佈二進位檔現透過 musl（`x86_64-unknown-linux-musl`）靜態連結——消除 glibc 版本相依性；可於 Ubuntu 22.04+、24.04、Alpine 及任何 x86_64 Linux 上執行
- 為 `build-all.sh` 新增 musl 交叉連結器工具鏈守衛

### 內部
- 將 `warn_on_walkdir_err` 輔助函式擷取至 `src/util/mod.rs`，合併了 5 處相同的 `filter_map` closure
- 使用 Unix 專屬 API 的跨平台測試現由 `#[cfg(unix)]` 守衛

## [1.0.13] - 2026-04-14

### 新增
- **AI 模型**：多模態子模型分解 — 複合模型（Gemma-4、LLaVA、Qwen-VL）分解為文字、視覺和音訊子模型元件
- **AI 模型**：新增 `SubModelInfo` 結構體，擷取每個子模型的架構資訊：model_type、layers、hidden_size、heads、dtype、vocab_size、上下文視窗及模態特定欄位（patch_size、conv_kernel_size 等）
- **AI 模型**：守衛條件 — 僅對真正的多模態模型（同時存在 text_config + vision/audio_config）產生子模型
- **CycloneDX**：在父 AI 模型元件內巢狀 `components` 陣列，每個子模型帶有 `radeis:ai:sub_model:*` 屬性
- **SPDX**：子套件與父模型之間透過 `CONTAINS` 包含關係連接
- **主控台**：子模型摘要表，顯示模態、模型類型、層數、隱藏層大小、注意力頭數、dtype 及模態特定擴展資訊
- **測試**：新增 5 項測試（4 項 Safetensors + 1 項 GGUF），涵蓋多模態擷取、純文字守衛、僅視覺+文字及 GGUF 補強

## [1.0.12] - 2026-04-14

### 新增
- **AI 模型**：從 HuggingFace 伴隨檔案擷取豐富中繼資料，適用於 Safetensors 和 GGUF 儲存庫
- **AI 模型**：解析 `generation_config.json` — temperature、top_k、top_p 推論預設值
- **AI 模型**：解析 `tokenizer_config.json` — processor_class、model_max_length（含天文數字截斷）
- **AI 模型**：解析 `preprocessor_config.json` — 影像、音訊和視訊處理器類型及參數
- **AI 模型**：解析 `README.md` YAML frontmatter — base_model（字串或清單）、license、pipeline_tag、quantized_by、prompt_template、tags、languages、datasets
- **AI 模型**：擴展 `config.json` 擷取 — model_type、text_config（hidden_layers、hidden_size、attention_heads、max_position_embeddings）、多模態偵測（vision_config、audio_config）
- **AI 模型**：Dtype 回退鏈 — `torch_dtype` > `dtype` > `text_config.dtype`
- **AI 模型**：透過 `adapter_config.json` 偵測 LoRA/QLoRA adapter
- **AI 模型**：GGUF 伴隨檔案補強 — 二進位中繼資料始終優先，伴隨檔案填補空缺
- **AI 模型**：來自二進位和 README 來源的 tags、languages 和 datasets 去重聯集合併
- **AI 模型**：所有伴隨檔案讀取設有 1 MB 安全上限
- **AI 模型**：不區分大小寫的 README.md 檔名比對及 CRLF 換行符號支援
- **CycloneDX**：新增約 25 個 `radeis:ai:*` 屬性，提供豐富的 AI 模型中繼資料
- **SPDX**：sourceInfo 擴展，新增 model_type、上下文視窗和模態摘要
- **主控台**：AI Model Details 表格擴展，新增架構、多模態、生成和來源追溯區段
- **測試**：`safetensors_tests.rs` 新增 8 項測試、`gguf_tests.rs` 新增 6 項測試

## [1.0.11] - 2026-04-13

### 新增
- **AI 模型**：Safetensors AI 模型 SBOM 解析 — 支援 `.safetensors`、`model.safetensors.index.json` 和 `config.json`
- **AI 模型**：目錄級掃描 — 無論分片數量多少，每個模型僅產生一筆 Dependency 記錄
- **AI 模型**：HuggingFace Safetensors 模型的 `pkg:huggingface` PURL
- **AI 模型**：Safetensors 模型的 CycloneDX `machine-learning-model` 元件類型及 `modelCard`
- **AI 模型**：分片去重 — 多分片模型（如 `model-00001-of-00002.safetensors`）合併為單筆 SBOM 記錄
- **AI 模型**：新增 `AIModelMetadata` 欄位：`safetensors_format`、`total_size_bytes`、`shard_count`、`torch_dtype`、`transformers_version`、`vocab_size`
- **測試**：`tests/parser_tests/safetensors_tests.rs` 新增 12 項測試

## [1.0.10] - 2026-04-13

### 新增
- **Java/Gradle**：完整解析 `build.gradle`（Groovy DSL）和 `build.gradle.kts`（Kotlin DSL）的相依性 — 此前僅支援偵測
- **Java/Gradle**：字串記法（`'group:artifact:version'`）、映射記法（`group: 'g', name: 'a', version: 'v'`）、平台/BOM 支援
- **Java/Gradle**：範圍分類 — `testImplementation` → 測試、`compileOnly` → 已提供、`annotationProcessor`/`kapt`/`ksp` → 建置
- **Java/Gradle**：Android 專案支援（`androidTestImplementation`、`androidTestCompile`）

### 變更
- **Java**：Gradle 狀態在生態系統表中從「僅偵測」升級為「生產就緒」

## [1.0.9] - 2026-04-10

### 新增
- **AI 模型**：GGUF 二進位解析器，支援中繼資料擷取（架構、量化類型、張量資訊、上下文長度）
- **AI 模型**：CycloneDX `machine-learning-model` 元件類型，附帶 `modelCard`（訓練參數、資料集）
- **AI 模型**：SPDX `pkg:huggingface` PURL，用於 AI 模型相依性識別
- **AI 模型**：完整性驗證 — 張量參數交叉驗證及 SHA-256 雜湊，用於模型檔案真實性校驗
- **AI 模型**：常見 AI 模型授權條款的 SPDX 格式正規化
- **CLI**：`--scan-ai-models` 旗標，用於啟用 GGUF 模型掃描（預設值：true）

### 變更
- **CLI**：將 5 個 C/C++ 建置系統旗標（`--scan-cmake`、`--scan-pkgconfig`、`--scan-autotools`、`--scan-makefiles`、`--scan-mk-files`）合併為單一的 `--scan-c-build-systems` 旗標
- **CLI**：將 `--meson-parse-subprojects` 合併至 `--scan-meson`（啟用 Meson 掃描時始終生效）
- **核心**：`scan_directory()` 從 20 個參數精簡至 13 個

### 移除
- **CLI**：`--resolve-system-deps` 旗標（無效程式碼 — 從未連接到任何實作）
- **CLI**：`--meson-parse-subprojects` 旗標（啟用 `--scan-meson` 時子專案解析現已始終啟用）

## [1.0.7] - 2026-03-12

### 變更 - 漏洞掃描改為選用啟用

- **預設輸出更簡潔** — 停用掃描時，漏洞摘要列、風險評估區段及各套件詳情均不再顯示

## [1.0.6] - 2026-03-04

### 新增 - 正式環境 SBOM 過濾與自動化相依性範圍分類

**第 1-3 階段完成：核心分類系統**
- **自動範圍分類**，適用於所有相依性（v1.0.6 第 1-3 階段）
  - 6 種範圍類型：執行階段、建置、測試、開發、選用、已提供
  - 多策略分類：生態系統、名稱模式、目錄分析
  - 信心分數（0.0-1.0），附詳細推論說明
  - 支援 10 種以上生態系統（npm、PIP、cargo、SYSTEM、BUILD-CONFIG 等）

**第 4 階段完成：全面測試與驗證**
- **42 項新整合測試**，驗證範圍過濾與分類準確性
  - 14 項範圍過濾整合測試
  - 11 項端對端正式環境模式測試
  - 17 項真實情境分類驗證測試
- **全部 609 項測試通過**（203 個函式庫測試 + 203 個二進位測試 + 200 個整合測試 + 3 個文件測試）
- **已使用真實相依性進行驗證**：
  - 常見 C/C++ 執行階段函式庫（zlib、curl、openssl、protobuf）
  - 建置工具（cmake、gcc、ninja、meson）
  - 測試框架（pytest、jest、gtest、unity）
  - 開發工具（pylint、black、eslint、prettier）
  - Web 框架（django、flask、express、react）

**第 5 階段完成：文件**
- **v1.0.6 功能的完整文件**
  - 更新 [README.md](README.md)，新增正式環境模式範例
  - 新增 [SCOPE_CLASSIFICATION.md](docs/SCOPE_CLASSIFICATION.md) — 完整的範圍過濾指南（300 行以上）
  - 更新 [CLI.md](docs/CLI.md)，新增範圍過濾選項
  - CHANGELOG.md 新增第 5 階段完成說明
- **文件涵蓋內容**：
  - 快速上手範例
  - 相依性範圍說明（6 種類型）
  - 分類方法（4 種策略）
  - 過濾選項及範例
  - 信心分數解讀
  - 疑難排解指南
  - 正式環境、安全合規的最佳實踐
  - 真實情境驗證指標

**範圍過濾 CLI：**
- **`--scope-filter <SCOPE>`** — 依範圍過濾相依性
  - 支援多值：runtime、build、test、development、optional、provided
  - 範例：`--scope-filter runtime --scope-filter optional`
- **`--production`** — 正式環境模式（僅執行階段 + 選用）
  - 大幅縮減 SBOM 體積（例如，典型專案從 67 個套件縮減至 11 個）
  - 等價於 `--scope-filter runtime --scope-filter optional`

**輸出與報告增強：**
- **範圍統計**，顯示於終端機及 Markdown 報告
  - 各範圍計數及百分比
  - 平均分類信心分數
  - 相依性總數
- **相依性建立的建構者模式**
  - `Dependency::new().with_scope(scope, confidence, reason)`
  - 更簡潔的測試程式碼與更好的 API 使用體驗

**分類功能：**
- **生態系統感知分類**：
  - SYSTEM 函式庫 → 執行階段（信心分數 0.8+）
  - BUILD-CONFIG → 建置（待連結分析優化）
  - GIT-SUBMODULE → 已提供
  - MESON-WRAP/SUBPROJECT → 建置
  - PIP/npm/cargo → 視上下文而定
- **基於名稱的啟發式規則**：
  - 已知工具的精確比對（信心分數 1.0）
  - 大小寫不敏感比對
  - 基於模式的分類
- **詳盡推論說明**：所有分類均附詳細解釋

**依類別的測試涵蓋範圍：**
- **範圍過濾**（14 項測試）：
  - 預設行為（無過濾）
  - 正式環境模式過濾
  - 自訂單範圍/多範圍過濾
  - 邊界情況（無效過濾器、空結果）
- **端對端工作流程**（11 項測試）：
  - 完整分類流水線
  - 正式環境 SBOM 產生
  - CycloneDX 輸出整合
  - SBOM 體積縮減驗證
  - 生態系統多樣性驗證
- **真實情境分類**（17 項測試）：
  - 常用函式庫偵測準確率
  - 建置工具識別
  - 測試框架識別
  - 開發工具分類
  - 信心分數分佈
  - 分類推論驗證

### 變更
- **Dependency 結構體**新增範圍欄位
  - `scope: DependencyScope` — 分類結果
  - `scope_confidence: f32` — 信心分數（0.0-1.0）
  - `scope_reason: String` — 人類可讀的說明
- **主流水線**現包含自動分類（步驟 3.5/5）
- **SBOM 結構體**包含 `scope_statistics: Option<ScopeStatistics>`
- **終端機輸出**在摘要中顯示範圍分佈

### 第 4 階段驗證結果
- **測試通過率**：100%（609/609 項測試通過）
- **分類準確率**：
  - 建置工具：100% 準確（cmake、gcc、ninja、meson）
  - 測試框架：100% 準確（pytest、jest、junit、mocha）
  - 開發工具：100% 準確（pylint、black、eslint、prettier）
  - 執行階段函式庫：SYSTEM 生態系統高準確率
  - BUILD-CONFIG：預設歸為建置（待連結分析優化）
- **信心分數分佈**：
  - 精確名稱比對：0.95-1.0（pytest、cmake 等）
  - 基於生態系統：0.7-0.9（SYSTEM、GIT-SUBMODULE）
  - 基於啟發式：0.5-0.8（退回情況）
- **生態系統涵蓋範圍**：已驗證 10 種以上生態系統
- **正式環境 SBOM 體積縮減**：典型情況縮減 50-80%（如從 67 個縮減至 11 個套件）

## [1.0.5] - 2026-03-02

### 新增 - GitHub Actions 與 CI/CD 基礎建設

**多平台建置系統：**
- **GitHub Actions 工作流程**，用於自動化跨平台建置與發布
  - macOS（ARM64 與 Intel x86_64），透過 osxcross 交叉編譯
  - Linux（x86_64 glibc）
  - Windows（x86_64），透過 MinGW 交叉編譯
  - 帶快取的平行建置，加速 CI/CD 流水線
  - 自動化發布，包含二進位資產與總和檢查碼

**發布自動化：**
- **自動擷取發布說明**，來源為 CHANGELOG.md
- **版本一致性驗證**，檢查 git 標籤與 Cargo.toml 是否相符
- **SHA256 總和檢查碼**產生，適用於所有發布二進位檔案
- **PDF 產生**，使用 xnexus-md2pdf-tool 產生二進位發行指南
- 帶封面頁與專業排版的 VicOne 風格 README.pdf

**建置改進：**
- 自代管執行器支援，整合企業級 GitHub
- 用於私有子模組存取的 GIT_TOKEN 驗證
- 成品保留（90 天），用於建置可追溯性
- 平台專屬建置過濾（僅建置所需平台）
- 可設定的正式版/預發布版旗標

### 變更 - 程式碼品質與死碼清理

**API 簡化（約刪除 155 行）：**
- **移除所有格式函式的 `_with_mode` 後綴**：
  - `print_spdx_json_with_mode` → `print_spdx_json`
  - `print_spdx_tag_value_with_mode` → `print_spdx_tag_value`
  - `save_spdx_json_with_mode` → `save_spdx_json`
  - `save_spdx_tag_value_with_mode` → `save_spdx_tag_value`
  - `convert_to_spdx_with_mode` → `convert_to_spdx`
  - `convert_to_cyclonedx_with_mode` → `convert_to_cyclonedx`
  - `print_cyclonedx_json_with_mode` → `print_cyclonedx_json`
  - `save_cyclonedx_json_with_mode` → `save_cyclonedx_json`

**消除編譯器警告：**
- 移除格式模組中未使用的包裝函式
- 清理所有解析器模組中未使用的匯入
- 移除解析器函式中的死碼
- 修正測試參照，採用新的簡化 API

**文件：**
- 更新 BINARY_README.md，新增完整的 MIT 授權條款文字（VicOne Inc. 版權）
- 為面向客戶的文件移除內部發行區段
- 將 LICENSE 中的版權持有人從「William Chang」更新為「VicOne Inc.」

### 遷移說明

**僅影響函式庫呼叫方**（二進位使用者不受影響）：

格式 API 已簡化，函式名稱更清晰：

```rust
// 舊 API（v1.0.5 之前）
save_spdx_json_with_mode(sbom, path, &SbomMode::Complete, false)
convert_to_cyclonedx_with_mode(sbom, &SbomMode::Complete)

// 新 API（v1.0.5+）
save_spdx_json(sbom, path, &SbomMode::Complete, false)
convert_to_cyclonedx(sbom, &SbomMode::Complete)
```

### 基礎建設

**發布資產：**
- macOS（ARM64/Intel）、Linux（x86_64 glibc）、Windows（x86_64）預建置二進位檔案
- README.md（Markdown 二進位發行指南）
- README.pdf（VicOne 風格 PDF 指南）— 新增！
- checksums.txt（SHA256 驗證）

所有二進位檔案均透過 GitHub Actions 建置，並經過自動化測試與驗證。

## [1.0.4] - 2026-02-25

### 新增 - Meson 與 Bazel 建置系統

**現代 C/C++ 建置系統支援：**
- **Meson 建置系統解析器** — 現代元建置系統支援
  - 解析 `meson.build` 檔案中的 dependency() 和 subproject() 宣告
  - 支援 wrap 檔案（`*.wrap`）用於外部相依性管理
  - 處理版本限制與建置選項
  - 與 Conan 鎖定檔整合（已在 conanfile.lock 中偵測到 meson 1.2.2）
  - 額外涵蓋約 2.5% 的 C/C++ 專案

- **Bazel 建置系統解析器** — Google 建置系統支援
  - 解析 `BUILD`、`BUILD.bazel` 與 `WORKSPACE` 檔案
  - 支援 http_archive、git_repository 與 maven_jar 規則
  - 從 URL 和 commit SHA 擷取版本
  - 處理外部儲存庫參照（@repo//:target）
  - 額外涵蓋約 2.5% 的 C/C++ 專案

**CLI 增強：**
- `--scan-meson` 旗標，啟用/停用 Meson 掃描（預設：true）
- `--scan-bazel` 旗標，啟用/停用 Bazel 掃描（預設：true）
- 所有 C/C++ 生態系統旗標的完整說明文字

### 新增 - 全面的比較報告

**競品分析：**
- **6 份完整比較報告**，共 2,897 行
  - OpenStudio、UD Trucks 正式環境、VDL Bus 正式環境
  - ROS 2 Humble Desktop、scikit-learn、Python 測試夾具
- **2,561 個相依性**橫跨所有專案
- **比競品多偵測 2.1%〜58.8% 的套件**
- **4 項競品商業工具所不具備的獨特能力**：
  - Autotools/pkg-config 支援（舊版 C 專案）
  - ROS 2 package.xml 解析
  - Git 子模組遞迴掃描
  - CMake FetchContent/ExternalProject

**成本節省分析：**
- 相比 BlackDuck 授權費用節省 **$220K-$1.65M**
- 單次掃描費用：$0（radeis_sc2sbom）vs $50-$200（SaaS 競品）
- 比較文件總計：7 份報告 + 索引

### 變更

**涵蓋率影響：**
- v1.0.4 之前：約 90% 的 C/C++ 專案
- v1.0.4 之後：**約 95% 的 C/C++ 專案**

**Bug 修正：**
- 修正 Bazel 解析器在 BUILD 檔案運算式中的括號比對問題
- 將系統套件 purl 類型更正為 `pkg:generic/` 格式
- 更新掃描器測試以適配新的簽章變更

**文件：**
- 更新 README.md，新增 Meson 和 Bazel 生態系統支援
- 完善 BENCHMARKS.md，新增詳細的比較方法論
- 在 WHATS_NEW.md 中新增 v1.0.4 發布說明

### 基礎建設

**測試：**
- 105 項單元測試通過（新增 8 項 Meson/Bazel 測試）
- 使用真實 BUILD 和 meson.build 檔案進行整合測試
- 向下相容：與 v1.0.0-1.0.3 100% 相容

## [1.0.3] - 2026-02-24

### 新增 - C 舊版建置系統支援

**傳統 C/C++ 建置系統解析器：**
- **pkg-config 解析器** — 系統函式庫相依性偵測
  - 解析 `.pc` 檔案（pkg-config 中繼資料檔案）
  - 從 configure.ac 擷取 `PKG_CHECK_MODULES` 宣告
  - 版本與相依性鏈解析
  - 處理 Requires/Requires.private 欄位
  - 系統函式庫相依性約 80% 的涵蓋率

- **Autotools 解析器** — GNU 建置系統支援
  - 解析 `configure.ac` 檔案
    - `AC_CHECK_LIB(library, function)` 宣告
    - `AC_SEARCH_LIBS(function, [libs...])` 宣告
    - `PKG_CHECK_MODULES(PREFIX, packages)` 巨集
  - 解析 `Makefile.am` 檔案
    - `LDADD` 與 `LIBADD` 連結器旗標擷取
    - `-l` 旗標解析用於函式庫相依性
  - 純 C 專案約 60% 的涵蓋率

- **Makefile 啟發式解析器** — 純 Makefile 支援
  - 從 LDFLAGS/LIBS 中基於模式擷取 `-l` 旗標
  - 偵測帶函式庫名稱的 `pkg-config --libs` 呼叫
  - 盡力解析，無需完整 Make 求值
  - 舊版 C++ 專案約 40% 的涵蓋率

**CLI 增強：**
- `--scan-pkgconfig` 旗標，啟用/停用 pkg-config 掃描（預設：true）
- `--scan-autotools` 旗標，啟用/停用 Autotools 掃描（預設：true）
- `--scan-makefiles` 旗標，啟用/停用 Makefile 掃描（預設：true）
- `--resolve-system-deps` 旗標，嘗試解析系統函式庫版本（預設：true）

### 變更

**涵蓋率影響：**
- 結合 v1.0.0-1.0.2：**約 90% 的所有 C/C++ 專案**
- 成功掃描舊版專案（curl、nginx、openssl 模式）
- 填補無現代建置系統專案的缺口

**SPDX purl 支援：**
- 新增 `pkg:generic/{name}@{version}?type=pkg-config` 格式
- 新增 `pkg:generic/{name}@{version}?type=autotools` 格式
- 新增 `pkg:generic/{name}@{version}?type=makefile` 格式

**文件：**
- 更新 README.md，新增 C 舊版建置系統支援
- 在 WHATS_NEW.md 中新增 v1.0.3 詳細發布說明
- 記錄啟發式解析的限制與最佳實踐

### 基礎建設

**測試：**
- 所有 5 個 C 解析器的單元測試，使用 tempfile 夾具
- 使用真實 configure.ac、Makefile.am、Makefile 樣本進行整合測試
- 測試夾具包含 openssl.pc、curl 風格的 configure.ac 模式

## [1.0.2] - 2026-02-24

### 新增 - Conan 套件管理員支援

**Conan C++ 套件管理員：**
- **Conan 鎖定檔解析器**（`conan.lock`）
  - 解析 Conan v1 鎖定檔格式（JSON）
  - 擷取帶完整版本和修訂資訊的套件參照
  - 支援直接相依性與傳遞相依性圖
  - 處理遠端儲存庫中繼資料
- **Conan 清單解析器**（`conanfile.txt`、`conanfile.py`）
  - 鎖定檔不可用時的退回方案
  - 版本限制解析（>=、~、==等）
  - 選項與產生器偵測

**SPDX purl 支援：**
- 新增 `pkg:conan/{name}@{version}` 格式
- 可用時包含修訂與遠端中繼資料

**CLI 增強：**
- 自動偵測 `conan.lock`、`conanfile.txt`、`conanfile.py`
- 整合至現有 C/C++ 掃描工作流程

### 變更

**掃描器改進：**
- 透過深度驗證優化子模組掃描
- 提升 CMake 解析穩健性（*.cmake 模組檔案）
- 修正遞迴掃描中多餘的深度檢查
- 改善對格式錯誤 Conan 檔案的錯誤處理

**文件：**
- 重組 README.md，改善結構
- 在 `docs/` 目錄建立完整文件
- 更新 WHATS_NEW.md，新增 v1.0.2 發布說明
- 在支援的生態系統表格中新增 Conan

### 基礎建設

**測試：**
- 使用真實鎖定檔樣本進行 Conan 解析器單元測試
- conanfile.txt 和 conanfile.py 的整合測試
- 全部測試通過（共 124 項）

**CLI：**
- 新增 `--output` 旗標，用於自訂 SBOM 輸出目錄
- 改善說明文字與範例

## [1.0.1] - 2026-02-23

### 新增 - CMake 支援與遞迴子模組掃描

**CMake 相依性解析器：**
- **FetchContent 解析器** — 現代 CMake 外部相依性
  - 從 CMakeLists.txt 解析 `FetchContent_Declare()` 區塊
  - 支援 GIT_REPOSITORY、GIT_TAG、URL、URL_HASH
  - 從 URL_HASH 擷取 SHA256 總和檢查碼，用於供應鏈安全
  - 採用 Git URL 解析器產生正確的 github/gitlab/bitbucket purl
  - 從 GIT_TAG 或 URL 路徑擷取版本
- **ExternalProject 解析器** — 舊版 CMake 外部專案支援
  - 解析 `ExternalProject_Add()` 區塊
  - 處理與 FetchContent 相同的 Git 與 URL 模式
  - 靜態解析（無需執行 CMake）

**遞迴子模組掃描：**
- 遞迴掃描 Git 子模組內的相依性
  - package.json、Cargo.toml、CMakeLists.txt 及所有支援的清單檔案
  - 深度限制，防止無限遞迴
  - 可設定最大深度（預設：3）
- 掃描器模組中新增 `scan_submodule_recursively()` 函式

**CLI 增強：**
- `--scan-cmake` 旗標，啟用/停用 CMake 相依性掃描（預設：true）

### 變更

**掃描器簽章：**
- 更新 `scan_directory()`，新增 `scan_cmake` 參數
- 所有呼叫點已更新以傳遞新參數
- 測試已更新以適配新掃描器簽章

**SPDX purl 支援：**
- 新增 `pkg:cmake/{name}@{version}` 格式
- 非 Git 來源退回至 `pkg:generic/{name}@{version}`

**文件：**
- 更新 README.md，新增 CMake 支援與新 CLI 旗標
- 在 WHATS_NEW.md 中新增 v1.0.1 詳細發布說明
- 記錄 CMake 變數處理限制（`${VAR}` 將被略過）

### 基礎建設

**測試：**
- 建立 `tests/parser_tests/cmake_tests.rs`，包含 8 項全面測試
  - FetchContent 解析（Git 和 URL 來源）
  - ExternalProject 解析
  - CMake 變數處理（對無法解析的變數發出警告並略過）
  - 總和檢查碼擷取驗證
- 在 `tests/fixtures/cmake/` 中建立測試夾具
  - CMakeLists_fetchcontent.txt
  - CMakeLists_externalproject.txt
  - CMakeLists_with_variables.txt
- 全部 124 項測試通過（原有 116 項 + 8 項新 CMake 測試）

## [1.0.0] - 2026-02-20

### 新增 - C++ 生態系統支援

**首個 C++ 生態系統支援：**
此主要版本（1.0.0）新增了完整的 C/C++ 專案 SBOM 產生能力，採用 Rust 開發。

**vcpkg 套件管理員：**
- **vcpkg 清單解析器**（`vcpkg.json`）
  - 所有版本限制格式：`version>=`、`version>`、`version=`、`version-semver`、`version-date`
  - 用於版本固定的覆蓋區段
  - 特性中繼資料儲存於 source_file 欄位
  - 產生 `pkg:vcpkg/{name}@{version}` purl 格式
- 已整合至掃描器，支援自動偵測 vcpkg.json

**Git 子模組偵測：**
- **Git 子模組解析器**（`.gitmodules`）
  - 解析 INI 格式的子模組定義
  - 透過 `git ls-tree HEAD` 解析 commit SHA
  - 支援 HTTPS 與 SSH URL 格式
  - 多主機支援：GitHub、GitLab、Bitbucket、自代管 Git 伺服器
  - 依主機類型產生相應 purl 格式：
    - GitHub：`pkg:github/owner/repo@sha`
    - GitLab：`pkg:gitlab/owner/repo@sha`
    - Bitbucket：`pkg:bitbucket/owner/repo@sha`
    - 自代管：`pkg:generic/repo@sha`

**CLI 增強：**
- `--scan-submodules` 旗標，啟用/停用子模組掃描（預設：true）
- `--submodule-depth` 旗標，設定子模組的最大遞迴深度（預設：3）

### 新模組

**解析器模組：**
- `src/parsers/cpp/mod.rs` — C++ 解析器模組進入點
- `src/parsers/cpp/vcpkg.rs` — vcpkg 清單解析器（458 行）
- `src/parsers/git/mod.rs` — Git 解析器模組進入點
- `src/parsers/git/submodules.rs` — .gitmodules 解析器（292 行）
- `src/parsers/git/commit_resolver.rs` — Git commit SHA 解析器（140 行）
- `src/parsers/git/url_parser.rs` — 支援多主機的 Git URL 解析器（299 行）

**新增程式碼總計：** 1,511 行（19 個檔案變更）

### 變更

**掃描器整合：**
- 在 `scan_directory()` 中新增 vcpkg.json 和 .gitmodules 偵測
- 整合 commit SHA 解析，用於精確的子模組版本
- 對無法解析的 Git 參照發出警告

**SPDX 格式：**
- 為 vcpkg 套件新增 purl 產生
- 為 Git 子模組新增帶主機專屬格式的 purl 產生

**文件：**
- 更新 README.md，新增 C++ 生態系統支援表格
- 在 WHATS_NEW.md 中新增 v1.0.0 詳細發布說明
- 記錄 vcpkg 版本限制格式
- 記錄 Git URL 解析與多主機支援

### 基礎建設

**測試：**
- 帶版本限制驗證的 vcpkg 解析器測試
- 帶 URL 解析的 Git 子模組解析器測試
- C++ 專案整合測試
- 全部測試通過

**Bug 修正（1.0.0 預發布完善）：**
- 修正 Python 版本運算子剝離（>=、==、~=）
- 為 Python 新增傳遞相依性解析
- 修正 Python 解析器中 `__future__` 誤判
- 改善 Git 操作中的錯誤處理

## [0.9.3] - 2026-02-10

### 新增 - Pipfile/Pipfile.lock 與 pyproject.toml 解析器支援

**Pipfile/Pipfile.lock 支援（第 1 階段）：**
- **Pipfile.lock 解析器** — 為 Pipenv 專案提供完整的鎖定檔支援
  - 使用 serde_json 反序列化解析 JSON 格式
  - 從 `default`（正式環境）和 `develop`（開發）區段擷取所有套件
  - **SHA256 總和檢查碼擷取**，來自 hashes 陣列，用於供應鏈安全
  - 透過 `index` 欄位的存在來偵測直接相依性
  - 使用 rayon 平行批次從 PyPI 擷取中繼資料
  - **提升 487%**：從 8 個套件（v0.9.1）提升至 47 個（v0.9.3）
  - 基於 Pipenv 專案與 Black Duck **100% 對等**

- **Pipfile 清單解析器** — 鎖定檔不可用時的退回方案
  - 使用現有 toml crate 解析 TOML 格式
  - 處理版本規格：`*`、`==`、`>=`、`~=`、複雜限制
  - 區分 `[packages]`（正式環境）和 `[dev-packages]`（開發）

**pyproject.toml 支援（第 2 階段）：**
- **多格式 pyproject.toml 解析器** — 現代 Python 套件標準（PEP 517/518）
  - **PEP 621 格式** — `[project]` 區段，含 `dependencies` 和 `optional-dependencies`
  - **Poetry 格式** — `[tool.poetry]` 區段，含 `dependencies` 和 `dev-dependencies`
  - **Poetry 1.2+ 群組** — `[tool.poetry.group.*.dependencies]` 格式
  - **PDM 格式** — `[tool.pdm]` 區段，含 `dependencies` 和 `dev-dependencies`
  - 使用正規表示式解析複雜的相依性規格
  - 從選用相依性群組（dev、test、tests、testing）偵測開發相依性
  - 處理 Poetry 版本限制：插入符（`^`）、波浪符（`~`）、比較運算子
  - 自動過濾 Python 版本限制

**poetry.lock 總和檢查碼擷取（第 3 階段）：**
- **從 `[[package.files]]` 區段擷取 SHA256 總和檢查碼**
  - 從 files 陣列中擷取第一個檔案雜湊值
  - 格式：`sha256:abc123...` → `abc123...`
  - 增強 Poetry 專案的供應鏈安全
  - 零額外網路負擔

### 變更
- assessment-service 儲存庫的 Python 套件偵測從 8 個提升至 47 個
- 消除 Pipenv 專案所有 `@detected` 版本佔位符
- 套件名稱使用規範 PyPI 格式（例如 `repoze.lru` 而非 `repoze-lru`）
- 從 poetry.lock 和 Pipfile.lock 擷取 SHA256 總和檢查碼（儲存於 Dependency 結構體，供未來 SPDX 輸出整合使用）
- 整合載入動畫："parsing Pipfile.lock..." 訊息在掃描期間顯示
- 進度指示器自動顯示 Python 套件計數

### 測試
- 已驗證從 assessment-service Pipfile.lock 偵測到 47 個 Python 套件
- 100% 版本準確率（無 "@detected" 佔位符殘留）
- 與 Black Duck 的套件名稱完全相符（47/47 個套件）
- 已驗證 Pipfile.lock 和 poetry.lock 的總和檢查碼擷取

### 效能
- 使用 rayon 平行批次擷取 PyPI 中繼資料（沿用現有模式）
- 使用 serde 反序列化進行單遍 JSON/TOML 解析
- 相比 v0.9.1 無效能下降
- 與 v0.9.2 進度指示器無縫整合

### 技術細節
- **修改的檔案：**
  - `src/parsers/python.rs` — 新增 3 個解析器函式（+219 行）
    - `parse_pipfile_lock_with_relationships()` — 帶總和檢查碼的 Pipfile.lock
    - `parse_pipfile()` — Pipfile 清單
    - `parse_pyproject_toml()` — pyproject.toml 多格式
  - `src/parsers/mod.rs` — 匯出新的解析器函式
  - `src/scanner/mod.rs` — 註冊解析器並整合載入動畫（+8 行）
  - `Cargo.toml` — 版本升級至 0.9.3

- **相依性：**
  - 無新增相依性（使用現有的 serde_json、toml、regex、rayon）

- **解析器優先級順序：**
  1. Pipfile.lock（最高 — 帶總和檢查碼的精確版本）
  2. poetry.lock（高 — 帶總和檢查碼的精確版本）
  3. requirements.txt（中 — 版本規格）
  4. setup.py（中 — 版本規格）
  5. Pipfile（中 — 版本規格）
  6. pyproject.toml（中 — 版本規格）
  7. 匯入掃描（最低 — 無版本）

### 競爭定位
- **與 Black Duck 對等**，Python 套件偵測（47/47 個套件）
- **更優的套件命名** — 使用規範 PyPI 名稱
- **全面的 Python 支援** — Pipfile、Poetry、pip、setuptools、PDM、PEP 621
- **供應鏈安全** — 來自鎖定檔的 SHA256 總和檢查碼
- **現代標準** — 完整的 pyproject.toml 支援

### 遷移說明
- **函式庫使用者的破壞性變更**：`ScanContext.poetry_relationships` 更名為 `python_lockfile_relationships`（CLI 用法不受影響）
- 現有 Pipenv 專案自動受益於改進的偵測
- Poetry 專案現在在 SBOM 輸出中包含總和檢查碼
- pyproject.toml 專案現在可產生準確的 SBOM

## [0.9.1.1] - 2026-01-28（緊急修正）

### 修正 - Markdown 報告顯示 Bug

**問題：**
- ROS 多套件 Markdown 報告顯示空的或不完整的程式碼區塊
- 範例："PIP（1 個套件）"標題下程式碼區塊為空，"ROS（8 個套件）"只顯示 2 個套件
- 由於套件計數與顯示的套件不相符，導致讀者困惑

**根本原因：**
- 終端機報告產生使用 `render_dependency_list()`，該函式僅顯示**正式環境**直接相依性
- 標題統計了所有套件（包含開發相依性）
- 這導致開發相依性從顯示中被過濾掉，但仍計入標題數量

**解決方案：**
- 修改 ROS 多套件區段，包含所有直接相依性（正式環境 + 開發）
- 使用擴展過濾的 `render_tree_classic()` 顯示所有直接相依性
- 保持帶有正確分支字元（`├──`、`└──`）的樹狀結構
- 現在所有計入的套件均在程式碼區塊中以樹狀視覺化顯示

**影響：**
- 修正 ROS 多套件報告中的空程式碼區塊
- 顯示所有直接相依性（正式環境和開發），採用正確的樹狀結構
- 保持標題與顯示套件之間計數一致
- 不影響一般（非 ROS）專案報告

**測試結果：**
- ros2run PIP 區段現在顯示：`└── pytest @ unspecified [direct, dev]`
- ros2run ROS 區段現在顯示全部 8 個套件的樹狀結構（├──、└──）
- 修正前：空 PIP 區塊，不完整的 ROS 清單（僅顯示 8 個中的 2 個）

**修改的檔案：**
- `src/formats/console.rs`（第 1292-1324 行）

## [0.9.1] - 2026-01-28

### 新增 - ROS/rosdistro 版本解析與儲存庫 URL 豐富化

**ROS 套件版本解析：**
- **自動 ROS 發行版偵測** — 與 rosdistro GitHub API 整合，解析 ROS 套件版本
  - 從 ros/rosdistro 儲存庫擷取 distribution.yaml
  - 支援 ROS 2 發行版：jazzy、iron、humble、galactic、foxy
  - 支援 ROS 1 發行版：noetic、melodic
- **手動覆蓋的 CLI 旗標** — `--ros-distro <distro>`，用於明確指定 ROS 發行版
  - 優先級順序：CLI 旗標 > ROS_DISTRO 環境變數 > 預設值（"jazzy"）
  - 範例：`--ros-distro humble`、`--ros-distro iron`
- **套件名稱變體解析** — 處理多種命名慣例
  - 基礎名稱：`rclpy`
  - Python 前綴：`python3-rclpy`
  - 發行版前綴：`ros-jazzy-rclpy`
  - 底線變體：`ament-index-python`、`python3-ament-index-python`
- **全域快取** — 每次掃描工作階段每個發行版僅擷取一次 rosdistro
  - 10 秒逾時，優雅退回至"unspecified"
  - 使用 rayon 平行解析（與現有中繼資料豐富化模式一致）

**儲存庫 URL 豐富化：**
- **GitHub URL 擷取** — 為 ROS 套件填入 SPDX `downloadLocation` 欄位
  - 從 rosdistro distribution.yaml 擷取 `source.url`
  - 47 個套件帶有 GitHub 儲存庫 URL（ros2cli 專案基準）
  - 完整的原始碼可追溯性，用於安全稽核
  - 零效能負擔（使用現有 rosdistro 擷取）

### 變更
- ROS 相依性現在顯示解析後的版本，而非"unspecified"
  - 修正前：`rclpy @ unspecified, downloadLocation: NOASSERTION`
  - 修正後：`rclpy @ 7.1.9, downloadLocation: https://github.com/ros2/rclpy.git`（jazzy）
- SPDX `downloadLocation` 欄位已為 ROS 套件填入（47 個套件帶 URL）
- 更新 `scan_directory()` 簽章，接受選用的 `ros_distro` 參數
- 完善 `detect_ros_distribution()`，採用三層優先級系統
- 在 `RosPackageInfo` 結構體中新增 `repository_url` 欄位
- 將 `lookup_package_version()` 更名為 `lookup_package_info()`（回傳版本 + URL）

### 測試
- 新增儲存庫 URL 豐富化單元測試（`test_resolve_ros_dependency_versions_with_repository_url`）
- 新增 5 項 rosdistro 函式單元測試（版本解析、套件變體、非 ROS 套件）
- 新增 2 項 ros2cli 掃描整合測試，使用不同發行版
- 全部 97 項測試通過

### 效能
- 每個 ROS 發行版單次網路擷取（工作階段期間快取）
- 10 秒逾時，支援優雅降級
- 不影響非 ROS 專案的效能
- 使用 rayon 平行解析
- 儲存庫 URL 擷取無額外網路負擔

### 技術細節
- 新增相依性：`serde_yaml` v0.9、`lazy_static` v1.4
- 修改的檔案：`src/cli.rs`、`src/parsers/ros.rs`、`src/scanner/mod.rs`、`src/main.rs`、`src/formats/spdx.rs`
- 新函式：`fetch_rosdistro_database()`、`detect_ros_distribution()`、`lookup_package_info()`、`resolve_ros_dependency_versions()`
- 在 `RosPackageInfo` 結構體中新增 `repository_url: Option<String>` 欄位
- 更新 SPDX 格式化器中的 `create_download_location()`，使用 `dependency.repository_url`

### 競爭定位
- **ROS 支援**：首個透過 rosdistro 實現自動化 ROS 套件版本解析的 SBOM 工具
- **儲存庫 URL**：首個為 ROS 套件填入 downloadLocation 的 SBOM 工具
- **對比 BlackDuck**：radeis 偵測到 94 個唯一相依性（是 BlackDuck 4 個的 23.5 倍）
  - 15 個獨立 ROS 套件 vs 3 個儲存庫（粒度高 5 倍）
  - 47 個套件帶 GitHub URL vs BlackDuck 的 0 個
  - 62 個套件帶已解析版本 vs BlackDuck 的 3 個（多 21 倍）

### 基準測試結果（ros2cli 專案）
- **94 個唯一相依性**
- **15 個 ros2cli 儲存庫內的獨立 ROS 套件**
- **47 個套件帶 GitHub 儲存庫 URL**
- **62 個套件帶已解析版本**（涵蓋率 66%）
- **223 條 SPDX 階層式條目**（含關聯關係）
- **偵測到 5 個漏洞性套件**並嵌入 SPDX 輸出

## [0.9.0] - 2026-01-28

### 新增 - 總和檢查碼、自動化與多生態系統中繼資料

**套件總和檢查碼：**
- **所有套件的 SHA-1 總和檢查碼** — 支援完整性驗證與可重現建置
  - 格式：40 字元小寫十六進位 SHA-1 雜湊值
  - 新增至每個套件的 SPDX `filesAnalyzed` 欄位
  - 支援供應鏈安全與 SBOM 驗證工作流程

**自動化修正建議：**
- Markdown 輸出中的機器可讀修正建議
- 用於自動漏洞修復的結構化格式
- 與現有漏洞報告整合

**多生態系統中繼資料擷取（網路模式）：**
- **所有生態系統的混合中繼資料擷取** — 優先使用本機檔案，退回至登錄 API
  - npm：package.json + npm 登錄 API（registry.npmjs.org）
  - Python：用於 poetry.lock 套件的 PyPI API（pypi.org/pypi）
  - Cargo：用於 Cargo.lock 套件的 crates.io API
  - PHP：用於 composer.json 套件的 Packagist API（repo.packagist.org/p2）
  - Ruby：用於 Gemfile 套件的 RubyGems API（rubygems.org/api/v2）
- **使用 rayon 平行批次擷取**，效能提升 10-27 倍
  - npm：689 個套件，從 10 分鐘以上 → 22.6 秒（加速 27 倍）
  - Python：100-500 個套件，從 10-20 分鐘 → 30 秒（加速 20-40 倍）
  - Cargo：100-500 個套件，從 7-15 分鐘 → 25 秒（加速 17-36 倍）
  - PHP：10-200 個套件，從 2-7 分鐘 → 20 秒（加速 6-21 倍）
  - Ruby：5-50 個 gem，從 1-2 分鐘 → 10 秒（加速 6-12 倍）
- **API 逾時從 5 秒縮短至 3 秒**，加速失敗處理

### 變更
- 檔案大小從 955KB（v0.8.0）優化至 899KB（縮減 6%）
- 效率從 1.384 KB/套件提升至 1.303 KB/套件（改善 6%）
- 保留 v0.8.0 的全部 690 個套件和 689 個 CPE 識別碼
- 所有解析器函式現在使用 3 遍模式：收集 → 平行擷取 → 建立相依性

### 效能
- **套件數量**：690 個（與 v0.8.0 一致）
- **檔案大小**：899 KB（比 v0.8.0 的 955 KB 小 6%）
- **CPE 識別碼**：689 個（與 v0.8.0 一致）
- **檔案效率**：1.303 KB/套件（優於 v0.8.0 的 1.384 KB/套件）

### 競爭定位
- **獨特功能**：唯一在 SPDX 輸出中同時嵌入漏洞 + CPE 識別碼 + SHA-1 總和檢查碼的工具
- **相比 v0.8.0 的改進**：檔案大小縮減 6%，同時保留所有功能

## [0.8.0] - 2026-01-27

### 新增 - 豐富中繼資料與安全功能

**中繼資料擷取（各生態系統涵蓋率 95% 以上）：**
- **授權資訊擷取** — 為 npm、Cargo、Python、ROS、PHP、Ruby 生態系統擷取並規範化授權識別碼（符合 SPDX 規範）
  - 授權涵蓋率達 95% 以上（650+/690 個套件，而 v0.7.0 為 0%）
  - 將 SPDX 的"NOASSERTION"替換為實際授權識別碼
- **供應商與來源方追蹤** — 供應鏈透明度所需的作者、維護者與組織中繼資料（涵蓋率 90% 以上，620+/690 個套件）
  - 對應至帶"Person:"和"Organization:"前綴的 SPDX supplier/originator 欄位
- **下載位址 URL** — 各生態系統套件登錄 URL，用於驗證與可重現建置
  - npm：`https://registry.npmjs.org/{package}/-/{file}.tgz`
  - PyPI：`https://pypi.org/project/{package}/{version}/`
  - Cargo：`https://crates.io/api/v1/crates/{package}/{version}/download`
  - 完整支援 Composer、RubyGems 和 Go 生態系統

**增強 SPDX 輸出：**
- **基於 UUID 的 SPDX ID** — 比循序 ID 唯一性更強，無命名空間衝突
  - 格式：`SPDXRef-Package-{sanitized-name}-{uuid}`
  - 合成"main"根套件，確保文件結構一致
- **原始檔案追蹤** — 完整稽核軌跡，顯示哪個擷取器和清單檔案偵測到每個套件（涵蓋率 98% 以上，685+/690 個套件）
  - 模式："Identified by the {extractor_type} extractor from {absolute_path}"
- **CPE 識別碼** — 通用平台列舉（CPE 2.3），用於安全漏洞關聯
  - 格式：`cpe:2.3:a:vendor:product:version:*:*:*:*:*:*:*`
  - 生態系統專屬的供應商擷取（npm 範圍套件、Composer、Go 模組）

**測試基礎建設：**
- **模組化測試結構** — 將所有 64 項測試從單體 main.rs 遷移至有組織的測試模組
  - 7 個類別（解析器、格式、掃描器、模型、錯誤、工具程式、整合）共 84 項測試
  - main.rs 從 2,233 行減少至 268 行（縮減 88%，刪除 1,965 行）
  - 18 個獨立測試檔案，便於組織與維護
- **原始檔案追蹤測試** — 11 項新測試，涵蓋所有解析器（npm、Cargo、Python、ROS、PHP、Ruby、Go）
- **多生態系統整合測試** — 2 項全面測試，驗證不同解析器間的原始檔案追蹤
- **UUID 和 CPE 測試** — 7 項新測試，用於 SPDX ID 唯一性和 CPE 識別碼產生

### 變更
- 在 `Dependency` 結構體中新增選用中繼資料欄位（license、author、maintainers、repository_url、homepage_url、source_file）
- 更新 SPDX 套件建立，填入 license、supplier、originator 和 sourceInfo 欄位
- SPDX ID 產生從循序式（`SPDXRef-Package-npm-1`）改為基於 UUID（`SPDXRef-Package-axios-{uuid}`）
- 關聯關係結構從扁平式（v0.7.0 的 699 個 DESCRIBES）改為階層式（1 個 DESCRIBES + 689 個 CONTAINS）
- 所有解析器現在使用絕對路徑追蹤原始檔案路徑
- CycloneDX 格式現在包含授權和供應商資訊
- 建立 `src/lib.rs`，支援整合測試（將模組公開為函式庫）
- 公開 SPDX 結構體以便測試（SPDXDocument、SPDXPackage、SPDXRelationship、SPDXExternalRef 欄位）

### 改進
- 透過豐富中繼資料增強合規報告能力
- 更豐富的中繼資料，用於供應鏈安全與透明度
- 改善測試涵蓋率與組織，提升長期可維護性
- 增強去重邏輯，正確處理 ImportScan 與 Manifest 優先級

### 修正
- **去重 Bug 修正** — ImportScan 條目現在當 LockFile/Manifest 版本存在時能正確被過濾
  - v0.7.0 錯誤保留了帶佔位符"detected"版本的 ImportScan 重複項
  - 移除 10 個重複套件（9 個唯一套件被計算了兩次：axios、uuid、6 個 AWS SDK 用戶端、serverless-sentry-lib、strftime）
  - 套件計數從 699 更正為 690（689 個真實套件 + 1 個合成"main"）
- 去重邏輯現在正確優先排序：LockFile > Manifest > ImportScan
- 測試結構現在已正確組織，用於 Rust 整合測試
- SPDX 外部參照現在以正確格式同時包含 PURL 和 CPE 識別碼

### 效能
- **套件數量**：690 個（Bug 修正：移除 v0.7.0 的 10 個重複 ImportScan 條目）
- **檔案大小**：955 KB（退步：比 v0.7.0 的 454 KB 增加 110%）
- **CPE 識別碼**：689 個（v0.8.0 新功能）
- **檔案效率**：1.384 KB/套件（退步：比 v0.7.0 的 0.649 KB/套件 差 113%）
- 所有 84 項測試的執行時間在 90 秒以內

### 技術細節
- 修改 [src/models/dependency.rs](src/models/dependency.rs) — 新增選用中繼資料欄位
- 修改 [src/formats/spdx.rs](src/formats/spdx.rs) — 基於 UUID 的 ID、CPE 產生、階層式關聯關係、中繼資料填入
- 修改 [src/formats/cyclonedx.rs](src/formats/cyclonedx.rs) — 授權和供應商支援
- 修改 [src/parsers/mod.rs](src/parsers/mod.rs) — 帶 ImportScan 優先級修正的增強去重
- 建立 [src/lib.rs](src/lib.rs) — 用於整合測試的函式庫進入點
- 建立 [tests/all_tests.rs](tests/all_tests.rs) — 整合測試模組進入點
- 在 tests/ 目錄下建立 18 個有組織結構的測試模組檔案

### 競爭定位
- **中繼資料豐富度**：現已與企業級工具相當（95% 以上授權、90% 以上供應商，對比 BlackDuck 99.9%）
- **獨特功能**：唯一在 SPDX 輸出中同時嵌入漏洞 + CPE 識別碼的工具
- **多生態系統領先**：完整支援 npm、Cargo、Python、ROS、PHP、Ruby 生態系統的中繼資料
- **注意**：v0.8.0 修正了 ImportScan 去重 Bug（移除 10 個重複項）並新增了 CPE 中繼資料（增大了檔案大小），兩個問題均在 v0.9.0 中得到解決

## [0.7.0] - 2026-01-27

**⚠️ 已知問題**：本版本存在去重 Bug，錯誤保留了 10 個帶"detected"版本的 ImportScan 重複套件。詳見 [PACKAGE_COUNT_ANALYSIS.md](scan_reports/PACKAGE_COUNT_ANALYSIS.md)。已在 v0.8.0 中修正。

### 新增
- **套件偵測** — 現可偵測 **699 個套件**（原為 563 個），但包含 10 個重複 ImportScan 條目（v0.8.0 更正為 690 個）
- **雙 SBOM 模式**，適用於不同使用情境：
  - `--sbom-mode complete` — 所有套件（699 個，含 10 個重複項，454KB），用於合規與資產清單
- 智慧清單過濾 — 當精確的鎖定檔版本存在時，自動移除多餘的 package.json 版本範圍
- `docs/` 資料夾中的完整文件：
  - [docs/sbom_modes_guide.md](docs/sbom_modes_guide.md) — 雙 SBOM 模式完整指南，含 CI/CD 範例
  - [docs/WHATS_NEW.md](docs/WHATS_NEW.md) — v0.7.0 詳細變更與遷移指南
  - [docs/plan/improvement_plan.md](docs/plan/improvement_plan.md) — 技術設計文件
  - [docs/plan/implementation_summary.md](docs/plan/implementation_summary.md) — 含指標的實作結果

### 變更
- **套件偵測提升 24.2%**（563 → 699 個，但含 10 個 ImportScan 重複項）
- 去重演算法現在使用 `(name, version, ecosystem)` 元組代替 `(name, ecosystem)` 進行版本感知
- NPM 解析器使用 HashSet 防止對同一 `package@version` 組合的重複處理
- README.md 全面修訂，更加簡潔清晰，聚焦於關鍵改進
- 更新所有 SPDX 和 CycloneDX 產生器，支援基於模式的過濾

### 修正
- 同一套件的多個版本現在能正確保留（例如 `@aws-sdk/client-sso@3.632.0` 和 `@aws-sdk/client-sso@3.848.0` 均保留）
- 當鎖定檔版本存在時，清單版本（如版本範圍 `^3.215.0`）自動過濾
- 約 126 個缺失的 AWS SDK 子套件現在能正確在巢狀 node_modules 中偵測到

### 效能
- 僅漏洞模式實現檔案大小縮減 98%（11 KB vs 454 KB）
- ⚠️ **699 個套件含 10 個 ImportScan 重複項**（v0.8.0 更正為 690 個）

### 技術細節
- 修改 [src/parsers/mod.rs](src/parsers/mod.rs) — 帶清單過濾的版本感知去重
- 修改 [src/parsers/npm.rs](src/parsers/npm.rs) — 基於 HashSet 的重複預防
- 修改 [src/cli.rs](src/cli.rs) — 新增 SbomMode 列舉
- 修改 [src/formats/spdx.rs](src/formats/spdx.rs) — SPDX 輸出的基於模式的過濾
- 修改 [src/formats/cyclonedx.rs](src/formats/cyclonedx.rs) — CycloneDX 輸出的基於模式的過濾
- 修改 [src/main.rs](src/main.rs) — 向所有格式產生器傳遞模式參數

### 競爭定位
- 雙 SBOM 模式為合規和安全工作流程提供彈性
- **已知問題**：去重 Bug 允許帶"detected"版本的 ImportScan 條目在 LockFile 版本存在時存活（v0.8.0 已修正）

## [0.6.0] - 2026-01-26

### 新增
- 從鎖定檔為 npm、Cargo 和 Poetry 產生真實的階層式相依性樹狀結構
  - 第 1 階段：npm（package-lock.json）— 完整的父子關聯關係
  - 第 2 階段：Cargo（Cargo.lock）— 解析 Rust 專案的 dependencies 陣列
  - 第 3 階段：Poetry（poetry.lock）— 解析 Python 專案的 [package.dependencies] 表格
- 使用基於圖形的分析精確標記 [direct] 與傳遞相依性
- 重組報告結構，區分套件清單與附錄區段
- 循環相依性偵測與處理
- 完整相依性鏈追蹤，顯示從根到有漏洞性的套件的所有路徑
- 模組化架構重構 — 將 main.rs（6,561 行）拆分為 24 個檔案中的 7 個模組

### 變更
- 報告結構現在先顯示直接正式環境相依性，然後是獨立清單，開發/傳遞相依性放入附錄
- 附錄放在漏洞區段之後，便於更好地組織
- Main.rs 從 6,561 行減少至 2,130 行（縮減 67.6%）

### 修正
- 基於實際父子關聯關係修正 is_direct 旗標，而非檔案路徑
- 終端機摘要計數現在使用相依性圖中修正的旗標
- VendorMode::Only Bug，該 Bug 阻止掃描 vendor 目錄內的檔案
- 合併重複的 create_package_url 函式
- 跨報告區段標準化直接相依性計數
- 漏洞樹狀結構現在使用關聯關係確保一致的 is_direct 旗標

### 測試
- 全部 59 項綜合單元測試通過
- 新增 Cargo 和 Poetry 關聯關係解析測試
- 零編譯警告

## [0.5.0] - 2026-01-23

### 破壞性變更
- 樹狀視覺化現在預設啟用（使用 --tree-style flat 恢復舊格式）

### 新增
- 三種模式的樹狀相依性視覺化（flat、tree、compact）
- 以嚴重性優先組織的階層式漏洞顯示
- 一覽式概覽的摘要統計區段
- Emoji 嚴重性指示器（🔴 嚴重、🟠 高、🟡 中、🟢 低）
- 每個漏洞的相依性鏈顯示
- 使用 --max-vulns-per-severity 的可折疊漏洞顯示
- 摘要區段中的風險評估

### 變更
- 使用 Unicode 框線繪製字元增強終端機輸出
- 使用一致分隔符改善視覺階層

## [0.4.0] - 2026-01-22

### 破壞性變更
- 預設啟用漏洞檢查
- 預設啟用 vendor 目錄掃描
- 預設啟用匯入退回掃描

### 新增
- 增強 Markdown 報告，含相依性來源追蹤（直接 vs 傳遞）

### 變更
- 改善 npm 套件的傳遞相依性偵測
- 終端機報告中預設輸出詳細漏洞資訊

## [0.3.1] - 2026-01-21

### 新增
- 多平台建置系統（Windows + Linux）
- 交叉編譯自動化
- 增強建置文件

## [0.3.0] - 2026-01-20

### 新增
- ROS/ROS2 多套件支援
- 階層式樹狀結構輸出
- SPDX 關聯關係（DESCRIBES、DEPENDS_ON）
- Python、JS/TS、Go 的匯入掃描退回
- 51 項綜合單元測試

## [0.2.0] - 2026-01-19

### 新增
- SPDX 2.3 支援（JSON + Tag-Value）
- 套件 URL（purl）實作
- 多格式輸出

## [0.1.0] - 2026-01-16

### 新增
- 初始版本
- 支援 8 種生態系統（npm、Cargo、pip、Go、RubyGems、Composer、Maven、ROS）
- 終端機輸出
- 18 項單元測試

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
