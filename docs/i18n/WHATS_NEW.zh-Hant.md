# 新版本說明

## v1.0.18 — Tree-sitter AST 掃描器（2026-05-13）

### 概述


### Tree-sitter AST 掃描器


- **詞法回退** — 解析失敗（語法錯誤、前處理器結構）的檔案自動回退到詞法掃描器，確保沒有檔案被靜默略過。



|-----|------|


### `--experiment-scan`


| 模式 | 發現項數 | 誤判率 |
|------|---------|--------|


```bash

```

### Phase 24 調校

在發布前套用了 17 項目標性規則層級變更以減少誤判。主要變更：


淨結果：相較 v1.0.18 前基線減少 89,479 個誤判（217,279 → 127,800 筆總發現項）。

### 遷移說明

`--cppcheck-path` 不再被接受；請從任何建置腳本中移除。所有其他內部版本旗標保持不變。公開版本不受影響。

---

## v1.0.17 — 進階 C/C++ SAST 與 AUTOSAR 版本提取（2026-05-11）

### 概述

v1.0.17 在內部 SAST 掃描器中增加了引數值檢測和 cppcheck 子程序整合，新增適用於 CI/IDE 流水線的 SARIF 2.1 輸出，並修復了三個 AUTOSAR 準確性問題：完整的 arxml 相依性解析、從 `.epd` 檔案和 Doxygen 標頭提取元件版本，以及對連結器旗標發現的函式庫進行正確的生態系統分類。

### 引數值比對

詞法掃描器現在在呼叫位置檢查引數值，而不僅僅是函式名稱，以偵測無法從被呼叫函式名稱推斷的誤用：

|-----|------|------|

### cppcheck 整合

安裝 cppcheck 後，掃描器將其作為子程序呼叫，並將資料流支援的發現項與詞法結果合併：

- cppcheck 輸出從其 XML 報告格式解析
- 優雅降級：若 cppcheck 未在 `PATH` 或 `--cppcheck-path` 中找到，則使用僅詞法結果並在報告中注明
- 使用 `--cppcheck-path <PATH>` 指定非預設二進位檔案位置

### SARIF 2.1 輸出

每次內部版本掃描現在都會在 Markdown 報告旁寫出 `<project>_static_analysis.sarif` 檔案，與 GitHub Code Scanning、VS Code SARIF Viewer 及任何支援 SARIF 2.1 的 CI 流水線相容。使用 `--sarif-output <PATH>` 覆寫預設輸出路徑。

### SARIF 基準比對

`--sarif-baseline <PATH>` 旗標接受先前的 SARIF 執行結果並抑制其中已出現的發現項，僅回報**新**發現項，適用於 PR 級 CI 閘控。比對使用由規則 ID + 檔案 URI + 起始行計算的 SHA-256 指紋。

### AUTOSAR arxml 相依性解析

`.arxml` 檔案現已完整解析以提取元件間相依關係，提取三種 AUTOSAR 元素類型：

| 元素 | 相依類型 |
|------|---------|
| `SW-COMPONENT-PROTOTYPE` | SWC 間組合 |
| `BSW-MODULE-DESCRIPTION` | BSW 模組參考 |
| SWC 型別定義元素（`APPLICATION-SW-COMPONENT-TYPE` 等） | 元件型別宣告 |

### AUTOSAR 版本提取

三個來源現在組合使用，取代 AUTOSAR 元件的 `unspecified` 版本字串：

- **`.epd` 檔案（BSW 模組）**：標準 ECUC 模組定義檔案，包含 `ADMIN-DATA/DOC-REVISIONS` 下的 `ECUC-MODULE-DEF/REVISION-LABEL`
- **Doxygen 風格 C/H 標頭（SWC 目錄）**：比對 `^\s*\*\s+SW Version\s*:\s*(\S+)` 模式，依父目錄名稱分組
- **版本解析優先順序**：`.epd` → Doxygen 標頭 → `"unspecified"`
- **生態系統升級**：連結器旗標發現的 `system` 生態系統條目（如 `-lAdc`、`-lMcu`）在找到符合的 epd/Doxygen 版本後升級為 `autosar` 生態系統



---


### 概述



|--------|------|

### 輸出


### 回退模式

當未找到清單檔案派生的元件目錄但掃描根目錄下存在 C/C++ 原始檔案時，掃描器自動插入合成的 `(project_name, "C/C++") → scan_root` 條目，使無清單儲存庫也能被掃描。

### 內部功能開關

掃描器及相關格式化器僅在 `--features internal` 建置時編譯。`build-all.sh` 新增 `--internal` 旗標，同時產生公開版本和內部版本（內部版本檔案名稱新增 `-internal` 後綴）。

---


### 概述


### AUTOSAR 專案偵測

掃描器在主目錄遍歷前執行 `detect_autosar()` 預掃描，依序檢查三個信號（首次符合即短路）：

| 信號 | 觸發條件 |
|------|---------|
| DET-01 | 任意深度存在 `.arxml` 檔案 |
| DET-02 | 存在名為 `BSW`、`MCAL`、`RTE`、`AUTOSAR` 或 `SWC` 的目錄 |
| DET-03 | 根目錄或一級子目錄的 CMake 或 Makefile 中存在 `AUTOSAR_VERSION` 或 `AR_VERSION` 令牌 |

### AUTOSAR 輸出 — CycloneDX

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

### AUTOSAR 輸出 — SPDX

```
ExternalRef OTHER autosar:layer BSW-Memory
ExternalRef OTHER autosar:platform Classic
ExternalRef OTHER autosar:supplier Vector-Informatik
```

### 供應商對應（`--supplier-config`）

```yaml
NvM: "Vector Informatik"
CanIf: "ETAS"
```

已對應元件輸出供應商字串，未對應元件輸出 `NOASSERTION`。


**SPDX — `SECURITY` ExternalRef：**
```
```

**CycloneDX — `cwes` 陣列：**
```json
```

NVD 回應快取在 `~/.cache/sc2sbom/nvd/`（預設 TTL 24 小時）；每次 HTTP 請求限速 6 秒。

### 單一格式支援 `--output`

```bash
radeis_sc2sbom --path ./project --format spdx-json --output ./out
```

省略 `--output` 則輸出到標準輸出（保持原有行為）。

---

## v1.0.14 - 可靠性與 SBOM 品質（2026-04-24）

### 概述

v1.0.14 是一個由使用者回報的真實 C/C++ 專案掃描缺陷驅動的可靠性與 SBOM 品質里程碑。四個階段關閉了長期存在的缺口：掃描器不再因斷開的符號連結而中止，Makefile 變數參照不再洩漏至 `versionInfo`，常見 C/C++ 程式庫現在可解析至真實的 SPDX 授權識別碼，Linux 發佈二進位檔靜態連結，使其可在所有支援的發行版上執行而不受 glibc 版本漂移影響。先前失敗的掃描現在可端對端乾淨地完成。

### 斷開符號連結容忍

此前掃描路徑下任何一處斷開的符號連結都會中止整個走訪。掃描器現在會發出警告並繼續：

```
Warning: skipping /path/to/broken-link: No such file or directory (os error 2)
```

涵蓋全部 5 處 WalkDir 走訪點——主掃描器、`main.rs` 中的回退匯入掃描以及 C/C++ 解析器（`makefile.rs`、`mk_file.rs` 等）。`src/util/mod.rs` 中的共享 `warn_on_walkdir_err` 輔助函式替換了 5 個重複的 `filter_map` closure。

### Makefile `$(VAR)` 過濾

Makefile 片段經常包含未解析的變數參照，如 `DOPENSSL_VERSION := $(OPENSSL_VERSION)`。本次發佈前，這些參照原樣洩漏至 SBOM 輸出中：

**修正前（v1.0.13）：**
```json
{
  "name": "openssl",
  "versionInfo": "$(OPENSSL_VERSION)",
  "downloadLocation": "https://example.invalid/openssl-$(OPENSSL_VERSION).tar.gz"
}
```

