# 新機能・変更履歴

## v1.0.18 — Tree-sitter AST スキャナー（2026-05-13）

### 概要


### Tree-sitter AST スキャナー


- **レキシカルフォールバック** — パース失敗（構文エラー、プリプロセッサ構造）のファイルは自動的にレキシカルスキャナーにフォールバックし、サイレントスキップがありません。



|-----|--------|


### `--experiment-scan`


| モード | 発見項目数 | FP 率 |
|--------|----------|-------|


```bash

```

### Phase 24 チューニング

リリース前に 17 のターゲットを絞ったルールレベルの変更を適用し、誤検知を削減しました。主な変更：


純効果：v1.0.18 前ベースライン比で −89,479 の誤検知（217,279 → 127,800 件の合計発見項目）。

### 移行メモ

`--cppcheck-path` は受け付けられなくなりました；ビルドスクリプトから削除してください。その他の内部ビルドフラグは変更ありません。パブリックビルドは影響を受けません。

---

## v1.0.17 — 高度な C/C++ SAST と AUTOSAR バージョン抽出（2026-05-11）

### 概要

v1.0.17 では内部 SAST スキャナーに引数値検査と cppcheck サブプロセス統合を追加し、CI/IDE パイプライン向けの SARIF 2.1 出力を新設、3 つの AUTOSAR 精度問題を修正します：arxml の完全な依存関係解析、`.epd` ファイルと Doxygen ヘッダーからのコンポーネントバージョン抽出、リンカーフラグで発見したライブラリの正確なエコシステム分類。

### 引数値マッチング

レキシカルスキャナーが呼び出しサイトで引数値を検査するようになりました。関数名だけでは判断できない誤用を検出します：

|-----|---------|-----|

### cppcheck 統合

cppcheck がインストールされている場合、スキャナーはサブプロセスとして呼び出し、データフロー支援の発見項目をレキシカル結果とマージします：

- cppcheck 出力は XML レポート形式から解析
- グレースフルデグラデーション：`PATH` または `--cppcheck-path` に cppcheck が見つからない場合、レキシカルのみの結果を使用してレポートに注記
- `--cppcheck-path <PATH>` で非デフォルトのバイナリ場所を指定

### SARIF 2.1 出力

内部ビルドの全スキャンで `<project>_static_analysis.sarif` ファイルが Markdown レポートと並べて出力されます。GitHub Code Scanning、VS Code SARIF Viewer、SARIF 2.1 に対応する CI パイプラインと互換性があります。`--sarif-output <PATH>` でデフォルト出力パスを上書き可能です。

### SARIF ベースライン差分

`--sarif-baseline <PATH>` フラグは以前の SARIF 実行を受け取り、既に含まれる発見項目を抑制します。**新規**発見項目のみを報告するため、既存の問題には反応しない PR レベルの CI ゲートに最適です。マッチングはルール ID + ファイル URI + 開始行から計算した SHA-256 フィンガープリントを使用します。

### AUTOSAR arxml 依存関係解析

`.arxml` ファイルがコンポーネント間の依存関係のために完全に解析されるようになりました。3 種類の AUTOSAR 要素型を抽出します：

| 要素 | 依存関係の種類 |
|------|--------------|
| `SW-COMPONENT-PROTOTYPE` | SWC 間のコンポジション |
| `BSW-MODULE-DESCRIPTION` | BSW モジュール参照 |
| SWC 型定義要素（`APPLICATION-SW-COMPONENT-TYPE` など） | コンポーネント型宣言 |

### AUTOSAR バージョン抽出

3 つのソースを組み合わせて AUTOSAR コンポーネントの `unspecified` バージョン文字列を置き換えます：

- **`.epd` ファイル（BSW モジュール）**：`ADMIN-DATA/DOC-REVISIONS` 以下の `ECUC-MODULE-DEF/REVISION-LABEL` を含む標準 ECUC モジュール定義ファイル
- **Doxygen 形式 C/H ヘッダー（SWC ディレクトリ）**：`^\s*\*\s+SW Version\s*:\s*(\S+)` パターンにマッチし、親ディレクトリ名でグループ化
- **バージョン解決優先順位**：`.epd` → Doxygen ヘッダー → `"unspecified"`
- **エコシステム昇格**：リンカーフラグで発見した `system` エコシステムのエントリ（`-lAdc`、`-lMcu` など）が epd/Doxygen バージョンと一致した場合に `autosar` エコシステムに昇格



---


### 概要



|--------|------|

### 出力


### フォールバックモード

マニフェスト由来のコンポーネントディレクトリが見つからないがスキャンルート以下に C/C++ ソースファイルが存在する場合、スキャナーは合成した `(project_name, "C/C++") → scan_root` エントリを自動挿入し、マニフェストなしのリポジトリもスキャン可能にします。

### 内部機能ゲート

スキャナーと関連フォーマッターは `--features internal` ビルド時のみコンパイルされます。`build-all.sh` に `--internal` フラグを追加し、パブリック版とインターナル版を同時にビルドします（インターナルバイナリ名には `-internal` サフィックスを付加）。

---


### 概要


### AUTOSAR プロジェクト検出

スキャナーはメインのディレクトリウォーク前に `detect_autosar()` のプレパスを実行します。以下の 3 つのシグナルを順に確認し（最初のマッチで短絡）：

| シグナル | トリガー |
|--------|---------|
| DET-01 | 任意の深さで `.arxml` ファイルが存在する |
| DET-02 | `BSW`、`MCAL`、`RTE`、`AUTOSAR`、または `SWC` という名前のディレクトリが存在する |
| DET-03 | ルートまたは 1 階層下の CMake か Makefile にトークン `AUTOSAR_VERSION` または `AR_VERSION` が含まれる |

いずれかのシグナルが検出されると、`ScanContext.is_autosar` が `true` に設定され、AUTOSAR 分類パイプラインが自動的に有効になります。

### AUTOSAR コンポーネント分類

`classify_autosar_components()` は、検出された依存関係名をバンドルされた BSW モジュール設定（`--bsw-config <path>` で上書き可能）と照合します。マッチしたコンポーネントは `autosar` エコシステムにアップグレードされ、以下のアノテーションが付与されます：

- **`module_name`** — 例：`NvM`、`Det`、`CanIf`
- **`layer`** — 例：`BSW-Memory`、`BSW-SystemServices`、`BSW-Communication`
- **`platform`** — `Classic` または `Adaptive`

### AUTOSAR 出力 — CycloneDX

AUTOSAR コンポーネントは追加プロパティ付きの標準 CycloneDX コンポーネントとして出力されます：

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

### AUTOSAR 出力 — SPDX

AUTOSAR コンポーネントには対応する `ExternalRef OTHER` エントリが付与されます：

```
ExternalRef OTHER autosar:layer BSW-Memory
ExternalRef OTHER autosar:platform Classic
ExternalRef OTHER autosar:supplier Vector-Informatik
```

### サプライヤーマッピング（`--supplier-config`）

コンポーネント名をサプライヤー文字列にマッピングする YAML ファイルを `--supplier-config` で指定できます：

```yaml
NvM: "Vector Informatik"
CanIf: "ETAS"
Det: "In-house"
```

- マッピングされたコンポーネントは `autosar:supplier` としてサプライヤー文字列を出力
- マッピングされていないコンポーネントは `NOASSERTION` を出力
- AUTOSAR 以外のコンポーネントには影響なし
- YAML が存在しないまたは不正な場合は明確なメッセージとともにハードエラー



**SPDX — `SECURITY` ExternalRef：**
```
```

**CycloneDX — `cwes` 配列：**
```json
{
  "cwes": [1321, 94, 77]
}
```


**実装の詳細：**
- NVD レスポンスは `~/.cache/sc2sbom/nvd/` にキャッシュ（TTL は `--cache-ttl` で制御、デフォルト 24 時間）
- HTTP リクエストごとに 6 秒のレート制限（NVD パブリック API 制限）；キャッシュヒットは即時
- HTTP 失敗は stderr にログ出力してスキップ；スキャンは継続

### 単一フォーマット向け `--output`

以前は `--output <dir>` は `--format all` でのみ有効でした。v1.0.15 ではすべてのフォーマットに対応：

```bash
# SPDX JSON をファイルに書き込む（stdout の代わりに）
radeis_sc2sbom --path ./project --format spdx-json --output ./out

# CycloneDX JSON をファイルに書き込む
radeis_sc2sbom --path ./project --format cyclonedx-json --output ./out
```

単一フォーマットで `--output` を省略した場合、出力は引き続き stdout に送られます（従来の動作を維持）。

### 移行上の注意


---