**修正後（v1.0.14）：**
```json
{
  "name": "openssl",
  "versionInfo": "NOASSERTION",
  "downloadLocation": "NOASSERTION"
}
```

過濾器套用於 `mk_file.rs` 解析器層，同時在每個版本輸出點（`version_info_or_noassertion`、`generate_cpe_identifier`、`create_download_location`、`create_package_url` 和 CycloneDX 元件版本欄位）進行縱深防禦守衛。任何符合 `$(...)` 的值都會被替換為 `NOASSERTION` 而非被輸出。

### C/C++ 授權解析

原生 C/C++ 相依性此前預設為 `NOASSERTION`，因為 `Makefile` 與 `pkg-config` 檔案都沒有一致地公開標準授權欄位。v1.0.14 新增兩條解析路徑：

1. **`.pc` 的 `License:` 解析** — 當 pkg-config 檔案帶有 `License:` 行時，會被解析並提升為 `licenseConcluded`。
2. **`known_licenses.rs` 表** — 針對 24 個常見系統程式庫的精選查找表（openssl → Apache-2.0、zlib → Zlib、libcurl → curl、libssh2 → BSD-3-Clause、ncurses → X11 等）。在 `makefile.rs`、`mk_file.rs`、`pkgconfig.rs` 和 `pkgconfig_detector.rs` 中用作回退。

`openssl@3.0.7` 的範例輸出現在如下所示：

```json
{
  "name": "openssl",
  "versionInfo": "3.0.7",
  "licenseConcluded": "Apache-2.0",
  "licenseDeclared": "Apache-2.0"
}
```

而非兩個授權欄位皆為 `NOASSERTION`。

### 透過 musl 實現的靜態 Linux 二進位檔

Linux 發佈二進位檔現在以 `x86_64-unknown-linux-musl` 為目標，而非 `x86_64-unknown-linux-gnu`。此前該二進位檔攜帶來自建置主機的 GLIBC 2.39 相依性，於 Ubuntu 22.04 上會報 `version GLIBC_2.39 not found` 錯誤。musl 靜態連結的二進位檔沒有 glibc 相依性，可於任何 x86_64 Linux 上執行——Ubuntu 22.04+、24.04、Alpine、Debian、RHEL 等。向 `build-all.sh` 新增了 musl 交叉連結器工具鏈守衛，CI 的 Ubuntu runner 設定為提供 `musl-tools`。

### 遷移說明

無 CLI 或輸出結構描述變更。於 v1.0.14 上重新產生的既有 SBOM 可能在 C/C++ 元件上出現額外的授權結論欄位以及更少的 `$(VAR)` 風格版本字串——兩者皆為嚴格意義上的改進。此前被 glibc 連結二進位檔卡住的 Linux 使用者可直接替換為 v1.0.14 musl 二進位檔，無需其他變動。

### 測試目標

已針對最初暴露這四個缺陷的真實 C/C++ 專案進行驗證。掃描現在可執行至完成，除了預期的斷開符號連結通知外沒有其他警告，且輸出的 SPDX/CycloneDX 通過結構描述驗證。

詳見 [v1.0.14 計畫](../plan/v1.0.14_user_reported_bugfixes.md) 瞭解完整設計細節。

---

## v1.0.13 - 多模態子模型元件（2026-04-14）

### 概述

v1.0.13 將多模態 AI 模型分解為各自的子模型元件。像 `google/gemma-4-E2B-it` 這樣的模型現在被表示為一個父模型，包含獨立的文字、視覺和音訊子模型——每個子模型在 SBOM 中擁有自己的架構中繼資料。

### 子模型分解

`config.json` 中同時包含 `text_config` 和 `vision_config` 及/或 `audio_config` 的多模態模型會被自動分解：

```
gemma4 (parent — Gemma4ForConditionalGeneration)
├── gemma4_text  (35 layers, 1536 hidden, 8 heads, 262K vocab, 131K context)
├── gemma4_vision (16 layers, 768 hidden, 12 heads, patch_size=16)
└── gemma4_audio  (12 layers, 1024 hidden, 8 heads, conv_kernel=5)
```

### 輸出格式支援

- **CycloneDX** — 在父元件內部巢狀 `components` 陣列，每個子模型作為 `machine-learning-model`，帶有 `radeis:ai:sub_model:*` 屬性
- **SPDX** — 子套件與父模型之間透過 `CONTAINS` 包含關係連接
- **主控台** — 子模型摘要表，顯示模態、模型類型、層數、隱藏層大小、注意力頭數、dtype 及模態特定擴展資訊

### 守衛條件

子模型僅對真正的多模態模型產生——僅有 `text_config`（但沒有 `vision_config` 或 `audio_config`）的純文字模型不會產生多餘的子模型項目。

### 測試覆蓋

- 4 項新增 Safetensors 測試：多模態擷取、純文字守衛、無 text_config 守衛、僅視覺+文字（LLaVA 風格）
- 1 項新增 GGUF 測試：透過伴隨 config.json 補強子模型

詳見 [v1.0.13 計畫](../plan/v1.0.13_multimodal_sub_model_components.md) 瞭解完整設計細節。

---

## v1.0.12 - Safetensors 豐富中繼資料（2026-04-14）

### 概述

v1.0.12 透過從所有 HuggingFace 伴隨設定檔擷取豐富中繼資料，縮小了 GGUF 與 Safetensors SBOM 品質之間的差距。Safetensors 和 GGUF 模型目錄現在都能產生涵蓋架構深度、推論預設值、多模態能力、來源追溯和 adapter 偵測的詳細 SBOM。

### 伴隨檔案豐富中繼資料擷取

- **`config.json` 擴展** — model_type、text_config（hidden_layers、hidden_size、attention_heads、上下文視窗）、多模態偵測（vision_config、audio_config）、dtype 回退鏈
- **`generation_config.json`** — temperature、top_k、top_p 推論參數
- **`tokenizer_config.json`** — processor_class、model_max_length（天文數字安全截斷）
- **`preprocessor_config.json`** — 影像、音訊和視訊處理器類型及序列長度與取樣率
- **`README.md` frontmatter** — base_model（支援字串和清單）、license、model_creator、pipeline_tag、quantized_by、prompt_template、tags、languages、datasets
- **`adapter_config.json`** — LoRA/QLoRA adapter 偵測及基礎模型參考

### GGUF 伴隨檔案補強

GGUF 儲存庫現在同樣受益於伴隨檔案解析。二進位 KV 中繼資料始終優先；伴隨檔案僅填補空缺。Tags、languages 和 datasets 使用來自二進位和 README 兩個來源的去重聯集合併。

### 輸出增強

- **CycloneDX** — 新增約 25 個 `radeis:ai:*` 屬性，涵蓋架構、多模態、生成、處理器、來源追溯和 adapter 中繼資料
- **SPDX** — sourceInfo 擴展，新增 model_type、上下文視窗和模態摘要（保持精簡）
- **主控台** — AI Model Details 表格擴展，新增架構、多模態、生成參數和來源追溯列

### 安全性與邊界情況

- 所有伴隨檔案讀取設有 1 MB 上限，防止記憶體問題
- 不區分大小寫的 README.md 檔名比對（README.md、readme.md、Readme.md）
- CRLF 換行符號正規化，支援 Windows 產生的檔案
- 天文數字的 model_max_length 值（如 Gemma-4 的 1e30）安全捨棄
- prompt_template 截斷至 512 字元（儲存）和 80 字元（主控台顯示）

### 測試覆蓋

- `safetensors_tests.rs` 新增 8 項測試 — 伴隨檔案解析、多模態偵測、dtype 回退鏈、model_max_length 截斷
- `gguf_tests.rs` 新增 6 項測試 — config.json 擷取、README frontmatter（字串和清單 base_model、CRLF、小寫檔名）、adapter 偵測、tags 聯集合併