## v1.0.14 - 信頼性と SBOM 品質（2026-04-24）

### 概要

v1.0.14 は、実際の C/C++ プロジェクトのスキャンでユーザーから報告されたバグを受けた信頼性と SBOM 品質のマイルストーンです。4 つのフェーズで長年の課題を解決しました：スキャナーがブロークンシンボリックリンクで中断しなくなり、Makefile 変数参照が `versionInfo` に漏れなくなり、一般的な C/C++ ライブラリが実際の SPDX ライセンス識別子に解決されるようになり、Linux リリースバイナリが静的リンクされてサポート対象ディストリビューションで glibc ドリフトなしに実行できるようになりました。以前失敗していたスキャンは、エンドツーエンドでクリーンに完了するようになりました。

### ブロークンシンボリックリンク耐性

以前は、スキャンパス配下のどこか 1 つでもブロークンシンボリックリンクがあると、走査全体が中断していました。スキャナーは警告を出して続行するようになりました：

```
Warning: skipping /path/to/broken-link: No such file or directory (os error 2)
```

5 箇所すべての WalkDir 走査箇所をカバーしています — メインのスキャナー、`main.rs` のフォールバックインポートスキャン、C/C++ パーサー（`makefile.rs`、`mk_file.rs` など）。共有の `warn_on_walkdir_err` ヘルパーを `src/util/mod.rs` に配置し、重複していた 5 つの `filter_map` クロージャを置き換えました。

### Makefile `$(VAR)` フィルタリング

Makefile フラグメントには、`DOPENSSL_VERSION := $(OPENSSL_VERSION)` のような未解決の変数参照がしばしば含まれます。本リリース以前は、これらの参照がそのまま SBOM 出力に漏れていました：

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

フィルタは `mk_file.rs` パーサー層で適用され、さらに多層防御として各バージョン出力箇所（`version_info_or_noassertion`、`generate_cpe_identifier`、`create_download_location`、`create_package_url`、CycloneDX コンポーネントバージョンフィールド）でもガードされます。`$(...)` にマッチする値は、そのまま出力される代わりに `NOASSERTION` に置き換えられます。

### C/C++ ライセンス解決

ネイティブ C/C++ 依存関係は、`Makefile` も `pkg-config` ファイルも標準的なライセンスフィールドを一貫して公開しないため、以前はデフォルトで `NOASSERTION` となっていました。v1.0.14 は 2 つの解決パスを追加します：

1. **`.pc` の `License:` 解析** — pkg-config ファイルに `License:` 行がある場合、解析されて `licenseConcluded` に昇格されます。
2. **`known_licenses.rs` テーブル** — 24 個の一般的なシステムライブラリに対する厳選された参照テーブル（openssl → Apache-2.0、zlib → Zlib、libcurl → curl、libssh2 → BSD-3-Clause、ncurses → X11 など）。`makefile.rs`、`mk_file.rs`、`pkgconfig.rs`、`pkgconfig_detector.rs` でフォールバックとして使用されます。

`openssl@3.0.7` のサンプル出力は次のようになります：

```json
{
  "name": "openssl",
  "versionInfo": "3.0.7",
  "licenseConcluded": "Apache-2.0",
  "licenseDeclared": "Apache-2.0"
}
```

両方のライセンスフィールドが `NOASSERTION` になる代わりに。

### musl による静的 Linux バイナリ

Linux リリースバイナリは `x86_64-unknown-linux-gnu` ではなく `x86_64-unknown-linux-musl` をターゲットにするようになりました。以前はビルドホストからの GLIBC 2.39 依存を抱えており、Ubuntu 22.04 上で `version GLIBC_2.39 not found` となり動作しませんでした。musl リンクされた静的バイナリは glibc 依存がなく、Ubuntu 22.04+、24.04、Alpine、Debian、RHEL など、あらゆる x86_64 Linux で実行できます。`build-all.sh` に musl クロスリンカーツールチェーンガードを追加し、CI の Ubuntu ランナーは `musl-tools` を提供するよう構成されています。

### 移行上の注意

CLI や出力スキーマの変更はありません。v1.0.14 で再生成された既存の SBOM は、C/C++ コンポーネントで追加のライセンス結論フィールドが表示され、`$(VAR)` 形式のバージョン文字列が減少する可能性がありますが、いずれも厳密な改善です。以前は glibc リンクバイナリから移行できなかった Linux ユーザーは、他に変更を加えずに v1.0.14 musl バイナリを配置するだけで済みます。

### テスト対象

4 つのバグすべての元となった実際の C/C++ プロジェクトに対して検証済みです。スキャンは想定されるブロークンシンボリックリンク通知以外の警告なしに完了まで実行され、出力される SPDX/CycloneDX はスキーマ検証を通過します。

設計の詳細は [v1.0.14 計画](../plan/v1.0.14_user_reported_bugfixes.md) を参照してください。

---

## v1.0.13 - マルチモーダルサブモデルコンポーネント（2026-04-14）

### 概要

v1.0.13 はマルチモーダル AI モデルを構成サブモデルコンポーネントに分解します。`google/gemma-4-E2B-it` のようなモデルは、テキスト、ビジョン、オーディオの個別サブモデルを含む親モデルとして表現され、各サブモデルが SBOM 内に独自のアーキテクチャメタデータを持ちます。

### サブモデル分解

`config.json` 内に `text_config` と `vision_config` および/または `audio_config` を持つマルチモーダルモデルは自動的に分解されます：

```
gemma4 (parent — Gemma4ForConditionalGeneration)
├── gemma4_text  (35 layers, 1536 hidden, 8 heads, 262K vocab, 131K context)
├── gemma4_vision (16 layers, 768 hidden, 12 heads, patch_size=16)
└── gemma4_audio  (12 layers, 1024 hidden, 8 heads, conv_kernel=5)
```

### 出力形式サポート

- **CycloneDX** — 親コンポーネント内にネストされた `components` 配列、各サブモデルは `machine-learning-model` として `radeis:ai:sub_model:*` プロパティ付き
- **SPDX** — サブモデルパッケージと親モデル間を `CONTAINS` 包含関係で接続
- **コンソール** — サブモデルサマリーテーブル（モダリティ、モデルタイプ、レイヤー数、隠れ層サイズ、ヘッド数、dtype、モダリティ固有の追加情報を表示）

### ガード条件

サブモデルは真にマルチモーダルなモデルに対してのみ生成されます。`text_config` のみを持つテキスト専用モデル（`vision_config` や `audio_config` がない場合）は、不要なサブモデルエントリを生成しません。

### テストカバレッジ

- 4 件の新規 Safetensors テスト：マルチモーダル抽出、テキスト専用ガード、text_config なしガード、ビジョン+テキストのみ（LLaVA スタイル）
- 1 件の新規 GGUF テスト：コンパニオン config.json によるサブモデルエンリッチメント

設計の詳細は [v1.0.13 計画](../plan/v1.0.13_multimodal_sub_model_components.md) を参照してください。

---

## v1.0.12 - Safetensors リッチメタデータ（2026-04-14）

### 概要

v1.0.12 は、すべての HuggingFace コンパニオン設定ファイルからリッチメタデータを抽出することで、GGUF と Safetensors の SBOM 品質のメタデータギャップを解消します。Safetensors および GGUF モデルディレクトリの両方で、アーキテクチャの深層情報、推論デフォルト、マルチモーダル機能、来歴、アダプター検出を網羅した詳細な SBOM を生成するようになりました。

### コンパニオンファイルからのリッチメタデータ

- **`config.json` 拡張** — model_type、text_config（hidden_layers、hidden_size、attention_heads、コンテキストウィンドウ）、マルチモーダル検出（vision_config、audio_config）、dtype フォールバックチェーン
- **`generation_config.json`** — temperature、top_k、top_p 推論パラメータ
- **`tokenizer_config.json`** — processor_class、model_max_length（天文学的な値は安全にキャップ）
- **`preprocessor_config.json`** — 画像、オーディオ、ビデオプロセッサタイプとシーケンス長、サンプリングレート
- **`README.md` フロントマター** — base_model（文字列とリスト両方に対応）、license、model_creator、pipeline_tag、quantized_by、prompt_template、tags、languages、datasets
- **`adapter_config.json`** — ベースモデル参照付きの LoRA/QLoRA アダプター検出

### GGUF コンパニオンファイルエンリッチメント

GGUF リポジトリでも同じコンパニオンファイル解析が利用可能になりました。バイナリ KV メタデータが常に優先され、コンパニオンファイルはギャップを埋めるためにのみ使用されます。tags、languages、datasets は両ソースからの重複排除されたユニオンマージを使用します。