詳見 [v1.0.12 計畫](../plan/v1.0.12_safetensors_rich_metadata.md) 瞭解完整設計細節。

---

## v1.0.11 - Safetensors AI 模型 SBOM（2026-04-13）

### 概述

v1.0.11 新增 Safetensors AI 模型支援，使 `radeis_sc2sbom` 涵蓋現代 Transformer 模型（LLaMA、Mistral、Falcon 等）的主流格式。模型在目錄級別掃描——無論模型拆分為多少個分片，都只產生一筆 Dependency 記錄，並準確反映總大小、dtype 和架構中繼資料。

### Safetensors AI 模型支援

- **檔案偵測** — 從 `.safetensors`、`model.safetensors.index.json` 和 `config.json` 掃描中繼資料
- **目錄級去重** — 多分片模型（如 `model-00001-of-00002.safetensors`）按模型目錄合併為一筆 SBOM 記錄
- **CycloneDX 輸出** — 使用 `machine-learning-model` 元件類型，附帶包含架構、dtype 和大小中繼資料的 `modelCard`
- **SPDX 輸出** — 產生 `pkg:huggingface` PURL，用於在 HuggingFace 生態系統中識別模型
- **新增 `AIModelMetadata` 欄位**：`safetensors_format`、`total_size_bytes`、`shard_count`、`torch_dtype`、`transformers_version`、`vocab_size`

### 測試覆蓋

- `tests/parser_tests/safetensors_tests.rs` 新增 12 項測試，涵蓋單分片、多分片、基於索引檔案和 config.json 驅動等場景

---

## v1.0.10 - Java Complete（2026-04-13）

### 概述

v1.0.10 將 Gradle 從僅偵測升級為完整的相依性解析，支援 Groovy DSL（`build.gradle`）和 Kotlin DSL（`build.gradle.kts`）。這對於使用基於 Android 的機器人控制器和 Android 裝置上邊緣 AI 推論的 Physical AI 專案至關重要。

### Gradle 相依性解析

- **Groovy DSL**（`build.gradle`）— 解析字串記法（`'group:artifact:version'`）、映射記法（`group: 'g', name: 'a', version: 'v'`）、平台/BOM 宣告
- **Kotlin DSL**（`build.gradle.kts`）— 解析函式記法（`implementation("group:artifact:version")`）和平台 BOM
- **範圍分類** — `testImplementation` → 測試、`compileOnly` → 已提供、`annotationProcessor`/`kapt`/`ksp` → 建置，信賴度 1.0
- **Android 支援** — `androidTestImplementation` 和 `androidTestCompile` 正確分類為測試相依性
- **PURL 格式** — `pkg:maven/{group}/{artifact}@{version}`（與 Maven 相同）

詳見 [v1.0.10 計畫](../plan/v1.0.10_gradle_support.md) 瞭解完整設計細節。

---

## v1.0.9 - Physical AI Ready（2026-04-10）

### 概述

v1.0.9 新增 GGUF AI 模型支援，使 `radeis_sc2sbom` 成為首個原生支援 AI 模型二進位解析與完整性驗證的 SBOM 產生器。本版本亦透過合併 C/C++ 建置系統旗標簡化了 CLI。

### GGUF AI 模型支援

- **二進位解析器**直接從 `.gguf` 檔案擷取中繼資料 — 架構、量化類型、張量佈局、上下文長度及嵌入的授權資訊
- **CycloneDX 輸出**使用 `machine-learning-model` 元件類型，並附帶包含訓練參數和資料集參考的 `modelCard`
- **SPDX 輸出**產生 `pkg:huggingface` PURL，用於在 Hugging Face 生態系統中識別模型
- 使用 `--scan-ai-models true` 啟用

### AI 模型完整性驗證

- **張量參數交叉驗證** — 宣告的張量數量和維度將與實際二進位佈局進行驗證，以偵測截斷或損毀的模型檔案
- **SHA-256 雜湊** — 為每個模型檔案產生內容雜湊，用於供應鏈真實性檢查

### CLI 簡化

- 5 個獨立的 C/C++ 旗標（`--scan-cmake`、`--scan-pkgconfig`、`--scan-autotools`、`--scan-makefiles`、`--scan-mk-files`）合併為 `--scan-c-build-systems`
- `--meson-parse-subprojects` 已移除（啟用 `--scan-meson` 時始終生效）
- `--resolve-system-deps` 已移除（無效程式碼）
- `scan_directory()` 參數從 20 個精簡至 13 個

詳見 [v1.0.9 計畫](../plan/v1.0.9_physical_ai_ready.md) 瞭解完整設計細節。

---

## v1.0.7 - 漏洞掃描改為選用啟用（2026-03-12）

### 概述

漏洞掃描現已改為選用啟用。預設情況下，`radeis_sc2sbom` 產生乾淨的 SBOM，不產生任何網路請求。這使工具執行更快，更適合離線／氣隙環境以及只需要相依性清單的 CI/CD 流水線。

### 變更內容

  - 先前漏洞掃描會自動執行
- **預設輸出更簡潔**
  - 主控台輸出中隱藏漏洞摘要列
  - Markdown 報告中省略風險評估章節

### 遷移指南

```bash
# v1.0.7 之前（漏洞掃描自動執行）
./radeis_sc2sbom --path .

# v1.0.7+ 掃描漏洞（明確選用啟用）

# v1.0.7+ CI/CD：偵測到高危／嚴重漏洞時終止
./radeis_sc2sbom --path . \
```

---

## v1.0.6 - 正式環境 SBOM 過濾與自動相依性範圍分類（2026-03-04）

### 概述

新增自動相依性範圍分類功能，可產生僅包含執行時期實際所需套件的正式環境就緒 SBOM。新增的 `--production` 旗標和 `--scope-filter` 選項可在不損失精確度的前提下大幅縮減 SBOM 體積。

### 主要功能

#### 自動範圍分類
- **6 種範圍類型**：Runtime、Build、Test、Development、Optional、Provided
- **多啟發式引擎**：結合基於生態系統、名稱和目錄的規則
- **信賴度評分**：0.0–1.0 範圍，每項分類附帶可讀的原因說明
- **支援 10+ 種生態系統**：npm、pip、cargo、SYSTEM、BUILD-CONFIG、GIT-SUBMODULE、MESON-WRAP 等

#### 正式環境過濾
- **`--production`** 旗標：僅包含 Runtime 與 Optional 相依性
  - 範例：嵌入式專案從 106 個套件縮減至 33 個（減少 68.9%）
- **`--scope-filter <SCOPE>`**：選擇任意範圍類型組合
  - 支援多值：`--scope-filter runtime --scope-filter optional`
- **範圍統計**顯示在主控台及 Markdown 報告中（各範圍的數量及佔比）

#### 增強的 SBOM 輸出
- **SPDX 2.3**：`primaryPackagePurpose` 欄位由範圍分類填入
- **CycloneDX 1.5**：`scope` 欄位由範圍分類填入

#### 已驗證的分類精確度
| 分類 | 精確度 | 範例 |
|---|---|---|
| 建置工具 | 100% | cmake、gcc、ninja、meson |
| 測試框架 | 100% | pytest、jest、gtest、unity |
| 開發工具 | 100% | pylint、black、eslint、prettier |
| 執行時期函式庫 | 高 | zlib、curl、openssl、protobuf |

### 測試覆蓋
- **609 個測試全部通過**（203 lib + 203 bin + 200 整合 + 3 文件）
- **42 個新增整合測試**，涵蓋範圍過濾、正式環境模式和真實場景分類

### 向下相容性
- 預設行為與 v1.0.5 保持一致——範圍分類自動執行，但在未指定 `--production` 或 `--scope-filter` 時不過濾輸出。

---

## v1.0.5 - 增強版本擷取：雙模式 .mk 檔案掃描（2026-02-26）

### 概述