### 出力の強化

- **CycloneDX** — アーキテクチャ、マルチモーダル、生成、プロセッサ、来歴、アダプターメタデータをカバーする約 25 個の新しい `radeis:ai:*` プロパティ
- **SPDX** — model_type、コンテキストウィンドウ、モダリティサマリーを含む sourceInfo の拡張（簡潔さを維持）
- **コンソール** — AI Model Details テーブルにアーキテクチャ、マルチモーダル、生成パラメータ、来歴の行を追加

### 安全性とエッジケース

- すべてのコンパニオンファイル読み込みに 1 MB の上限を設定し、メモリ問題を防止
- README.md ファイル名の大文字小文字を区別しないマッチング（README.md、readme.md、Readme.md）
- Windows 生成ファイル向けの CRLF 改行正規化
- 天文学的な model_max_length 値（例：Gemma-4 の 1e30）を安全に破棄
- prompt_template を 512 文字（保存）および 80 文字（コンソール表示）に制限

### テストカバレッジ

- `safetensors_tests.rs` に 8 件の新規テスト — コンパニオンファイル解析、マルチモーダル検出、dtype フォールバックチェーン、model_max_length キャップ
- `gguf_tests.rs` に 6 件の新規テスト — config.json 抽出、README フロントマター（文字列とリストの base_model、CRLF、小文字ファイル名）、アダプター検出、tags ユニオンマージ

設計の詳細は [v1.0.12 計画](../plan/v1.0.12_safetensors_rich_metadata.md) を参照してください。

---

## v1.0.11 - Safetensors AI モデル SBOM（2026-04-13）

### 概要

v1.0.11 では Safetensors AI モデルサポートを追加し、`radeis_sc2sbom` を現代のトランスフォーマーモデル（LLaMA、Mistral、Falcon など）の主流フォーマットに対応させました。モデルはディレクトリレベルでスキャンされ、シャード数に関わらず、モデルごとに単一の Dependency エントリが出力されます。総サイズ、dtype、アーキテクチャのメタデータも正確に反映されます。

### Safetensors AI モデルサポート

- **ファイル検出** — `.safetensors`、`model.safetensors.index.json`、`config.json` からメタデータをスキャン
- **ディレクトリレベルの重複排除** — マルチシャードモデル（例：`model-00001-of-00002.safetensors`）をモデルディレクトリごとに 1 件の SBOM エントリに集約
- **CycloneDX 出力** — アーキテクチャ、dtype、サイズメタデータを含む `modelCard` 付きの `machine-learning-model` コンポーネントタイプを使用
- **SPDX 出力** — HuggingFace エコシステムでのモデル識別のために `pkg:huggingface` PURL を生成
- **新しい `AIModelMetadata` フィールド**：`safetensors_format`、`total_size_bytes`、`shard_count`、`torch_dtype`、`transformers_version`、`vocab_size`

### テストカバレッジ

- `tests/parser_tests/safetensors_tests.rs` に 12 件の新規テストを追加（シングルシャード、マルチシャード、インデックスベース、config.json 駆動シナリオ）

---

## v1.0.10 - Java Complete（2026-04-13）

### 概要

v1.0.10 は Gradle を検出のみから完全な依存関係解析に変換し、Groovy DSL（`build.gradle`）と Kotlin DSL（`build.gradle.kts`）の両方をサポートします。これは、Android ベースのロボティクスコントローラーや Android デバイスでのエッジ AI 推論を使用する Physical AI プロジェクトにとって重要です。

### Gradle 依存関係解析

- **Groovy DSL**（`build.gradle`）— 文字列記法（`'group:artifact:version'`）、マップ記法（`group: 'g', name: 'a', version: 'v'`）、プラットフォーム/BOM 宣言を解析
- **Kotlin DSL**（`build.gradle.kts`）— 関数記法（`implementation("group:artifact:version")`）とプラットフォーム BOM を解析
- **スコープ分類** — `testImplementation` → テスト、`compileOnly` → 提供済み、`annotationProcessor`/`kapt`/`ksp` → ビルド、信頼度 1.0
- **Android サポート** — `androidTestImplementation` と `androidTestCompile` がテスト依存関係として正しく分類
- **PURL 形式** — `pkg:maven/{group}/{artifact}@{version}`（Maven と同じ）

詳しくは [v1.0.10 計画](../plan/v1.0.10_gradle_support.md) をご覧ください。

---

## v1.0.9 - Physical AI Ready（2026-04-10）

### 概要

v1.0.9 では GGUF AI モデルサポートを追加し、`radeis_sc2sbom` を AI モデルバイナリのネイティブ解析と整合性検証を備えた初の SBOM ジェネレーターにしました。また、C/C++ ビルドシステムフラグの統合により CLI を簡素化しています。

### GGUF AI モデルサポート

- **バイナリパーサー**が `.gguf` ファイルからメタデータを直接抽出 — アーキテクチャ、量子化タイプ、テンソルレイアウト、コンテキスト長、組み込みライセンス情報
- **CycloneDX 出力**は `machine-learning-model` コンポーネントタイプを使用し、トレーニングパラメータとデータセット参照を含む `modelCard` を付与
- **SPDX 出力**は Hugging Face エコシステムでのモデル識別のために `pkg:huggingface` PURL を生成
- `--scan-ai-models true` で有効化

### AI モデル整合性検証

- **テンソルパラメータのクロスバリデーション** — 宣言されたテンソル数と次元を実際のバイナリレイアウトと照合し、切り詰められたまたは破損したモデルファイルを検出
- **SHA-256 ハッシュ** — 各モデルファイルにコンテンツハッシュを生成し、サプライチェーンの真正性検証に使用

### CLI の簡素化

- 5 つの個別 C/C++ フラグ（`--scan-cmake`、`--scan-pkgconfig`、`--scan-autotools`、`--scan-makefiles`、`--scan-mk-files`）を `--scan-c-build-systems` に統合
- `--meson-parse-subprojects` を削除（`--scan-meson` 有効時は常にオン）
- `--resolve-system-deps` を削除（デッドコード）
- `scan_directory()` の引数を 20 個から 13 個に削減

設計の詳細は [v1.0.9 計画](../plan/v1.0.9_physical_ai_ready.md) を参照してください。

---

## v1.0.7 - 脆弱性スキャンのオプトイン化（2026-03-12）

### 概要

脆弱性スキャンはオプトイン方式になりました。デフォルトでは `radeis_sc2sbom` はネットワーク通信を一切行わず、クリーンな SBOM を生成します。これにより、ツールの実行速度が向上し、オフライン／エアギャップ環境や依存関係一覧のみを必要とする CI/CD パイプラインに適した構成になっています。

### 変更内容

  - 以前は脆弱性スキャンが自動的に実行されていました
- **デフォルト出力のクリーン化**
  - 脆弱性サマリー行をコンソール出力から非表示
  - リスク評価セクションを Markdown レポートから除外

### 移行手順

```bash
# v1.0.7 以前（脆弱性スキャンが自動実行）
./radeis_sc2sbom --path .

# v1.0.7+ 脆弱性スキャン（明示的なオプトイン）

# v1.0.7+ CI/CD：高危険度／重大な脆弱性検出時に終了
./radeis_sc2sbom --path . \
```

---

## v1.0.6 - 本番環境向け SBOM フィルタリングと依存関係スコープの自動分類（2026-03-04）

### 概要

依存関係スコープの自動分類機能を導入し、実行時に実際に必要なパッケージのみを含む本番対応 SBOM を生成できるようになりました。新しい `--production` フラグと `--scope-filter` オプションにより、精度を落とすことなく SBOM のサイズを大幅に削減できます。

### 主な機能

#### スコープの自動分類
- **6 種類のスコープ**：Runtime、Build、Test、Development、Optional、Provided
- **複合ヒューリスティックエンジン**：エコシステム・名前・ディレクトリに基づくルールを組み合わせて適用
- **信頼度スコア**：0.0〜1.0 の範囲で、分類理由を人間が読める形式で付与
- **10 以上のエコシステムに対応**：npm、pip、cargo、SYSTEM、BUILD-CONFIG、GIT-SUBMODULE、MESON-WRAP など

#### 本番環境フィルタリング
- **`--production`** フラグ：Runtime と Optional の依存関係のみを含める
  - 例：組み込みプロジェクトが 106 パッケージ → 33 パッケージに削減（68.9% 削減）
- **`--scope-filter <SCOPE>`**：任意のスコープタイプの組み合わせを選択
  - 複数指定可：`--scope-filter runtime --scope-filter optional`
- **スコープ統計**をコンソールおよび Markdown レポートに表示（スコープ別の件数と割合）

#### 強化された SBOM 出力
- **SPDX 2.3**：スコープ分類から `primaryPackagePurpose` フィールドを設定
- **CycloneDX 1.5**：スコープ分類から `scope` フィールドを設定

#### 検証済みの分類精度
| カテゴリ | 精度 | 例 |
|---|---|---|
| ビルドツール | 100% | cmake、gcc、ninja、meson |
| テストフレームワーク | 100% | pytest、jest、gtest、unity |
| 開発ツール | 100% | pylint、black、eslint、prettier |
| ランタイムライブラリ | 高 | zlib、curl、openssl、protobuf |

### テストカバレッジ
- **609 件のテストが全て通過**（203 lib + 203 bin + 200 統合 + 3 ドキュメント）
- **42 件の新規統合テスト**：スコープフィルタリング、本番モード、実プロジェクト分類を網羅

### 後方互換性
- デフォルトの動作は v1.0.5 から変更なし——スコープ分類は自動実行されますが、`--production` または `--scope-filter` を指定しない限り出力はフィルタリングされません。

---

## v1.0.5 - バージョン抽出の強化：デュアルモード .mk ファイルスキャン（2026-02-26）

### 概要

Makefile から検出されたシステム／ランタイムライブラリで発生していた「バージョン未指定」問題を解消し、ビルドシステムリポジトリに対する独立した .mk ファイルマニフェスト解析も実現しました。**インテリジェントなデュアルモードアーキテクチャ**により、設定不要で異なるリポジトリ構造に自動的に対応します。本リリースでは、デュアル動作モードを備えた包括的な .mk ファイル解析と、ビルド後のバージョン抽出に向けた任意の .so バイナリスキャンが追加されています。

### 問題の背景

**v1.0.5 以前：**
```json
{
  "name": "z",
  "version": "unspecified",  // ❌ バージョン情報なし
  "ecosystem": "system"
}
```

**v1.0.5 以降（.mk ファイルスキャン有効時）：**
```json
{
  "name": "z",
  "version": "1.3.1",  // ✅ zlib.mk から抽出
  "ecosystem": "system",
  "source_file": "... [version from .mk file: 1.3.1]"
}
```

### 主な機能

#### デュアルモードアーキテクチャ

.mk ファイルスキャナーはリポジトリの構造に基づいて適切なモードを自動選択します：

**モード 1：バージョン解決**（アプリケーションプロジェクト）
- **トリガー：** `-l` フラグでシステムライブラリを検出する Makefile が存在する場合
- **処理内容：** .mk ファイルから Makefile 検出済みライブラリのバージョンを解決
- **エコシステム：** "system"（Makefile 検出のコンテキストを維持）
- **例：** embedded-project——16 個のシステムライブラリが「unspecified」から正確なバージョンに更新
- **用途：** システムライブラリにリンクするアプリケーションプロジェクト

**モード 2：独立マニフェスト解析**（ビルドシステムリポジトリ）
- **トリガー：** Makefile なしで .mk ファイルが存在する場合（または Makefile にライブラリが検出されない場合）
- **処理内容：** .mk ファイル内の全 `*_VERSION` 変数から直接依存関係を生成
- **エコシステム：** "BUILD-CONFIG"（ビルドシステムのソースコードを示す）
- **例：** embedded-toolchains プロジェクト——35 件の BUILD-CONFIG パッケージを検出、バージョンカバレッジ 100%
- **用途：** ビルド時依存関係を定義するビルドシステムリポジトリ

**自動重複排除：**
- 両方のモードが同じライブラリを検出した場合、モード 1（"system"）がモード 2（"BUILD-CONFIG"）より優先されます
- 正確な依存関係分類を維持しながら重複エントリーを防止
- `deduplicate_dependencies()` 関数にエコシステム対応ロジックを実装

#### .mk ファイル解析

組み込みシステムで一般的なビルド設定ファイルからバージョン情報を抽出します。

**探索戦略：** glob パターン `**/*.mk` を使用して、ディレクトリ構造に依存せずリポジトリの**任意の場所**にある .mk ファイルを検索します。

**.mk ファイルの例：**
```makefile
# toolchains/3rd_party/curl/curl.mk（または任意の場所）
CURL_VERSION ?= 8.15.0
CURL_NAME := curl-$(CURL_VERSION)
LIBCURL_SO := $(LIBCURL).4.8.0
```

**マッピング戦略：**
1. Makefile からライブラリを検出：`-lcurl` → ライブラリ名："curl"
2. glob で .mk ファイルを検索：`**/*.mk`（リポジトリ内の任意の場所で curl.mk を検索）
3. 全 .mk ファイルを解析して `CURL_VERSION ?= 8.15.0` を抽出
4. バージョン変数をライブラリにマップ：`CURL_VERSION` → "curl" → "8.15.0"
5. 依存関係バージョンを更新："curl" @ "8.15.0"

**対応パターン：**
- `VAR_VERSION ?= value`（条件付き代入）
- `VAR_VERSION := value`（単純代入）
- `VAR_VERSION = value`（再帰代入）

**ライブラリ名の正規化：**（モード 1 のみ——Makefile の `-l` フラグ解決用）
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
- 汎用：`foo` → `libfoo`（両方の形式を試行）

**ビルドツールフィルタリング：**（モード 2 のみ——誤検知防止）
- ビルドツールを除外：make、cmake、gcc、clang、python、perl、ruby、autoconf、automake、libtool、ninja、meson、bash、sh、awk、sed
- SBOM に "make@4.3" や "cmake@3.25.0" などの誤検知依存関係が出現することを防止

#### .so バイナリスキャン（優先度 2）

ビルド済みライブラリバイナリからバージョンを抽出します（ビルド後アプローチ）。

**技術手法：**
1. **.so ファイル名の解析：** `libcurl.so.4.8.0` → バージョン "4.8.0"
2. **ELF soname の読み取り：** `readelf -d libcurl.so | grep SONAME`（readelf が利用可能な場合）
3. **バージョン文字列の抽出：** バイナリ内容からバージョンパターンを検索

**検索ディレクトリ：**
- `lib/`
- `lib64/`（64 ビットライブラリ）
- `build/`
- `build/lib/`（CMake アウトオブツリービルド）
- `toolchains/install/lib/`
- `usr/lib/`
- `usr/lib64/`（64 ビットシステムライブラリ）
- `usr/local/lib/`（ローカルインストール）
- `.libs/`（autotools）

**シンボリックリンクの重複排除：**
- シンボリックリンクチェーンを自動解決（例：`libcurl.so` → `libcurl.so.4` → `libcurl.so.4.8.0`）
- 標準パスを使用して同一ライブラリの重複スキャンを防止
- シンボリックリンクされたライブラリ間でのバージョンレポートの一貫性を確保

**制限：** ライブラリのビルドが完了している必要があります（ソースのみのリポジトリには適しません）。

### CLI フラグ

```bash
# .mk ファイルのバージョン抽出（デフォルトで有効）
--scan-mk-files=true/false     # デフォルト：true

# .so バイナリのバージョン抽出（デフォルト無効、ビルド済みライブラリが必要）
--scan-so-files=true/false     # デフォルト：false
```

**デフォルト設定の理由：**
- `--scan-mk-files=true`：ソースのみのリポジトリでも安全、オーバーヘッドが低く、任意の .mk ファイルの場所に対応
- `--scan-so-files=false`：ビルド済みライブラリが必要なため、CI/CD では存在しない場合がある

### 実際の効果

**embedded-project プロジェクト：**
- **v1.0.5 以前：** 32 個のシステムライブラリが "unspecified" バージョン
- **v1.0.5 以降：** 32 個のシステムライブラリが .mk ファイルから正確なバージョンを取得
  - curl @ 8.15.0
  - elfutils @ 0.191
  - zlib @ 1.3.1
  - openssl @ 3.2.5
  - libssh2 @ 1.11.0
  - 他 27 件……

### 後方互換性

✅ **100% 後方互換**
- .mk ファイルのない既存プロジェクト：動作変更なし
- デフォルトの `--scan-mk-files=true` は Makefile ベースのプロジェクトにのみ影響
- .mk ファイルのないプロジェクトは引き続き "unspecified" バージョンを表示
- バージョン解決は追加的：既存バージョンは上書きされない

### テスト

- **154 件のユニットテストが通過**——既存テスト + 新規 .mk・.so パーサーテストを含む
- **3 件の新規統合テスト**——モード 1／モード 2 の重複排除、ビルドツールフィルタリング、マルチファイルシナリオ
- **包括的なコードレビュー**——全課題に対応（重複排除ロジック、シンボリックリンク処理、ディレクトリ探索の最適化）
- **統合テスト済み**：embedded-project および embedded-toolchains プロジェクト構造
- **既存パーサーへの回帰なし**