解決了從 Makefile 偵測到的系統／執行時期函式庫出現「未指定版本」的問題，同時支援對建置系統儲存庫進行獨立的 .mk 檔案清單解析。採用**智慧型雙模式架構**，無需設定即可自動適應不同的儲存庫類型。此版本新增了具備雙工作模式的完整 .mk 檔案解析，以及選用的 .so 二進位掃描（用於建置後版本擷取）。

### 問題描述

**v1.0.5 之前：**
```json
{
  "name": "z",
  "version": "unspecified",  // ❌ 無版本資訊
  "ecosystem": "system"
}
```

**v1.0.5 之後（啟用 .mk 檔案掃描）：**
```json
{
  "name": "z",
  "version": "1.3.1",  // ✅ 從 zlib.mk 擷取
  "ecosystem": "system",
  "source_file": "... [version from .mk file: 1.3.1]"
}
```

### 主要功能

#### 雙模式架構

.mk 檔案掃描器根據儲存庫結構自動選擇適當的模式：

**模式 1：版本解析**（應用程式專案）
- **觸發條件：** 存在 Makefile 且透過 `-l` 旗標偵測到系統函式庫
- **處理方式：** 從 .mk 檔案中解析 Makefile 偵測到的函式庫版本
- **生態系統：** "system"（保留 Makefile 偵測脈絡）
- **範例：** embedded-project 專案——16 個系統函式庫從「unspecified」升級為精確版本
- **適用情境：** 連結系統函式庫的應用程式專案

**模式 2：獨立清單解析**（建置系統儲存庫）
- **觸發條件：** 存在 .mk 檔案但無 Makefile（或 Makefile 中未偵測到函式庫）
- **處理方式：** 直接從 .mk 檔案中的所有 `*_VERSION` 變數建立相依性
- **生態系統：** "BUILD-CONFIG"（表示建置系統原始碼）
- **範例：** embedded-toolchains 專案——偵測到 35 個 BUILD-CONFIG 套件，版本覆蓋率 100%
- **適用情境：** 定義建置時相依性的建置系統儲存庫

**自動去重：**
- 當兩種模式偵測到同一個函式庫時，模式 1（"system"）優先於模式 2（"BUILD-CONFIG"）
- 在保持精確相依性分類的同時防止重複項目
- `deduplicate_dependencies()` 函式中實作了生態系統感知邏輯

#### .mk 檔案解析

從嵌入式系統中常見的建置設定檔擷取版本資訊。

**探索策略：** 採用 glob 模式 `**/*.mk` 尋找儲存庫**任意位置**的 .mk 檔案，不對目錄結構作任何假設。

**範例 .mk 檔案：**
```makefile
# toolchains/3rd_party/curl/curl.mk（或任意位置）
CURL_VERSION ?= 8.15.0
CURL_NAME := curl-$(CURL_VERSION)
LIBCURL_SO := $(LIBCURL).4.8.0
```

**對應策略：**
1. 從 Makefile 偵測函式庫：`-lcurl` → 函式庫名稱："curl"
2. 使用 glob 搜尋 .mk 檔案：`**/*.mk`（在儲存庫任意位置尋找 curl.mk）
3. 解析所有 .mk 檔案並擷取 `CURL_VERSION ?= 8.15.0`
4. 將版本變數對應至函式庫：`CURL_VERSION` → "curl" → "8.15.0"
5. 更新相依性版本："curl" @ "8.15.0"

**支援的模式：**
- `VAR_VERSION ?= value`（條件指定）
- `VAR_VERSION := value`（簡單指定）
- `VAR_VERSION = value`（遞迴指定）

**函式庫名稱正規化：**（僅模式 1——用於解析 Makefile 的 `-l` 旗標）
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
- 一般規則：`foo` → `libfoo`（兩種形式均嘗試）

**建置工具過濾：**（僅模式 2——防止誤報）
- 過濾建置工具：make、cmake、gcc、clang、python、perl、ruby、autoconf、automake、libtool、ninja、meson、bash、sh、awk、sed
- 防止 SBOM 中出現「make@4.3」或「cmake@3.25.0」等誤報相依性

#### .so 二進位掃描（優先順序 2）

從已建置的函式庫二進位檔擷取版本（建置後方式）。

**技術手段：**
1. **解析 .so 檔案名稱：** `libcurl.so.4.8.0` → 版本「4.8.0」
2. **讀取 ELF soname：** `readelf -d libcurl.so | grep SONAME`（如 readelf 可用）
3. **擷取版本字串：** 在二進位內容中搜尋版本模式

**搜尋目錄：**
- `lib/`
- `lib64/`（64 位元函式庫）
- `build/`
- `build/lib/`（CMake 外部建置）
- `toolchains/install/lib/`
- `usr/lib/`
- `usr/lib64/`（64 位元系統函式庫）
- `usr/local/lib/`（本機安裝）
- `.libs/`（autotools）

**符號連結去重：**
- 自動解析符號連結鏈（例如 `libcurl.so` → `libcurl.so.4` → `libcurl.so.4.8.0`）
- 採用標準路徑防止重複掃描同一函式庫
- 確保跨符號連結函式庫的版本報告一致

**限制：** 需要函式庫已完成建置（不適用於純原始碼儲存庫）。

### CLI 旗標

```bash
# .mk 檔案版本擷取（預設啟用）
--scan-mk-files=true/false     # 預設：true

# .so 二進位版本擷取（預設停用，需要已建置的函式庫）
--scan-so-files=true/false     # 預設：false
```

**預設值說明：**
- `--scan-mk-files=true`：對純原始碼儲存庫安全，額外負擔低，適用於任意 .mk 檔案位置
- `--scan-so-files=false`：需要已建置的函式庫，CI/CD 中可能不存在

### 實際效果

**embedded-project 專案：**
- **v1.0.5 之前：** 32 個系統函式庫版本為「unspecified」
- **v1.0.5 之後：** 32 個系統函式庫從 .mk 檔案擷取了精確版本
  - curl @ 8.15.0
  - elfutils @ 0.191
  - zlib @ 1.3.1
  - openssl @ 3.2.5
  - libssh2 @ 1.11.0
  - 以及另外 27 個……

### 向下相容性

✅ **100% 向下相容**
- 無 .mk 檔案的現有專案：行為不變
- 預設的 `--scan-mk-files=true` 僅影響以 Makefile 為基礎的專案
- 無 .mk 檔案的專案繼續顯示「unspecified」版本
- 版本解析為累加式：不替換任何既有版本

### 測試

- **154 個單元測試通過**——包含所有現有測試及新增的 .mk 和 .so 解析器測試
- **3 個新增整合測試**——模式 1／模式 2 去重、建置工具過濾、多檔案情境
- **全面程式碼審查**——所有問題已處理（去重邏輯、符號連結處理、目錄遍歷最佳化）
- **整合測試**涵蓋 embedded-project 和 embedded-toolchains 專案結構
- **零迴歸**於現有解析器

### 修改的檔案

**新增檔案：**
- `src/parsers/c/mk_file.rs` - 帶版本擷取的 .mk 檔案解析器
- `src/parsers/c/so_scanner.rs` - 帶版本擷取的 .so 二進位掃描器

**修改檔案：**
- `src/parsers/c/makefile.rs` - 為模式 1 新增版本解析邏輯
- `src/parsers/mod.rs` - 為模式 1／模式 2 新增生態系統感知去重
- `src/cli.rs` - 新增 `--scan-mk-files` 和 `--scan-so-files` 旗標
- `src/scanner/mod.rs` - 將新旗標傳遞給 Makefile 解析器 + 模式 2 觸發
- `src/main.rs` - 將 CLI 旗標接入掃描器
- `Cargo.toml` - 新增 `glob` 相依性以探索 .mk 檔案
- `tests/parser_tests/c_tests.rs` - 新增去重和過濾的整合測試

### 未來改進計畫（v1.0.6+）