### 変更ファイル

**新規ファイル：**
- `src/parsers/c/mk_file.rs` - バージョン抽出を備えた .mk ファイルパーサー
- `src/parsers/c/so_scanner.rs` - バージョン抽出を備えた .so バイナリスキャナー

**変更ファイル：**
- `src/parsers/c/makefile.rs` - モード 1 用のバージョン解決ロジックを追加
- `src/parsers/mod.rs` - モード 1／モード 2 用のエコシステム対応重複排除を追加
- `src/cli.rs` - `--scan-mk-files` と `--scan-so-files` フラグを追加
- `src/scanner/mod.rs` - 新フラグを Makefile パーサーに渡す + モード 2 のトリガー
- `src/main.rs` - CLI フラグをスキャナーに接続
- `Cargo.toml` - .mk ファイル探索用に `glob` 依存関係を追加
- `tests/parser_tests/c_tests.rs` - 重複排除とフィルタリングの統合テストを追加

### 今後の拡張予定（v1.0.6+）

1. **スマート .mk パターン検出** - 複数プロジェクトから .mk ファイルパターンを学習
2. **pkg-config .pc 生成** - .mk ファイルから他ツール向けの .pc ファイルを生成
3. **ビルドシステムプラグインアーキテクチャ** - プラグインによるカスタムビルドシステムのサポート
4. **バージョン制約の解決** - ">=3.0" などの制約を実際のバージョンに解決
5. **.mk ファイルからのライセンス抽出** - ビルド設定からライセンス情報を抽出

---

## v1.0.4 - Meson・Bazel ビルドシステムのサポート（2026-02-25）

### 概要

最新の C/C++ ビルドシステム（Meson・Bazel）のサポートを追加し、radeis_sc2sbom の C/C++ エコシステムへの包括的な対応を完成させました。v1.0.0〜1.0.3（vcpkg、Conan、CMake、Git サブモジュール、Autotools、pkg-config、Makefile）と合わせて、radeis は **C/C++ プロジェクトの約 95%** をカバーします。

### 主な機能

#### Meson ビルドシステムのサポート

`meson.build` ファイルの `dependency()` 宣言を解析します：

```python
# meson.build の例
project('myapp', 'cpp')

# バージョン制約付きの dependency()
zlib_dep = dependency('zlib', version: '>=1.2.11')

# バージョンなしの dependency()
openssl_dep = dependency('openssl')

# find_library() によるシステムライブラリ
cc = meson.get_compiler('c')
math_dep = cc.find_library('m')

# サブプロジェクト参照
libfoo_proj = subproject('libfoo')

executable('myapp', 'src/main.cpp',
  dependencies: [zlib_dep, openssl_dep, math_dep])
```

**対応機能：**
- `dependency()` 宣言からライブラリ名を抽出
- `version:` 引数からバージョン制約を抽出（存在する場合）（>=、==、>、<、!=）
- `cc.find_library()` 呼び出しからシステムライブラリを抽出
- `subproject()` 呼び出しからサブプロジェクト参照を検出（実際の解決は `.wrap` ファイル経由）
- PURL 形式：`pkg:generic/{name}@{version}?type=meson`（バージョンがある場合は含める）

**注意：** パーサーは現在、依存関係名とバージョン制約を抽出します。`modules:` 配列と `required:` フラグは構文的に認識されますが、現時点では構造化出力にキャプチャされません。

**実プロジェクトでの検証：**
- OpenStudio プロジェクトは conan.lock で meson 1.2.2 を開発依存関係として使用
- 本番スキャンで正常に検出・検証済み
- Meson に移行中の C++ プロジェクトへの即時適用性を実証

#### Bazel ビルドシステムのサポート

`WORKSPACE`/`WORKSPACE.bazel` および `MODULE.bazel` ファイルの外部依存関係を解析します：

```python
# WORKSPACE の例
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

# MODULE.bazel の例（Bazel 6.0+ bzlmod）
module(name = "myproject")

bazel_dep(name = "abseil-cpp", version = "20230802.1")
bazel_dep(name = "googletest", version = "1.14.0")
```

**対応機能：**
- `WORKSPACE`/`WORKSPACE.bazel` の `http_archive`、`git_repository`、ローカルリポジトリを解析
- `MODULE.bazel`（Bazel 6.0+ bzlmod）の `bazel_dep()` 宣言を解析
- 外部依存関係宣言から URL とバージョンを抽出
- 複数行の依存関係宣言に対応
- PURL 形式：`pkg:generic/{name}@{version}?type=bazel`、git ベースの依存関係には `pkg:github/{owner}/{repo}@{version}?type=bazel`

### CLI フラグ

```bash
# Meson・Bazel サポートの有効化／無効化（デフォルト両方有効）
--scan-meson=true/false        # meson.build と .wrap ファイル
--scan-bazel=true/false        # WORKSPACE、WORKSPACE.bazel、MODULE.bazel ファイル
```

### カバレッジへの影響

**v1.0.4 以前：**
- 最新の C++（vcpkg、Conan、CMake）：約 70〜80%
- 旧来の C（Autotools、pkg-config、Makefile）：約 80〜90%
- 合計カバレッジ：**C/C++ プロジェクトの約 90%**

**v1.0.4 以降：**
- 最新の C++（vcpkg、Conan、CMake、Meson、Bazel）：約 75〜85%
- 旧来の C（Autotools、pkg-config、Makefile）：約 80〜90%
- 合計カバレッジ：**C/C++ プロジェクトの約 95%**

### 実プロジェクトでのサポート

検証済みプロジェクト：
- **OpenStudio**（Conan）——meson 1.2.2 を開発依存関係として検出
- **ユニットテスト**——105 件のテスト通過（Meson・Bazel パーサー）

### 本番環境での検証

**OpenStudio スキャン結果（v1.0.4）：**
- 合計 49 パッケージ（Conan 48 件 + Python 1 件）
- conan.lock で meson 1.2.2 を開発依存関係として検出
- セキュリティスキャン結果はクリーン（脆弱性 0 件）
- 開発依存関係の分類を含む conan.lock の完全解析

これにより、v1.0.4 の Meson サポートが最新ビルドシステムを採用した実際の C++ プロジェクトに即座に活用できることが実証されました。

### 後方互換性

✅ **100% 維持** - 既存エコシステムパーサーへの回帰なし：
- 全 105 件のユニットテスト通過
- curl の結果は v1.0.3 と同一（確認済み）
- npm、Python、ROS、Conan、Git サブモジュール、CMake、Autotools はすべて安定
- Meson・Bazel パーサーは追加機能のみ

### 包括的な比較レポート

v1.0.4 の検証の一環として、6 つの多様なリポジトリに対する包括的な比較レポート（各 375〜596 行）を完成させました：

1. **curl**（C ライブラリ）- 446 行
2. **nodejs-service**（Node.js）- 444 行
3. **nodejs-project**（マルチクラウド）- 375 行
4. **OpenStudio**（C++ Conan）- 398 行
5. **mrpt**（ロボティクス C++）- 596 行
6. **ros2cli**（ROS 2）- 590 行

**合計：** 2,897 行の包括的な分析

**主な発見：**
- **全プロジェクト合計で 2,561 件の依存関係を追跡**
- **他のツールにはない 4 つの固有機能：**
  - C/C++ 非対応の Autotools（curl：29 ライブラリ）
  - ROS 2（ros2cli：223 コンポーネント）
  - Git サブモジュール（mrpt：SHA 付き 8 サブモジュール）
  - CMake ExternalProject（mrpt：3 依存関係）
- **BlackDuck と比較して 3 年間で 22 万〜165 万ドルのコスト削減**

詳細は [scan_reports/COMPARISON_REPORTS_INDEX.md](../scan_reports/COMPARISON_REPORTS_INDEX.md) をご参照ください。

---

## v1.0.3 - C レガシーサポート（pkg-config + Autotools + Makefile）（2026-02-24）

### 概要

従来の C プロジェクトビルドシステムへの包括的なサポートを追加し、GNU Autotools、pkg-config、および純粋な Makefile を使用するレガシー C/C++ プロジェクトの SBOM 生成を可能にします。現代的なパッケージマネージャーのサポート（vcpkg、Conan、CMake）では対応できなかった重要な空白を埋め、C/C++ プロジェクトの約 90% をカバーします。

### 主な機能

#### pkg-config（.pc ファイル）のサポート

システムライブラリの依存関係を抽出するために `.pc`（pkg-config）ファイルを解析します：

```
Name: OpenSSL
Version: 3.0.2
Description: Secure Sockets Layer and cryptography libraries
Requires: libcrypto libssl
```

**対応機能：**
- .pc ファイルからパッケージ名、バージョン、説明を抽出
- configure.ac 内の PKG_CHECK_MODULES() 呼び出しを検出
- Makefile 内の pkg-config シェル呼び出しを検出
- PURL 形式：`pkg:generic/{name}@{version}?type=pkg-config`

#### Autotools（configure.ac/Makefile.am）のサポート

GNU Autotools 設定ファイルからライブラリの依存関係を解析します：

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

**対応機能：**
- AC_CHECK_LIB、AC_SEARCH_LIBS、PKG_CHECK_MODULES から依存関係を抽出
- Makefile.am の LDADD/LIBADD 変数から -l フラグを抽出
- PKG_CHECK_MODULES のバージョン制約を保持
- PURL 形式：`pkg:generic/{name}@{version}?type=autotools`

#### 純粋な Makefile のヒューリスティックパーサー

手書き Makefile のベストエフォート解析：

```makefile
LDFLAGS = -lssl -lcrypto -lpthread -lz
OPENSSL_CFLAGS = $(shell pkg-config --cflags openssl)
```

**対応機能：**
- 正規表現を使用して -l フラグ（システムライブラリ）を抽出
- pkg-config 呼び出しを検出
- ライブラリ名による重複排除
- PURL 形式：`pkg:generic/{name}@{version}?type=makefile`

**制限：**
- 変数展開（`$(FOO)`）は非対応
- 条件ブロック（`ifeq`）は非対応
- Autotools プロジェクトでは Makefile 解析をスキップ

### CLI フラグ

```bash
# C レガシーサポートの有効化／無効化（デフォルト全て有効）
--scan-pkgconfig=true/false       # .pc ファイルと PKG_CHECK_MODULES
--scan-autotools=true/false       # configure.ac と Makefile.am
--scan-makefiles=true/false       # 純粋な Makefile（ヒューリスティック）
--resolve-system-deps=false       # システム pkg-config 解決（デフォルト無効）
```

### カバレッジへの影響

**v1.0.3 以前：**
- 最新の C++（vcpkg、Conan、CMake）：約 70〜80%
- 旧来の C（Autotools）：1% 未満
- システムライブラリ：1% 未満

**v1.0.3 以降：**
- 最新の C++（vcpkg、Conan、CMake）：約 70〜80%
- 旧来の C++（Makefile）：約 40%
- 純粋な C（Autotools）：約 60%
- システムライブラリの依存関係：約 80%

**合計カバレッジ：C/C++ プロジェクトの約 90%**

### 実プロジェクトでのサポート

検証済みプロジェクト：
- **curl**（Autotools）——openssl、zlib、nghttp2 を検出
- **nginx**（Makefile）——openssl、pcre、zlib を検出
- 標準 C ライブラリ（pthread、m、ssl、crypto、z）

### 出力例

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

### PURL の例

- **pkg-config**：`pkg:generic/openssl@3.0.2?type=pkg-config`
- **Autotools**：`pkg:generic/pthread@unspecified?type=autotools`
- **Makefile**：`pkg:generic/ssl@unspecified?type=makefile`

---

## v1.0.2 - Conan C++ パッケージマネージャーのサポート（2026-02-24）

### 概要

Conan C/C++ パッケージマネージャーへの完全なサポートを追加し、Conan 2.x を使用するプロジェクトの SBOM 生成を可能にします。ロックファイル、INI 形式マニフェスト、Python 形式マニフェストを解析し、ランタイム・ビルド・ツール・テストの各依存関係に対応します。

### 主な機能

#### Conan ロックファイルの解析（conan.lock）

正確なバージョンとレシピリビジョンを含む Conan 2.x ロックファイルを解析します：

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

**対応機能：**
- 参照からパッケージ名とバージョンを抽出
- レシピリビジョンハッシュをチェックサムとして保存
- ランタイム・ビルド・ツール・テスト依存関係を区別
- 同一ディレクトリ内でロックファイルがマニフェストより優先

#### Conan マニフェストの解析（conanfile.txt）

INI 形式の Conan マニフェストを解析します：

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

**対応機能：**
- バージョン制約：`[>=3.0]`、`[>1.0 <2.0]`、`[~=1.82]`、`[^1.0]`
- ユーザー／チャンネル表記：`package/version@user/channel`
- ビルド・ツール・テスト依存関係を開発依存関係としてマーク

#### Conan Python マニフェストの解析（conanfile.py）

正規表現抽出を使用して Python 形式の Conan マニフェストを解析します：

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

**対応パターン：**
- リスト形式：`requires = ["dep1", "dep2"]`
- メソッド呼び出し：`self.requires("dep")`
- ビルド依存関係：`build_requires`、`self.build_requires()`
- ツール依存関係：`tool_requires`、`self.tool_requires()`
- テスト依存関係：`test_requires`、`self.test_requires()`

#### SBOM 出力

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

### 技術実装

**追加ファイル：**
- [src/parsers/cpp/conan.rs](../src/parsers/cpp/conan.rs) - conan.lock パーサー
- [src/parsers/cpp/conan_manifest.rs](../src/parsers/cpp/conan_manifest.rs) - conanfile.txt/py パーサー
- [tests/parser_tests/conan_tests.rs](../tests/parser_tests/conan_tests.rs) - 10 件のテストケース

**統合：**
- スキャナーが `conan.lock`、`conanfile.txt`、`conanfile.py` を検出
- ロックファイル優先：`conan.lock` が存在する場合はマニフェストをスキップ
- SPDX purl 形式：`pkg:conan/{name}@{version}`
- CycloneDX コンポーネントに対応

### テストカバレッジ

```bash
cargo test conan_tests
```

**10 件のテストケースが以下をカバー：**
- レシピリビジョン付きロックファイルの解析
- バージョン制約付き INI マニフェストの解析
- Python マニフェストの解析（リスト形式とメソッド形式）
- バージョン範囲の処理
- ユーザー／チャンネル表記
- 不正な入力の処理
- 空マニフェストの処理
- purl 形式の生成

### 使用例

```bash
# Conan 依存関係を持つプロジェクトをスキャン
./target/release/radeis_sc2sbom --path /path/to/conan-project --format spdx-json

# 出力例
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

### 設計上の決定

1. **ロックファイル優先**：`conan.lock` は正確なバージョンを提供するため、マニフェストより優先
2. **レシピリビジョン**：トレーサビリティのために `checksum_sha256` フィールドに保存
3. **開発依存関係**：ビルド・ツール・テスト依存関係は `is_dev: true` でマーク
4. **バージョン制約**：マニフェストからそのまま保持（例：`>=3.0`）
5. **エコシステム識別子**：全 Conan 依存関係で `"conan"` を使用

---

## v1.0.1 - CMake サポートと再帰的サブモジュールスキャン（2026-02-23）

### 概要

静的な CMake 依存関係検出（FetchContent/ExternalProject）と Git サブモジュール内の再帰的依存関係スキャンを追加しました。CMake のビルド実行を必要とせずに完全な依存関係の発見が可能になります。

### 主な機能

#### CMake 依存関係の解析

CMakeLists.txt ファイルを静的に解析して FetchContent_Declare と ExternalProject_Add の依存関係を抽出します：

```cmake
# FetchContent_Declare（最新の CMake 3.11+）
FetchContent_Declare(
  json
  GIT_REPOSITORY https://github.com/nlohmann/json.git
  GIT_TAG        v3.11.2
)