1. **智慧型 .mk 模式偵測** - 從多個專案學習 .mk 檔案模式
2. **pkg-config .pc 產生** - 從 .mk 檔案產生 .pc 檔案供其他工具使用
3. **建置系統外掛架構** - 透過外掛支援自訂建置系統
4. **版本限制解析** - 將「>=3.0」等限制解析為實際版本
5. **從 .mk 檔案擷取授權** - 從建置設定中擷取授權資訊

---

## v1.0.4 - Meson 與 Bazel 建置系統支援（2026-02-25）

### 概述

新增對現代 C/C++ 建置系統（Meson 和 Bazel）的支援，完善了 radeis_sc2sbom 對 C/C++ 生態系統的全面覆蓋。結合 v1.0.0–1.0.3（vcpkg、Conan、CMake、Git 子模組、Autotools、pkg-config、Makefile），radeis 現已支援約 **95% 的 C/C++ 專案**。

### 主要功能

#### Meson 建置系統支援

解析 `meson.build` 檔案中的 `dependency()` 宣告：

```python
# meson.build 範例
project('myapp', 'cpp')

# 帶版本限制的 dependency()
zlib_dep = dependency('zlib', version: '>=1.2.11')

# 不帶版本的 dependency()
openssl_dep = dependency('openssl')

# 透過 find_library() 參照系統函式庫
cc = meson.get_compiler('c')
math_dep = cc.find_library('m')

# 子專案參照
libfoo_proj = subproject('libfoo')

executable('myapp', 'src/main.cpp',
  dependencies: [zlib_dep, openssl_dep, math_dep])
```

**支援的功能：**
- 從 `dependency()` 宣告中擷取函式庫名稱
- 從 `version:` 引數擷取版本限制（如有）（>=、==、>、<、!=）
- 從 `cc.find_library()` 呼叫擷取系統函式庫
- 從 `subproject()` 呼叫偵測子專案參照（實際解析透過 `.wrap` 檔案進行）
- PURL 格式：`pkg:generic/{name}@{version}?type=meson`（有版本時包含版本）

**說明：** 解析器目前擷取相依性名稱和版本限制。`modules:` 陣列和 `required:` 旗標在語法層面可識別，但目前未擷取到結構化輸出中。

**真實專案驗證：**
- OpenStudio 專案在 conan.lock 中將 meson 1.2.2 列為開發相依性
- 已成功在正式環境掃描中偵測並驗證
- 證明了對正在遷移至 Meson 的 C++ 專案的即時適用性

#### Bazel 建置系統支援

解析 `WORKSPACE`/`WORKSPACE.bazel` 和 `MODULE.bazel` 檔案中的外部相依性：

```python
# WORKSPACE 範例
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

# MODULE.bazel 範例（Bazel 6.0+ bzlmod）
module(name = "myproject")

bazel_dep(name = "abseil-cpp", version = "20230802.1")
bazel_dep(name = "googletest", version = "1.14.0")
```

**支援的功能：**
- 解析 `WORKSPACE`/`WORKSPACE.bazel` 中的 `http_archive`、`git_repository` 和本機儲存庫
- 解析 `MODULE.bazel`（Bazel 6.0+ bzlmod）中的 `bazel_dep()` 宣告
- 從外部相依性宣告中擷取 URL 和版本
- 支援多行相依性宣告
- PURL 格式：`pkg:generic/{name}@{version}?type=bazel`，或對基於 git 的相依性採用 `pkg:github/{owner}/{repo}@{version}?type=bazel`

### CLI 旗標

```bash
# 啟用／停用 Meson 和 Bazel 支援（預設均啟用）
--scan-meson=true/false        # meson.build 和 .wrap 檔案
--scan-bazel=true/false        # WORKSPACE、WORKSPACE.bazel、MODULE.bazel 檔案
```

### 覆蓋範圍影響

**v1.0.4 之前：**
- 現代 C++（vcpkg、Conan、CMake）：約 70–80%
- 舊版 C（Autotools、pkg-config、Makefile）：約 80–90%
- 綜合覆蓋率：約 **90% 的 C/C++ 專案**

**v1.0.4 之後：**
- 現代 C++（vcpkg、Conan、CMake、Meson、Bazel）：約 75–85%
- 舊版 C（Autotools、pkg-config、Makefile）：約 80–90%
- 綜合覆蓋率：約 **95% 的 C/C++ 專案**

### 真實專案支援

已成功測試的專案：
- **OpenStudio**（Conan）——偵測到 meson 1.2.2 作為開發相依性
- **單元測試**——105 個測試通過（Meson 和 Bazel 解析器）

### 正式環境驗證

**OpenStudio 掃描結果（v1.0.4）：**
- 共 49 個套件（48 個 Conan + 1 個 Python）
- 在 conan.lock 中偵測到 meson 1.2.2 作為開發相依性
- 安全掃描結果乾淨（0 個漏洞）
- 完整解析 conan.lock，含開發相依性分類

這驗證了 v1.0.4 的 Meson 支援對採用現代建置系統的真實 C++ 專案立即適用。

### 向下相容性

✅ **100% 維持相容** - 現有生態系統解析器無迴歸：
- 全部 105 個單元測試通過
- curl 結果與 v1.0.3 完全一致（已驗證）
- npm、Python、ROS、Conan、Git 子模組、CMake、Autotools 均保持穩定
- Meson 和 Bazel 解析器僅為新增功能

### 綜合比較報告

作為 v1.0.4 驗證的一部分，我們已完成針對 6 個多元化儲存庫的綜合比較報告（每份 375–596 行）：

1. **curl**（C 函式庫）- 446 行
2. **nodejs-service**（Node.js）- 444 行
3. **nodejs-project**（多雲端）- 375 行
4. **OpenStudio**（C++ Conan）- 398 行
5. **mrpt**（機器人 C++）- 596 行
6. **ros2cli**（ROS 2）- 590 行

**合計：** 2,897 行綜合分析

**主要發現：**
- **跨所有專案共追蹤 2,561 個相依性**
- **4 項其他工具不具備的獨特能力：**
  - C/C++ Autotools（curl：29 個函式庫）
  - ROS 2（ros2cli：223 個元件）
  - Git 子模組（mrpt：8 個含 SHA 的子模組）
  - CMake ExternalProject（mrpt：3 個相依性）
- **相比 BlackDuck 3 年節省 22 萬〜165 萬美元**

詳見 [scan_reports/COMPARISON_REPORTS_INDEX.md](../scan_reports/COMPARISON_REPORTS_INDEX.md)。

---

## v1.0.3 - C 舊版支援（pkg-config + Autotools + Makefile）（2026-02-24）

### 概述

新增對傳統 C 專案建置系統的全面支援，支援採用 GNU Autotools、pkg-config 和純 Makefile 的舊版 C/C++ 專案產生 SBOM。填補了現代套件管理器支援（vcpkg、Conan、CMake）留下的關鍵缺口，實現了對約 90% C/C++ 專案的覆蓋。

### 主要功能

#### pkg-config（.pc 檔案）支援

解析 `.pc`（pkg-config）檔案以擷取系統函式庫相依性：

```
Name: OpenSSL
Version: 3.0.2
Description: Secure Sockets Layer and cryptography libraries
Requires: libcrypto libssl
```

**支援的功能：**
- 從 .pc 檔案擷取套件名稱、版本和描述
- 偵測 configure.ac 中的 PKG_CHECK_MODULES() 呼叫
- 偵測 Makefile 中的 pkg-config shell 呼叫
- PURL 格式：`pkg:generic/{name}@{version}?type=pkg-config`

#### Autotools（configure.ac/Makefile.am）支援

解析 GNU Autotools 設定檔中的函式庫相依性：

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