# ExternalProject_Add（旧来のパターン）
ExternalProject_Add(
  zlib
  URL https://zlib.net/zlib-1.2.13.tar.gz
  URL_HASH SHA256=abc123...
)
```

**対応機能：**
- GIT_REPOSITORY + GIT_TAG の抽出
- URL ベースの依存関係と URL からのバージョン抽出
- URL_HASH（SHA256）チェックサムの抽出
- CMake 変数 `${VAR}` を含むエントリーをスキップ（静的解析では解決不可）
- 複数ホストの Git URL 解析（GitHub、GitLab、Bitbucket、汎用）

**SBOM 出力：**
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

#### 再帰的サブモジュールスキャン

Git サブモジュール内の依存関係（package.json、Cargo.toml、CMakeLists.txt など）を深さ制限付きで自動的にスキャンします：

**例：** npm 依存関係を持つサブモジュールを含むプロジェクト
```
.
├── .gitmodules
├── libs/
│   └── json/          # Git サブモジュール
│       ├── package.json   # 再帰的にスキャン
│       └── CMakeLists.txt # こちらもスキャン
```

**機能：**
- サブモジュール内の全マニフェストタイプを検出（npm、Cargo、vcpkg、CMake など）
- ネストされたサブモジュールに対応（サブモジュール内のサブモジュール）
- `--submodule-depth` で深さ制限を設定可能（デフォルト：3 レベル）
- ソース帰属にサブモジュールの起源を表示

**ネストされた依存関係の SBOM 出力：**
```json
{
  "name": "typescript",
  "versionInfo": "5.0.0",
  "ecosystem": "npm",
  "sourceInfo": "javascript/packagejson extractor from libs/json/package.json (submodule: libs/json)"
}
```

#### CLI の強化

**新規フラグ：**
- `--scan-cmake=<true|false>` - CMake スキャンの有効化／無効化（デフォルト：true）

**既存フラグ（完全実装済み）：**
- `--submodule-depth=<N>` - ネストされたサブモジュールの最大再帰深さ（デフォルト：3）

**使用例：**
```bash
# CMake と再帰的サブモジュールスキャンを有効（デフォルト）
./target/release/radeis_sc2sbom /path/to/project

# CMake スキャンを無効化
./target/release/radeis_sc2sbom /path/to/project --scan-cmake=false

# サブモジュールの再帰を 1 レベルに制限
./target/release/radeis_sc2sbom /path/to/project --submodule-depth=1
```

### 技術詳細

#### CMake パーサーの実装
- **ファイル：** [src/parsers/cmake/mod.rs](../src/parsers/cmake/mod.rs)、[src/parsers/cmake/fetchcontent.rs](../src/parsers/cmake/fetchcontent.rs)、[src/parsers/cmake/external_project.rs](../src/parsers/cmake/external_project.rs)
- **パターン：** `(?is)` フラグを使用した正規表現ベースの解析（大文字小文字を区別しない + 複数行）
- **制限：** CMake 変数は解決不可——静的な値のみ対応

#### 再帰スキャンの実装
- **関数：** [src/scanner/mod.rs](../src/scanner/mod.rs) 内の `scan_submodule_recursively()`
- **安全性：** 深さ制限により循環参照による無限ループを防止
- **カバレッジ：** 全既存パーサー（npm、Cargo、Python、Go、vcpkg、CMake など）

### 競合優位点

|---------|----------------|-------------|-----------|
| **CMake FetchContent** | ✅ 静的解析 | ❌ | ✅ ビルドキャプチャのみ |
| **CMake ExternalProject** | ✅ 静的解析 | ❌ | ✅ ビルドキャプチャのみ |
| **サブモジュール内ネスト依存関係** | ✅ 再帰スキャン | ❌ | ❌ |
| **ビルド不要** | ✅ 全て静的 | ✅ | ❌ CMake はビルドが必要 |

### 移行に関する注意

**API の変更：**
- `scan_directory()` のシグネチャに `scan_cmake` パラメーターを追加（カスタム統合に影響）
- テストを `scan_cmake` 引数を含めて更新

**破壊的変更：** CLI ユーザーへの破壊的変更なし（後方互換）

---

## v1.0.0 - C++ サポート（2026-02-20）

### 概要

vcpkg マニフェスト解析と Git サブモジュール検出を備えた最初の C++ エコシステムサポートです。最新のパッケージマネージャーを使用する C/C++ プロジェクトの SBOM 生成を可能にします。

### 主な機能

#### vcpkg マニフェストパーサー

全バージョン制約形式に対応した vcpkg.json の完全サポート：

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

**対応バージョン形式：**
- 単純な文字列依存関係：`"zlib"`
- `version>=`：最小バージョン制約
- `version>`：より大きいバージョン
- `version=`：正確なバージョン
- `version-semver`：セマンティックバージョン
- `version-date`：日付ベースのバージョン
- `port-version`：ポートリビジョン番号
- バージョンピン留め用の overrides セクション
- features は source_file メタデータに保存

**SBOM 出力：**
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

#### Git サブモジュールの検出

コミット SHA の解決を含む `.gitmodules` ファイルの自動解析：

```ini
[submodule "libs/json"]
    path = libs/json
    url = https://github.com/nlohmann/json.git
    branch = master
```

**機能：**
- サブモジュールの名前、パス、URL、ブランチを解析
- `git ls-tree HEAD` を通じてコミット SHA を解決
- HTTPS および SSH URL に対応
- 複数ホストに対応：GitHub、GitLab、Bitbucket、セルフホスト

**SBOM 出力：**
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

#### 脆弱性のあるパッケージの潜在的フラグ

インポートスキャンで検出されたパッケージ（version="detected"）の脆弱性は、**潜在的**としてマークされます：

```
   Package version 'detected' is unknown (detected via import scanning).
   Actual version may not be affected.
```

これにより以下を区別できます：
- **確認済みの脆弱性** - バージョンが既知のパッケージ（例：`urllib3@2.6.0`）
- **潜在的な脆弱性** - インポートスキャンで検出されたがバージョン情報のないパッケージ

#### バージョン形式の修正

- **修正前：** `"==2.6.0"`（無効な purl 形式）
- **修正後：** `"2.6.0"`（正しい purl 形式）

この修正により、誤検知の脆弱性数が大幅に減少しました（MRPT 比較で 37 → 8）。

#### Python の検出方法

radeis はマニフェストファイル（requirements.txt、pyproject.toml）から**宣言された依存関係**をスキャンし、インストール済み環境をスキャンするツールよりも正確な SBOM を生成します：

| アプローチ | radeis | 環境スキャナー |
|----------|--------|---------------------|
| 検出方法 | 宣言された依存関係 | インストール済みパッケージ |
| 捕捉内容 | プロジェクトの要件 | 環境固有のパッケージ |
| 精度 | より正確 | 誤検知が含まれる場合がある |

**重要な洞察：** 環境スキャナーは実際のプロジェクト要件ではない `importlib-metadata` や `zipp`（Python < 3.10 のバックポート）などを含める場合があります。

### 新規 CLI オプション

```bash
--scan-submodules <BOOL>   # Git サブモジュールスキャンを有効化（デフォルト：true）
--submodule-depth <N>      # 最大再帰深さ（デフォルト：3）
```

**使用例：**
```bash
# vcpkg を使用する C++ プロジェクトをスキャン
radeis_sc2sbom --path ./cpp_project --format spdx-json

# サブモジュールスキャンを無効化
radeis_sc2sbom --path . --scan-submodules false