**支援的功能：**
- 從 AC_CHECK_LIB、AC_SEARCH_LIBS、PKG_CHECK_MODULES 擷取相依性
- 從 Makefile.am 的 LDADD/LIBADD 變數擷取 -l 旗標
- 保留 PKG_CHECK_MODULES 中的版本限制
- PURL 格式：`pkg:generic/{name}@{version}?type=autotools`

#### 純 Makefile 啟發式解析器

對手寫 Makefile 進行盡力解析：

```makefile
LDFLAGS = -lssl -lcrypto -lpthread -lz
OPENSSL_CFLAGS = $(shell pkg-config --cflags openssl)
```

**支援的功能：**
- 採用正規表示式擷取 -l 旗標（系統函式庫）
- 偵測 pkg-config 呼叫
- 依函式庫名稱去重
- PURL 格式：`pkg:generic/{name}@{version}?type=makefile`

**限制：**
- 不支援變數展開（`$(FOO)`）
- 不支援條件區塊（`ifeq`）
- 在 Autotools 專案中跳過 Makefile 解析

### CLI 旗標

```bash
# 啟用／停用 C 舊版支援（預設全部啟用）
--scan-pkgconfig=true/false       # .pc 檔案和 PKG_CHECK_MODULES
--scan-autotools=true/false       # configure.ac 和 Makefile.am
--scan-makefiles=true/false       # 純 Makefile（啟發式）
--resolve-system-deps=false       # 系統 pkg-config 解析（預設停用）
```

### 覆蓋範圍影響

**v1.0.3 之前：**
- 現代 C++（vcpkg、Conan、CMake）：約 70–80%
- 舊版 C（Autotools）：<1%
- 系統函式庫：<1%

**v1.0.3 之後：**
- 現代 C++（vcpkg、Conan、CMake）：約 70–80%
- 舊版 C++（Makefile）：約 40%
- 純 C（Autotools）：約 60%
- 系統函式庫相依性：約 80%

**綜合覆蓋率：約 90% 的 C/C++ 專案**

### 真實專案支援

已成功測試的專案：
- **curl**（Autotools）——偵測到 openssl、zlib、nghttp2
- **nginx**（Makefile）——偵測到 openssl、pcre、zlib
- 標準 C 函式庫（pthread、m、ssl、crypto、z）

### 輸出範例

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

### PURL 範例

- **pkg-config**：`pkg:generic/openssl@3.0.2?type=pkg-config`
- **Autotools**：`pkg:generic/pthread@unspecified?type=autotools`
- **Makefile**：`pkg:generic/ssl@unspecified?type=makefile`

---

## v1.0.2 - Conan C++ 套件管理器支援（2026-02-24）

### 概述

新增對 Conan C/C++ 套件管理器的完整支援，支援採用 Conan 2.x 的專案產生 SBOM。支援解析鎖定檔、INI 格式清單和 Python 格式清單，涵蓋執行時期、建置、工具和測試相依性。

### 主要功能

#### Conan 鎖定檔解析（conan.lock）

解析 Conan 2.x 鎖定檔，包含精確版本和配方修訂：

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

**支援的功能：**
- 從參照中擷取套件名稱和版本
- 將配方修訂雜湊作為總和檢查碼儲存
- 區分執行時期、建置、工具和測試相依性
- 同目錄下鎖定檔優先於清單檔

#### Conan 清單解析（conanfile.txt）

解析 INI 格式的 Conan 清單：

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

**支援的功能：**
- 版本限制：`[>=3.0]`、`[>1.0 <2.0]`、`[~=1.82]`、`[^1.0]`
- 使用者／頻道表示法：`package/version@user/channel`
- 建置、工具和測試相依性標記為開發相依性

#### Conan Python 清單解析（conanfile.py）

採用正規表示式擷取解析 Python 格式的 Conan 清單：

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

**支援的模式：**
- 清單格式：`requires = ["dep1", "dep2"]`
- 方法呼叫：`self.requires("dep")`
- 建置相依性：`build_requires`、`self.build_requires()`
- 工具相依性：`tool_requires`、`self.tool_requires()`
- 測試相依性：`test_requires`、`self.test_requires()`

#### SBOM 輸出

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

### 技術實作

**新增檔案：**
- [src/parsers/cpp/conan.rs](../src/parsers/cpp/conan.rs) - conan.lock 解析器
- [src/parsers/cpp/conan_manifest.rs](../src/parsers/cpp/conan_manifest.rs) - conanfile.txt/py 解析器
- [tests/parser_tests/conan_tests.rs](../tests/parser_tests/conan_tests.rs) - 10 個測試案例

**整合：**
- 掃描器偵測 `conan.lock`、`conanfile.txt`、`conanfile.py`
- 鎖定檔優先：若存在 `conan.lock` 則跳過清單
- SPDX purl 格式：`pkg:conan/{name}@{version}`
- 支援 CycloneDX 元件

### 測試覆蓋

```bash
cargo test conan_tests
```

**10 個測試案例涵蓋：**
- 帶配方修訂的鎖定檔解析
- 帶版本限制的 INI 清單解析
- Python 清單解析（清單和方法格式）
- 版本範圍處理
- 使用者／頻道表示法
- 格式錯誤輸入處理
- 空清單處理
- purl 格式產生

### 範例

```bash
# 掃描含 Conan 相依性的專案
./target/release/radeis_sc2sbom --path /path/to/conan-project --format spdx-json

# 輸出範例
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

### 設計決策

1. **鎖定檔優先**：`conan.lock` 提供精確版本，優先於清單檔
2. **配方修訂**：儲存在 `checksum_sha256` 欄位以供溯源
3. **開發相依性**：建置、工具和測試相依性以 `is_dev: true` 標記
4. **版本限制**：從清單中原樣保留（如 `>=3.0`）
5. **生態系統識別碼**：所有 Conan 相依性採用 `"conan"`

---

## v1.0.1 - CMake 支援與遞迴子模組掃描（2026-02-23）

### 概述

新增靜態 CMake 相依性偵測（FetchContent/ExternalProject）及 Git 子模組內的遞迴相依性掃描。無需執行 CMake 建置即可完整探索所有相依性。

### 主要功能

#### CMake 相依性解析

靜態解析 CMakeLists.txt 檔案以擷取 FetchContent_Declare 和 ExternalProject_Add 相依性：

```cmake
# FetchContent_Declare（現代 CMake 3.11+）
FetchContent_Declare(
  json
  GIT_REPOSITORY https://github.com/nlohmann/json.git
  GIT_TAG        v3.11.2
)

# ExternalProject_Add（舊版模式）
ExternalProject_Add(
  zlib
  URL https://zlib.net/zlib-1.2.13.tar.gz
  URL_HASH SHA256=abc123...
)
```

**支援的功能：**
- GIT_REPOSITORY + GIT_TAG 擷取
- 基於 URL 的相依性及從 URL 擷取版本
- URL_HASH（SHA256）總和檢查碼擷取
- 跳過含 CMake 變數 `${VAR}` 的項目（無法靜態解析）
- 多代管平台 Git URL 解析（GitHub、GitLab、Bitbucket、一般）

**SBOM 輸出：**
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

#### 遞迴子模組掃描

自動掃描 Git 子模組內部的相依性（package.json、Cargo.toml、CMakeLists.txt 等），支援深度限制：

**範例：** 含子模組的專案（子模組內有 npm 相依性）
```
.
├── .gitmodules
├── libs/
│   └── json/          # Git 子模組
│       ├── package.json   # 遞迴掃描
│       └── CMakeLists.txt # 同樣掃描
```

**功能：**
- 偵測子模組內所有清單類型（npm、Cargo、vcpkg、CMake 等）
- 支援巢狀子模組（子模組中的子模組）
- 透過 `--submodule-depth` 設定深度限制（預設：3 層）
- 來源標註顯示子模組來源

**巢狀相依性的 SBOM 輸出：**
```json
{
  "name": "typescript",
  "versionInfo": "5.0.0",
  "ecosystem": "npm",
  "sourceInfo": "javascript/packagejson extractor from libs/json/package.json (submodule: libs/json)"
}
```

#### CLI 增強

**新增旗標：**
- `--scan-cmake=<true|false>` - 啟用／停用 CMake 掃描（預設：true）

**現有旗標（現已完整實作）：**
- `--submodule-depth=<N>` - 巢狀子模組最大遞迴深度（預設：3）

**範例：**
```bash
# 啟用 CMake 和遞迴子模組掃描（預設）
./target/release/radeis_sc2sbom /path/to/project

# 停用 CMake 掃描
./target/release/radeis_sc2sbom /path/to/project --scan-cmake=false

# 將子模組遞迴限制為 1 層
./target/release/radeis_sc2sbom /path/to/project --submodule-depth=1
```

### 技術細節

#### CMake 解析器實作
- **檔案：** [src/parsers/cmake/mod.rs](../src/parsers/cmake/mod.rs)、[src/parsers/cmake/fetchcontent.rs](../src/parsers/cmake/fetchcontent.rs)、[src/parsers/cmake/external_project.rs](../src/parsers/cmake/external_project.rs)
- **模式：** 以正規表示式為基礎的解析，採用 `(?is)` 旗標（大小寫不敏感 + 多行）
- **限制：** 無法解析 CMake 變數——僅支援靜態值

#### 遞迴掃描實作
- **函式：** [src/scanner/mod.rs](../src/scanner/mod.rs) 中的 `scan_submodule_recursively()`
- **安全性：** 深度限制防止循環參照導致的無限迴圈
- **覆蓋：** 所有現有解析器（npm、Cargo、Python、Go、vcpkg、CMake 等）

### 競爭優勢

|---------|----------------|-------------|-----------|
| **CMake FetchContent** | ✅ 靜態解析 | ❌ | ✅ 僅建置捕獲 |
| **CMake ExternalProject** | ✅ 靜態解析 | ❌ | ✅ 僅建置捕獲 |
| **子模組內巢狀相依性** | ✅ 遞迴掃描 | ❌ | ❌ |
| **無需建置** | ✅ 全靜態 | ✅ | ❌ CMake 需要建置 |

### 遷移說明

**API 變更：**
- `scan_directory()` 簽章新增 `scan_cmake` 參數（影響自訂整合）
- 測試已更新以包含 `scan_cmake` 引數

**重大變更：** 對 CLI 使用者無重大變更（向下相容）

---

## v1.0.0 - C++ 支援（2026-02-20）

### 概述

首次支援 C++ 生態系統，包含 vcpkg 清單解析和 Git 子模組偵測。支援採用現代套件管理器的 C/C++ 專案產生 SBOM。此工具採用 Rust 開發，具備高效能靜態分析能力。

### 主要功能

#### vcpkg 清單解析器

完整支援 vcpkg.json，涵蓋所有版本限制格式：

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

**支援的版本格式：**
- 簡單字串相依性：`"zlib"`
- `version>=`：最低版本限制
- `version>`：大於版本
- `version=`：精確版本
- `version-semver`：語意版本
- `version-date`：基於日期的版本
- `port-version`：連接埠修訂號
- overrides 段用於版本固定
- features 儲存在 source_file 中繼資料中

**SBOM 輸出：**
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

#### Git 子模組偵測

自動解析 `.gitmodules` 檔案並解析提交 SHA：

```ini
[submodule "libs/json"]
    path = libs/json
    url = https://github.com/nlohmann/json.git
    branch = master
```

**功能：**
- 解析子模組名稱、路徑、URL 和分支
- 透過 `git ls-tree HEAD` 解析提交 SHA
- 支援 HTTPS 和 SSH URL
- 支援多代管平台：GitHub、GitLab、Bitbucket、自行代管

**SBOM 輸出：**
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

#### 潛在漏洞標記

透過匯入掃描（version="detected"）發現的套件，其漏洞現標記為**潛在**：

```
   Package version 'detected' is unknown (detected via import scanning).
   Actual version may not be affected.
```

這有助於區分：
- **已確認漏洞** - 版本已知的套件（如 `urllib3@2.6.0`）
- **潛在漏洞** - 透過匯入掃描偵測到但無版本資訊的套件

#### 版本格式修正

- **修正前：** `"==2.6.0"`（無效的 purl 格式）
- **修正後：** `"2.6.0"`（正確的 purl 格式）

此修正顯著減少了誤報漏洞數量（37 → 8，以 MRPT 比較為例）。

#### Python 偵測方法

radeis 掃描來自清單檔（requirements.txt、pyproject.toml）的**宣告相依性**，比掃描已安裝環境的工具產生更精確的 SBOM：

| 方式 | radeis | 環境掃描工具 |
|----------|--------|---------------------|
| 偵測方法 | 宣告相依性 | 已安裝套件 |
| 捕獲內容 | 專案需求 | 特定環境的套件 |
| 精確度 | 更精確 | 可能含誤報 |

**關鍵洞察：** 環境掃描工具可能包含 `importlib-metadata` 和 `zipp`（Python < 3.10 的回移植套件）等並非實際專案需求的套件。

### 新增 CLI 選項

```bash
--scan-submodules <BOOL>   # 啟用 Git 子模組掃描（預設：true）
--submodule-depth <N>      # 最大遞迴深度（預設：3）
```

**範例：**
```bash
# 掃描含 vcpkg 的 C++ 專案
radeis_sc2sbom --path ./cpp_project --format spdx-json

# 停用子模組掃描
radeis_sc2sbom --path . --scan-submodules false