# サブモジュールの再帰深さを制限
radeis_sc2sbom --path . --submodule-depth 1
```

### purl 形式のサポート

| エコシステム | purl 形式 | 例 |
|-----------|-------------|---------|
| vcpkg | `pkg:vcpkg/{name}@{version}` | `pkg:vcpkg/zlib@1.2.13` |
| GitHub | `pkg:github/{owner}/{repo}@{commit}` | `pkg:github/nlohmann/json@bc889af` |
| GitLab | `pkg:gitlab/{owner}/{repo}@{commit}` | `pkg:gitlab/owner/repo@abc123` |
| Bitbucket | `pkg:bitbucket/{owner}/{repo}@{commit}` | `pkg:bitbucket/owner/repo@def456` |
| 汎用 | `pkg:generic/{name}@{version}` | `pkg:generic/custom-lib@1.0.0` |

### 技術的な変更

**新規ファイル：**
- `src/parsers/cpp/mod.rs` - C++ パーサーモジュールのエントリーポイント
- `src/parsers/cpp/vcpkg.rs` - vcpkg.json パーサー
- `src/parsers/git/mod.rs` - Git パーサーモジュールのエントリーポイント
- `src/parsers/git/submodules.rs` - .gitmodules パーサー
- `src/parsers/git/commit_resolver.rs` - Git コミット SHA リゾルバー
- `src/parsers/git/url_parser.rs` - Git URL パーサー

**変更ファイル：**
- `src/cli.rs` - 新規 CLI オプション
- `src/scanner/mod.rs` - vcpkg.json と .gitmodules の検出
- `src/formats/spdx.rs` - vcpkg/git-submodule の purl サポート
- `src/formats/console.rs` - 潜在的脆弱性の表示
- `src/parsers/python.rs` - バージョン形式の修正（演算子の除去）
- `src/parsers/mod.rs` - Python 標準ライブラリに `__future__` を追加
- `src/main.rs` - 新しい引数をスキャナーに渡す

### v0.9.3 からの移行

**破壊的変更なし。** 既存の全機能はそのまま動作します。

新機能は以下の条件で自動的に有効になります：
- `vcpkg.json` が見つかった場合 → vcpkg パーサーが実行
- `.gitmodules` が見つかった場合 → サブモジュール検出が実行（git が必要）

### 今後のロードマップ（v1.0.x）

- v1.0.1：CMake FetchContent/ExternalProject の解析
- v1.0.2：Conan パッケージマネージャーのサポート
- サブモジュール内の再帰スキャン

---

## v0.9.3 - Python の大幅改善（2026-02-10）

### 概要

Pipenv と pyproject.toml の完全なサポートにより、Python パッケージ検出数が **487% 向上**しました。

### 主な改善点

#### Pipfile/Pipfile.lock パーサー
ロックファイル解析を含む Pipenv の完全統合。

**改善前：**
```json
{
  "name": "azure-identity",
  "versionInfo": "detected",
  "sourceInfo": "Import scanner"
}
```

**改善後：**
```json
{
  "name": "azure-identity",
  "versionInfo": "1.12.0",
  "downloadLocation": "https://pypi.org/project/azure-identity/1.12.0/",
  "sourceInfo": "python/pipfilelock extractor"
}
```

**結果（nodejs-service）：**
- Pipfile.lock から 47 パッケージを検出（以前は 8 件）
- バージョン精度 100%（"@detected" ゼロ）
- Black Duck と完全一致（47/47）
- SHA256 チェックサムを抽出

#### pyproject.toml マルチフォーマットパーサー
3 つの形式を自動サポート：

**PEP 621（標準形式）：**
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

#### SHA256 チェックサムの抽出
サプライチェーンセキュリティのためにロックファイルからチェックサムを抽出：
- Pipfile.lock：hashes 配列の最初の SHA256
- poetry.lock：files[0].hash フィールドのハッシュ
- 内部保存（SPDX/CycloneDX 出力にはまだ未反映）

#### 重複防止
ロックファイルが存在する場合、対応するマニフェストファイルをスキップ：
- Pipfile.lock が存在する場合 → Pipfile をスキップ
- poetry.lock が存在する場合 → pyproject.toml をスキップ

### パフォーマンス指標

| 指標 | v0.9.3 | v0.9.2 | 改善 |
|--------|--------|--------|-------------|
| Python パッケージ | 47 | 8 | +487% |
| 実際のバージョン | 47 | 0 | +47 |
| "@detected" バージョン | 0 | 8 | -100% |
| Black Duck との一致率 | 100% | 17% | +83% |

### 競合上の位置づけ

| ツール | パッケージ数 | バージョン精度 | チェックサム | 対応形式 |
|------|----------|------------------|-----------|---------|
| **radeis v0.9.3** | 47 🏆 | 100% 🏆 | ✅ SHA256 🏆 | Pipfile、poetry、pyproject 🏆 |
| Black Duck | 47 | 100% | 不明 | Pipfile、poetry |

### 新規ファイルサポート

**Pipfile.lock（ロックファイル）**
- 正確なバージョン（`==1.12.0`）
- SHA256 ハッシュ
- 直接依存関係の検出（`index` フィールド）
- 開発依存関係（`develop` セクション）

**Pipfile（マニフェスト）**
- バージョン仕様（`*`、`>=1.0`、`==1.2.3`）
- 開発依存関係
- 複数ソース

**pyproject.toml（マルチフォーマット）**
- PEP 621、Poetry、PDM 形式
- 自動検出
- バージョン仕様とエクストラ

### 技術的な変更

**変更ファイル：**
- `src/parsers/python.rs`（+219 行）- 新規パーサー
- `src/scanner/mod.rs`（+30 行）- パーサーの登録
- `Cargo.toml` - バージョンを 0.9.3 に更新

**主要アルゴリズム：**
```rust
// Pipfile.lock の構造
#[derive(Deserialize)]
struct PipfileLock {
    default: HashMap<String, PipfilePackage>,
    develop: Option<HashMap<String, PipfilePackage>>,
}

// チェックサムの抽出
fn extract_first_sha256(hashes: &[String]) -> Option<String> {
    hashes.iter()
        .find(|h| h.starts_with("sha256:"))
        .map(|h| h.trim_start_matches("sha256:").to_string())
}

// pyproject.toml のマルチフォーマット処理
pub fn parse_pyproject_toml(path: &Path) -> Result<Vec<Dependency>> {
    // PEP 621、Poetry、PDM の順に試みる
    if let Some(project) = pyproject.get("project") { ... }
    if let Some(poetry) = pyproject.get("tool").and_then(|t| t.get("poetry")) { ... }
    if let Some(pdm) = pyproject.get("tool").and_then(|t| t.get("pdm")) { ... }
}
```

### ユースケース

**Pipenv プロジェクト：**
```bash
radeis_sc2sbom --path ./python_project --format spdx-json

# 出力：47 パッケージ、バージョン精度 100%
```

**最新の Python プロジェクト：**
```bash
radeis_sc2sbom --path ./pyproject_project --format all

# PEP 621、Poetry、または PDM 形式を自動検出
```

**サプライチェーン監査：**
```bash
radeis_sc2sbom --path ./workspace --format spdx-json

# ロックファイルから SHA256 チェックサムを抽出
```

### v0.9.2 からの移行

**破壊的変更なし。** 全機能が自動的に有効になります：

```bash
# v0.9.2：8 パッケージ、全て "@detected"
radeis_sc2sbom --path ./nodejs-service --format spdx-json

# v0.9.3：47 パッケージ、全て実際のバージョン
# （同じコマンド、自動的に改善）
```

**変更内容：**
- ✅ Pipfile/Pipfile.lock を自動解析
- ✅ pyproject.toml を解析（全形式）
- ✅ SHA256 チェックサムを抽出
- ✅ バージョン精度 100%
- ✅ パフォーマンスへの影響ゼロ

### バグ修正

1. Python バージョン検出——ロックファイルにより "@detected" を排除
2. チェックサム抽出——SHA256 サポートを追加
3. パーサーの優先順位——ロックファイルがマニフェストを上書き
4. 重複防止——ロックファイルが存在する場合はマニフェストをスキップ

---

## v0.9.2 - ユーザーエクスペリエンスの改善（2026-02-08）

### プログレスインジケーター

スキャン中のリアルタイムフィードバック：

```
[1/5] Walking directory tree... 47 entries scanned
[2/5] Parsing complete... 54 dependencies discovered
[3/5] Deduplicating dependencies... 54 → 47 unique
[5/5] Scan complete
```

### 機能

- パーセンテージ表示付きのプログレスバー
- 長時間処理中のスピナーアニメーション
- 5 段階のパイプライン可視化
- 残り時間の推定

---

## v0.9.1 - ROS 統合（2026-01-28）

### 自動バージョン解決

ROS パッケージは `package.xml` にバージョン情報がありません。v0.9.1 は rosdistro API を介した自動解決を追加します。

**改善前：**
```
rclpy @ unspecified
downloadLocation: NOASSERTION
```

**改善後：**
```
rclpy @ 7.1.9 (from rosdistro)
downloadLocation: https://github.com/ros2/rclpy.git
```

### ros2cli ベンチマーク

- 94 件のユニーク依存関係（BlackDuck の 4 件と比較）
- ROS パッケージ 15 件を検出
- GitHub URL 付きパッケージ 47 件
- バージョン付きパッケージ 62 件（66%）
- 脆弱性 5 件を発見

### 機能

- 並列 API 取得（10〜27 倍の高速化）
- SHA-1 チェックサム
- 自動修正推奨
- コンパクト SPDX モード（30% の容量削減）

**対応ディストリビューション：** jazzy、iron、humble、rolling、noetic、melodic

---

## v0.9.1.1 - ホットフィックス（2026-01-28）

### 修正：Markdown レポートの表示問題

**問題：** ROS マルチパッケージレポートで空のコードブロックが表示される

**修正：** ROS レポートセクションで全ての直接依存関係を表示するよう変更

---

## v0.8.0 - メタデータとセキュリティ（2026-01-22）

### リッチなメタデータ

- エコシステム全体でライセンスカバレッジ 95%+
- サプライヤー／提供元のトラッキング 90%+
- UUID ベースの SPDX ID
- セキュリティツール向けの CPE 識別子

### 改善点

- 並列メタデータ取得
- ライセンス検出の強化
- サプライヤートラッキングの改善
- CPE 生成の向上

---

## 今後のロードマップ

### v0.9.4+
- SPDX/CycloneDX 出力でのチェックサム表示
- PyPI メタデータキャッシュの強化
- pdm.lock サポート
- conda environment.yml サポート

---

**最新リリース：** v1.0.7（2026 年 3 月 12 日）
**ステータス：** 本番環境対応済み
**Python サポート：** 業界トップクラス 🏆