# 限制子模組遞迴深度
radeis_sc2sbom --path . --submodule-depth 1
```

### purl 格式支援

| 生態系統 | purl 格式 | 範例 |
|-----------|-------------|---------|
| vcpkg | `pkg:vcpkg/{name}@{version}` | `pkg:vcpkg/zlib@1.2.13` |
| GitHub | `pkg:github/{owner}/{repo}@{commit}` | `pkg:github/nlohmann/json@bc889af` |
| GitLab | `pkg:gitlab/{owner}/{repo}@{commit}` | `pkg:gitlab/owner/repo@abc123` |
| Bitbucket | `pkg:bitbucket/{owner}/{repo}@{commit}` | `pkg:bitbucket/owner/repo@def456` |
| 一般 | `pkg:generic/{name}@{version}` | `pkg:generic/custom-lib@1.0.0` |

### 技術變更

**新增檔案：**
- `src/parsers/cpp/mod.rs` - C++ 解析器模組入口
- `src/parsers/cpp/vcpkg.rs` - vcpkg.json 解析器
- `src/parsers/git/mod.rs` - Git 解析器模組入口
- `src/parsers/git/submodules.rs` - .gitmodules 解析器
- `src/parsers/git/commit_resolver.rs` - Git 提交 SHA 解析器
- `src/parsers/git/url_parser.rs` - Git URL 解析器

**修改檔案：**
- `src/cli.rs` - 新增 CLI 選項
- `src/scanner/mod.rs` - vcpkg.json 和 .gitmodules 偵測
- `src/formats/spdx.rs` - vcpkg/git-submodule purl 支援
- `src/formats/console.rs` - 潛在漏洞顯示
- `src/parsers/python.rs` - 版本格式修正（去除運算子）
- `src/parsers/mod.rs` - 將 `__future__` 加入 Python 標準函式庫
- `src/main.rs` - 將新引數傳遞給掃描器

### 從 v0.9.3 遷移

**無重大變更。** 所有現有功能正常運作。

新功能在以下情況自動啟用：
- 發現 `vcpkg.json` → 執行 vcpkg 解析器
- 發現 `.gitmodules` → 執行子模組偵測（需要 git）

### 未來路線圖（v1.0.x）

- v1.0.1：CMake FetchContent/ExternalProject 解析
- v1.0.2：Conan 套件管理器支援
- 子模組內遞迴掃描

---

## v0.9.3 - Python 全面優化（2026-02-10）

### 概述

完整支援 Pipenv 和 pyproject.toml，Python 套件偵測量提升 **487%**。

### 主要改進

#### Pipfile/Pipfile.lock 解析器
完整整合 Pipenv，支援鎖定檔解析。

**改進前：**
```json
{
  "name": "azure-identity",
  "versionInfo": "detected",
  "sourceInfo": "Import scanner"
}
```

**改進後：**
```json
{
  "name": "azure-identity",
  "versionInfo": "1.12.0",
  "downloadLocation": "https://pypi.org/project/azure-identity/1.12.0/",
  "sourceInfo": "python/pipfilelock extractor"
}
```

**結果（nodejs-service）：**
- 從 Pipfile.lock 偵測到 47 個套件（之前為 8 個）
- 100% 版本精確率（零「@detected」）
- 與 Black Duck 完全一致（47/47）
- 擷取 SHA256 總和檢查碼

#### pyproject.toml 多格式解析器
自動支援三種格式：

**PEP 621（標準格式）：**
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

#### SHA256 總和檢查碼擷取
從鎖定檔擷取總和檢查碼以確保供應鏈安全：
- Pipfile.lock：hashes 陣列中的第一個 SHA256
- poetry.lock：files[0].hash 欄位中的雜湊
- 內部儲存（尚未輸出到 SPDX/CycloneDX）

#### 重複防止
鎖定檔現在跳過對應的清單檔：
- 存在 Pipfile.lock → 跳過 Pipfile
- 存在 poetry.lock → 跳過 pyproject.toml

### 效能指標

| 指標 | v0.9.3 | v0.9.2 | 提升幅度 |
|--------|--------|--------|-------------|
| Python 套件 | 47 | 8 | +487% |
| 真實版本 | 47 | 0 | +47 |
| 「@detected」版本 | 0 | 8 | -100% |
| Black Duck 一致率 | 100% | 17% | +83% |

### 競爭定位

| 工具 | 套件數 | 版本精確率 | 總和檢查碼 | 格式 |
|------|----------|------------------|-----------|---------|
| **radeis v0.9.3** | 47 🏆 | 100% 🏆 | ✅ SHA256 🏆 | Pipfile、poetry、pyproject 🏆 |
| Black Duck | 47 | 100% | 未知 | Pipfile、poetry |

### 新增檔案支援

**Pipfile.lock（鎖定檔）**
- 精確版本（`==1.12.0`）
- SHA256 雜湊
- 直接相依性偵測（`index` 欄位）
- 開發相依性（`develop` 段）

**Pipfile（清單）**
- 版本規格（`*`、`>=1.0`、`==1.2.3`）
- 開發相依性
- 多來源支援

**pyproject.toml（多格式）**
- PEP 621、Poetry、PDM 格式
- 自動偵測
- 版本規格與擴充功能

### 技術變更

**修改檔案：**
- `src/parsers/python.rs`（+219 行）- 新增解析器
- `src/scanner/mod.rs`（+30 行）- 解析器註冊
- `Cargo.toml` - 版本升至 0.9.3

**關鍵演算法：**
```rust
// Pipfile.lock 結構
#[derive(Deserialize)]
struct PipfileLock {
    default: HashMap<String, PipfilePackage>,
    develop: Option<HashMap<String, PipfilePackage>>,
}

// 總和檢查碼擷取
fn extract_first_sha256(hashes: &[String]) -> Option<String> {
    hashes.iter()
        .find(|h| h.starts_with("sha256:"))
        .map(|h| h.trim_start_matches("sha256:").to_string())
}

// pyproject.toml 多格式處理
pub fn parse_pyproject_toml(path: &Path) -> Result<Vec<Dependency>> {
    // 依序嘗試 PEP 621、Poetry、PDM
    if let Some(project) = pyproject.get("project") { ... }
    if let Some(poetry) = pyproject.get("tool").and_then(|t| t.get("poetry")) { ... }
    if let Some(pdm) = pyproject.get("tool").and_then(|t| t.get("pdm")) { ... }
}
```

### 使用情境

**Pipenv 專案：**
```bash
radeis_sc2sbom --path ./python_project --format spdx-json

# 輸出：47 個套件，100% 版本精確率
```

**現代 Python 專案：**
```bash
radeis_sc2sbom --path ./pyproject_project --format all

# 自動偵測 PEP 621、Poetry 或 PDM 格式
```

**供應鏈稽核：**
```bash
radeis_sc2sbom --path ./workspace --format spdx-json

# 從鎖定檔擷取 SHA256 總和檢查碼
```

### 從 v0.9.2 遷移

**無重大變更。** 所有功能自動生效：

```bash
# v0.9.2：8 個套件，全為「@detected」
radeis_sc2sbom --path ./nodejs-service --format spdx-json

# v0.9.3：47 個套件，全為真實版本
# （命令相同，自動改善）
```

**變更內容：**
- ✅ 自動解析 Pipfile/Pipfile.lock
- ✅ 解析 pyproject.toml（所有格式）
- ✅ 擷取 SHA256 總和檢查碼
- ✅ 100% 版本精確率
- ✅ 零效能影響

### 錯誤修正

1. Python 版本偵測——透過鎖定檔消除「@detected」
2. 總和檢查碼擷取——新增 SHA256 支援
3. 解析器優先順序——鎖定檔覆寫清單
4. 重複防止——存在鎖定檔時跳過清單

---

## v0.9.2 - 使用者體驗優化（2026-02-08）

### 進度指示器

掃描過程中提供即時回饋：

```
[1/5] Walking directory tree... 47 entries scanned
[2/5] Parsing complete... 54 dependencies discovered
[3/5] Deduplicating dependencies... 54 → 47 unique
[5/5] Scan complete
```

### 功能

- 帶百分比進度的進度列
- 長時間操作的旋轉動畫
- 5 階段流水線可見性
- 預估完成時間

---

## v0.9.1 - ROS 整合（2026-01-28）

### 自動版本解析

ROS 套件在 `package.xml` 中缺少版本資訊。v0.9.1 透過 rosdistro API 實作自動解析。

**改進前：**
```
rclpy @ unspecified
downloadLocation: NOASSERTION
```

**改進後：**
```
rclpy @ 7.1.9 (from rosdistro)
downloadLocation: https://github.com/ros2/rclpy.git
```

### ros2cli 基準測試

- 94 個唯一相依性（對比 BlackDuck 的 4 個）
- 偵測到 15 個 ROS 套件
- 47 個套件帶 GitHub URL
- 62 個套件有版本號（66%）
- 發現 5 個漏洞

### 功能

- 平行 API 擷取（速度提升 10〜27 倍）
- SHA-1 總和檢查碼
- 自動修復建議
- 精簡 SPDX 模式（體積縮小 30%）

**支援的發行版：** jazzy、iron、humble、rolling、noetic、melodic

---

## v0.9.1.1 - 緊急修正（2026-01-28）

### 修正：Markdown 報告顯示問題

**問題：** ROS 多套件報告中出現空程式碼區塊

**修正：** 在 ROS 報告章節中顯示所有直接相依性

---

## v0.8.0 - 中繼資料與安全性（2026-01-22）

### 豐富中繼資料

- 跨生態系統授權覆蓋率 95%+
- 供應商／來源追蹤 90%+
- 基於 UUID 的 SPDX ID
- 安全工具使用的 CPE 識別碼

### 改進內容

- 平行中繼資料擷取
- 增強的授權偵測
- 改進的供應商追蹤
- 更完善的 CPE 產生

---

## 未來路線圖

### v0.9.4+
- 在 SPDX/CycloneDX 輸出中顯示總和檢查碼
- 增強 PyPI 中繼資料快取
- pdm.lock 支援
- conda environment.yml 支援

---

**最新版本：** v1.0.7（2026 年 3 月 12 日）
**狀態：** 正式環境就緒
**Python 支援：** 業界領先 🏆
