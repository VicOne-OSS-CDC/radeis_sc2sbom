# 変更履歴

このファイルにはプロジェクトの重要な変更がすべて記録されています。

フォーマットは [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) に基づいており、
このプロジェクトは [セマンティックバージョニング](https://semver.org/spec/v2.0.0.html) に従っています。

## [Unreleased]

## [1.0.18] - 2026-05-13

### 追加

### 変更
- Phase 24 チューニング：17 のルールレベル変更により v1.0.18 前ベースライン比で誤検知を 89,479 件削減

### 削除
- `--cppcheck-path` CLI オプション（cppcheck 統合を AST スキャナーに置き換え）
- `check_plaintext_password` デッドコード関数

## [1.0.17] - 2026-05-11

### 追加（内部ビルド）
- **`--cppcheck-path <PATH>`**：cppcheck バイナリの PATH 検索を上書き
- **SARIF 2.1 出力**：`<project>_static_analysis.sarif` を `_static_analysis.md` と並べて出力；GitHub Code Scanning、VS Code Problem Matcher、CI/CD パイプラインに対応
- **`--sarif-output <PATH>`**：SARIF レポートをカスタムパスに書き込み
- **SARIF フィンガープリント**：各 `SarifResult` の SHA-256 フィンガープリントによる実行間の安定した重複排除
- **`--sarif-baseline <PATH>`**：現在の発見項目を以前の SARIF 実行と比較；新規発見項目のみを報告——回帰のみを通知する CI ゲートを実現
- **AUTOSAR arxml 解析**：`.arxml` ファイルが依存関係のために完全に解析される——`SW-COMPONENT-PROTOTYPE`、`BSW-MODULE-DESCRIPTION`、および全 SWC 型定義要素を `autosar` エコシステム依存関係として抽出
- **AUTOSAR バージョン抽出**：`.epd` ファイル（`ECUC-MODULE-DEF/REVISION-LABEL`）と Doxygen C/H ヘッダー（`SW Version : X.Y.Z`）をスキャンし、AUTOSAR 依存関係に `unspecified` の代わりに実バージョン文字列を設定
- **AUTOSAR エコシステム昇格**：Makefile `-lFoo` フラグで発見した `system` エコシステムの依存関係が、一致する `.epd` または Doxygen バージョンが見つかった場合に `autosar` エコシステムに昇格——重複エントリと誤分類のリンカー依存関係を解消

### 修正
- AUTOSAR プロジェクトで深度 > 3 の C/C++ ファイルが SAST スキャンされなかった問題——`has_c_cpp_files` の最大深度を 3 から 6 に引き上げ
- 同じ `(name, ecosystem)` に対してバージョン付きと `unspecified` の AUTOSAR 依存関係が両方存在する場合の重複を解消——バージョン付きエントリが優先
- 同名の `autosar` エコシステムエントリが存在する場合に `system` エコシステムエントリを抑制

## [1.0.16] - 2026-05-10

### 追加
- **フォールバックモード**：マニフェスト由来のコンポーネントディレクトリが見つからないがスキャンルート以下に C/C++ ソースファイルが存在する場合、合成した `(project_name, "C/C++") → scan_root` エントリを自動挿入 — マニフェストなしのリポジトリ（例：NIST Juliet テストスイート）のスキャンを可能にする
- **`has_c_cpp_files` ヘルパー**：`is_c_cpp_source` 述語を再利用した浅い `WalkDir`（最大深度 3）チェック；非 C リポジトリでの誤検知を防ぐためにフォールバックモードが使用
- **`resolve_component_dir` ヘルパー**：3 つの戦略（完全一致、`lib` プレフィックス、大文字小文字を区別しないスキャン）でマニフェスト宣言の C/C++ 依存関係をベンダーソースサブディレクトリにマッピング；マッチするベンダーサブディレクトリがない依存関係には `None` を返し、外部/システム依存関係からの誤検知を防止

### 内部変更
- `build-all.sh` に `--internal` フラグを追加：パブリック版とインターナル版を同時にビルドし、インターナルバイナリ名に `-internal` サフィックスを付加

## [1.0.15] - 2026-05-09

### 追加
- **AUTOSAR 検出**：スキャン前に `detect_autosar()` プレパスを実行；`.arxml` ファイル（DET-01）、BSW/MCAL/RTE/AUTOSAR/SWC ディレクトリ名（DET-02）、ビルドファイルの `AUTOSAR_VERSION`/`AR_VERSION` トークン（DET-03）でプロジェクトを検出
- **AUTOSAR 分類**：`classify_autosar_components()` が依存関係名を内蔵 BSW モジュール設定と照合；一致するコンポーネントを `ecosystem="autosar"` に昇格し `AutosarMetadata`（module_name、layer、platform）を付与
- **AUTOSAR 出力 — CycloneDX**：AUTOSAR コンポーネントが `autosar:layer` および `autosar:platform` プロパティを出力（例：`"BSW-Memory"`、`"Classic"`）
- **AUTOSAR 出力 — SPDX**：AUTOSAR コンポーネントが `ExternalRef OTHER` エントリとして `autosar:layer` および `autosar:platform` を出力
- **サプライヤー設定**：`--supplier-config <path>` で AUTOSAR コンポーネント名をサプライヤー文字列にマッピングする YAML ファイルを受け付け；マッピングされたコンポーネントは CycloneDX プロパティと SPDX ExternalRef に `autosar:supplier` を出力；未マッピングは `NOASSERTION` にフォールバック
- **BSW 設定オーバーライド**：`--bsw-config <path>` でカスタム YAML ファイルにより内蔵 AUTOSAR BSW モジュール設定を上書き

### 変更
- `--output` が `--format all` だけでなく全単一フォーマット（`spdx-json`、`spdx-tag-value`、`cyclonedx-json`、`console`）に対応；単一フォーマットで `--output` を省略すると標準出力に出力（従来の動作を維持）

### 内部

## [1.0.14] - 2026-04-24

### 修正
- ブロークンシンボリックリンクに遭遇してもスキャナーが中断しなくなりました — `Warning: skipping` を出力して続行します（5 箇所すべての WalkDir 使用箇所に適用：scanner、フォールバックインポートスキャン、C パーサー）
- `$(OPENSSL_VERSION)` のような Makefile 変数参照が SPDX `versionInfo` に漏れなくなりました — パーサーおよびすべての SPDX/CycloneDX バージョン出力箇所でフィルタリングし、代わりに `NOASSERTION` を出力
- C/C++ ライブラリのライセンスが、すべて `NOASSERTION` となる代わりに `.pc` の `License:` フィールドと既知ライブラリ参照テーブル（24 の一般的なシステムライブラリ）から解決されるようになりました

### 変更
- Linux リリースバイナリは musl（`x86_64-unknown-linux-musl`）により静的リンクされるようになりました — glibc バージョン依存を排除；Ubuntu 22.04+、24.04、Alpine、その他あらゆる x86_64 Linux 上で実行可能
- `build-all.sh` に musl クロスリンカーツールチェーンのガードを追加

### 内部変更
- `warn_on_walkdir_err` ヘルパーを `src/util/mod.rs` に抽出し、5 箇所で重複していた `filter_map` クロージャを統合
- Unix 固有 API を使用するクロスプラットフォームテストを `#[cfg(unix)]` でガード

## [1.0.13] - 2026-04-14

### 追加
- **AI モデル**：マルチモーダルサブモデル分解 — 複合モデル（Gemma-4、LLaVA、Qwen-VL）をテキスト、ビジョン、オーディオのサブモデルコンポーネントに分解
- **AI モデル**：新しい `SubModelInfo` 構造体。各サブモデルのアーキテクチャを記録：model_type、layers、hidden_size、heads、dtype、vocab_size、コンテキストウィンドウ、モダリティ固有フィールド（patch_size、conv_kernel_size など）
- **AI モデル**：ガード条件 — 真にマルチモーダルなモデル（text_config + vision/audio_config が存在）に対してのみサブモデルを生成
- **CycloneDX**：親 AI モデルコンポーネント内にネストされた `components` 配列、各サブモデルに `radeis:ai:sub_model:*` プロパティ付き
- **SPDX**：親モデルから各サブモデルへの `CONTAINS` 包含関係を持つ子パッケージ
- **コンソール**：サブモデルサマリーテーブル（モダリティ、モデルタイプ、レイヤー数、隠れ層サイズ、ヘッド数、dtype、モダリティ固有の追加情報を表示）
- **テスト**：5 件の新規テスト（Safetensors 4 件 + GGUF 1 件）— マルチモーダル抽出、テキスト専用ガード、ビジョン+テキストのみ、GGUF エンリッチメントをカバー

## [1.0.12] - 2026-04-14

### 追加
- **AI モデル**：HuggingFace コンパニオンファイルからのリッチメタデータ抽出（Safetensors および GGUF リポジトリの両方に対応）
- **AI モデル**：`generation_config.json` の解析 — temperature、top_k、top_p 推論デフォルト
- **AI モデル**：`tokenizer_config.json` の解析 — processor_class、model_max_length（天文学的な値のキャップ付き）
- **AI モデル**：`preprocessor_config.json` の解析 — 画像、オーディオ、ビデオプロセッサタイプとパラメータ
- **AI モデル**：`README.md` YAML フロントマターの解析 — base_model（文字列またはリスト）、license、pipeline_tag、quantized_by、prompt_template、tags、languages、datasets
- **AI モデル**：`config.json` 抽出の拡張 — model_type、text_config（hidden_layers、hidden_size、attention_heads、max_position_embeddings）、マルチモーダル検出（vision_config、audio_config）
- **AI モデル**：dtype フォールバックチェーン — `torch_dtype` > `dtype` > `text_config.dtype`
- **AI モデル**：`adapter_config.json` による LoRA/QLoRA アダプター検出
- **AI モデル**：GGUF コンパニオンファイルエンリッチメント — バイナリメタデータが常に優先、コンパニオンファイルはギャップを補完
- **AI モデル**：バイナリ + README ソースからの tags、languages、datasets の重複排除ユニオンマージ
- **AI モデル**：すべてのコンパニオンファイル読み込みに 1 MB の安全上限を設定
- **AI モデル**：README.md ファイル名の大文字小文字を区別しないマッチングおよび CRLF 改行対応
- **CycloneDX**：リッチ AI モデルメタデータ向けの約 25 個の新しい `radeis:ai:*` プロパティ
- **SPDX**：model_type、コンテキストウィンドウ、モダリティサマリーを含む sourceInfo の拡張
- **コンソール**：アーキテクチャ、マルチモーダル、生成、来歴セクションを追加した AI Model Details テーブルの拡張
- **テスト**：`safetensors_tests.rs` に 8 件、`gguf_tests.rs` に 6 件の新規テスト

## [1.0.11] - 2026-04-13

### 追加
- **AI モデル**：Safetensors AI モデル SBOM パース — `.safetensors`、`model.safetensors.index.json`、`config.json` に対応
- **AI モデル**：ディレクトリレベルスキャン — シャード数に関わらず、モデルごとに 1 件の Dependency エントリを生成
- **AI モデル**：HuggingFace Safetensors モデル向け `pkg:huggingface` PURL
- **AI モデル**：Safetensors モデル向け CycloneDX `machine-learning-model` コンポーネントタイプおよび `modelCard`
- **AI モデル**：シャード重複排除 — マルチシャードモデル（例：`model-00001-of-00002.safetensors`）を単一の SBOM エントリに集約
- **AI モデル**：新しい `AIModelMetadata` フィールド：`safetensors_format`、`total_size_bytes`、`shard_count`、`torch_dtype`、`transformers_version`、`vocab_size`
- **テスト**：`tests/parser_tests/safetensors_tests.rs` に 12 件の新規テストを追加

## [1.0.10] - 2026-04-13

### 追加
- **Java/Gradle**：`build.gradle`（Groovy DSL）と `build.gradle.kts`（Kotlin DSL）の完全な依存関係解析 — 従来は検出のみ
- **Java/Gradle**：文字列記法（`'group:artifact:version'`）、マップ記法（`group: 'g', name: 'a', version: 'v'`）、プラットフォーム/BOM サポート
- **Java/Gradle**：スコープ分類 — `testImplementation` → テスト、`compileOnly` → 提供済み、`annotationProcessor`/`kapt`/`ksp` → ビルド
- **Java/Gradle**：Android プロジェクトサポート（`androidTestImplementation`、`androidTestCompile`）

### 変更
- **Java**：Gradle のステータスをエコシステム表で「検出のみ」から「本番対応」に変更

## [1.0.9] - 2026-04-10

### 追加
- **AI モデル**：GGUF バイナリパーサーによるメタデータ抽出（アーキテクチャ、量子化タイプ、テンソル情報、コンテキスト長）
- **AI モデル**：CycloneDX `machine-learning-model` コンポーネントタイプ、`modelCard` 付き（トレーニングパラメータ、データセット）
- **AI モデル**：SPDX `pkg:huggingface` PURL による AI モデル依存関係の識別
- **AI モデル**：整合性検証 — テンソルパラメータのクロスバリデーションおよび SHA-256 ハッシュによるモデルファイルの真正性確認
- **AI モデル**：一般的な AI モデルライセンスの SPDX 形式への正規化
- **CLI**：`--scan-ai-models` フラグで GGUF モデルスキャンを有効化（デフォルト：true）

### 変更
- **CLI**：5 つの C/C++ ビルドシステムフラグ（`--scan-cmake`、`--scan-pkgconfig`、`--scan-autotools`、`--scan-makefiles`、`--scan-mk-files`）を単一の `--scan-c-build-systems` フラグに統合
- **CLI**：`--meson-parse-subprojects` を `--scan-meson` に統合（Meson スキャン有効時は常にオン）
- **コア**：`scan_directory()` の引数を 20 個から 13 個に簡素化

### 削除
- **CLI**：`--resolve-system-deps` フラグ（デッドコード — 実装に接続されていなかった）
- **CLI**：`--meson-parse-subprojects` フラグ（`--scan-meson` がアクティブな場合、サブプロジェクト解析は常に有効化）

## [1.0.7] - 2026-03-12

### 変更 - 脆弱性スキャンのオプトイン化

- **デフォルトでクリーンな出力** — スキャンが無効の場合、脆弱性サマリー行、リスク評価セクション、パッケージごとの詳細はすべて非表示になります

## [1.0.6] - 2026-03-04

### 追加 - 本番環境向け SBOM フィルタリングと自動依存関係スコープ分類

**フェーズ 1〜3 完了：コア分類システム**
- **すべての依存関係に対する自動スコープ分類**（v1.0.6 フェーズ 1〜3）
  - 6 種類のスコープ：Runtime、Build、Test、Development、Optional、Provided
  - 多戦略分類：エコシステム、名前パターン、ディレクトリ分析
  - 信頼度スコア（0.0〜1.0）と詳細な根拠
  - 10 以上のエコシステムをサポート（npm、PIP、cargo、SYSTEM、BUILD-CONFIG 等）

**フェーズ 4 完了：包括的なテストと検証**
- **42 件の新規統合テスト**によるスコープフィルタリングと分類精度の検証
  - スコープフィルタリング統合テスト 14 件
  - エンドツーエンドの本番環境モードテスト 11 件
  - 実環境分類検証テスト 17 件
- **609 件のテストすべて合格**（ライブラリ 203 件 + バイナリ 203 件 + 統合 200 件 + ドキュメント 3 件）
- **実際の依存関係による検証済み**：
  - 一般的な C/C++ ランタイムライブラリ（zlib、curl、openssl、protobuf）
  - ビルドツール（cmake、gcc、ninja、meson）
  - テストフレームワーク（pytest、jest、gtest、unity）
  - 開発ツール（pylint、black、eslint、prettier）
  - Web フレームワーク（django、flask、express、react）

**フェーズ 5 完了：ドキュメント**
- **v1.0.6 機能の包括的なドキュメント**
  - [README.md](README.md) を更新し、本番環境モードの例を追加
  - 新規 [SCOPE_CLASSIFICATION.md](docs/SCOPE_CLASSIFICATION.md) — 完全なスコープフィルタリングガイド（300 行以上）
  - [CLI.md](docs/CLI.md) を更新し、スコープフィルタリングオプションを追加
  - CHANGELOG.md にフェーズ 5 完了の記録を追記
- **ドキュメントの内容**：
  - クイックスタート例
  - 依存関係スコープの説明（6 種類）
  - 分類方法（4 つの戦略）
  - フィルタリングオプションと例
  - 信頼度スコアの解釈
  - トラブルシューティングガイド
  - 本番環境・セキュリティ・コンプライアンスのベストプラクティス
  - 実環境検証の指標

**スコープフィルタリング CLI：**
- **`--scope-filter <SCOPE>`** — スコープによる依存関係フィルタリング
  - 複数値に対応：runtime、build、test、development、optional、provided
  - 例：`--scope-filter runtime --scope-filter optional`
- **`--production`** — 本番環境モード（runtime + optional のみ）
  - SBOM サイズを大幅に削減（例：典型的なプロジェクトで 67 → 11 パッケージ）
  - `--scope-filter runtime --scope-filter optional` と同等

**出力とレポートの強化：**
- **スコープ統計**をコンソールおよび Markdown レポートに表示
  - スコープ別のカウントとパーセンテージ
  - 分類の平均信頼度
  - 依存関係の合計数
- **依存関係作成のビルダーパターン**
  - `Dependency::new().with_scope(scope, confidence, reason)`
  - テストコードの簡略化と API の使いやすさの向上

**分類機能：**
- **エコシステム対応分類**：
  - SYSTEM ライブラリ → Runtime（信頼度 0.8 以上）
  - BUILD-CONFIG → Build（リンク分析の改善待ち）
  - GIT-SUBMODULE → Provided
  - MESON-WRAP/SUBPROJECT → Build
  - PIP/npm/cargo → コンテキスト依存
- **名前ベースのヒューリスティック**：
  - 既知ツールの完全一致（信頼度 1.0）
  - 大文字小文字を区別しないマッチング
  - パターンベースの分類
- **詳細な根拠**：すべての分類に詳細な説明を含む

**カテゴリ別テスト内容：**
- **スコープフィルタリング**（14 件）：
  - デフォルト動作（フィルタリングなし）
  - 本番環境モードフィルタリング
  - カスタム単一・複数スコープフィルタリング
  - エッジケース（無効なフィルタ、空の結果）
- **エンドツーエンドのワークフロー**（11 件）：
  - 完全な分類パイプライン
  - 本番環境 SBOM 生成
  - CycloneDX 出力統合
  - SBOM サイズ削減の検証
  - エコシステム多様性の検証
- **実環境分類**（17 件）：
  - 一般的なライブラリの検出精度
  - ビルドツールの識別
  - テストフレームワークの識別
  - 開発ツールの分類
  - 信頼度スコアの分布
  - 分類根拠の検証

### 変更
- **Dependency 構造体**にスコープフィールドを追加
  - `scope: DependencyScope` — 分類結果
  - `scope_confidence: f32` — 信頼度スコア（0.0〜1.0）
  - `scope_reason: String` — 人間が読める説明
- **メインパイプライン**に自動分類を追加（ステップ 3.5/5）
- **SBOM 構造体**に `scope_statistics: Option<ScopeStatistics>` を追加
- **コンソール出力**のサマリーにスコープ内訳を表示

### フェーズ 4 検証結果
- **テスト合格率**：100%（609/609 件合格）
- **分類精度**：
  - ビルドツール：100% 正確（cmake、gcc、ninja、meson）
  - テストフレームワーク：100% 正確（pytest、jest、junit、mocha）
  - 開発ツール：100% 正確（pylint、black、eslint、prettier）
  - ランタイムライブラリ：SYSTEM エコシステムで高精度
  - BUILD-CONFIG：デフォルトで Build に分類（リンク分析の改善待ち）
- **信頼度スコアの分布**：
  - 名前の完全一致：0.95〜1.0（pytest、cmake 等）
  - エコシステムベース：0.7〜0.9（SYSTEM、GIT-SUBMODULE）
  - ヒューリスティックベース：0.5〜0.8（フォールバック時）
- **エコシステムカバレッジ**：10 以上のエコシステムで検証済み
- **本番環境 SBOM のサイズ削減**：一般的に 50〜80%（例：67 → 11 パッケージ）

## [1.0.5] - 2026-03-02

### 追加 - GitHub Actions と CI/CD インフラ

**マルチプラットフォームビルドシステム：**
- **GitHub Actions ワークフロー**による自動クロスプラットフォームビルドとリリース
  - macOS（ARM64 と Intel x86_64）：osxcross によるクロスコンパイル
  - Linux（x86_64 glibc）
  - Windows（x86_64）：MinGW によるクロスコンパイル
  - キャッシュ付き並列ビルドによる CI/CD パイプラインの高速化
  - バイナリアセットとチェックサムを含む自動リリース

**リリース自動化：**
- CHANGELOG.md からの**リリースノート自動抽出**
- git タグと Cargo.toml 間の**バージョン整合性検証**
- すべてのリリースバイナリの **SHA256 チェックサム**生成
- xnexus-md2pdf-tool を使用したバイナリ配布ガイドの **PDF 生成**
- カバーページとプロフェッショナルなフォーマットを備えた VicOne スタイルの README.pdf

**ビルドの改善：**
- エンタープライズ GitHub 統合によるセルフホストランナーのサポート
- プライベートサブモジュールアクセス用の GIT_TOKEN 認証
- ビルドの追跡可能性のためのアーティファクト保持（90 日間）
- プラットフォーム固有のビルドフィルタリング（必要なプラットフォームのみビルド）
- 正式リリース/プレリリースフラグの設定可能化

### 変更 - コード品質とデッドコードの除去

**API の簡略化（約 155 行を削除）：**
- **すべてのフォーマット関数から `_with_mode` サフィックスを削除**：
  - `print_spdx_json_with_mode` → `print_spdx_json`
  - `print_spdx_tag_value_with_mode` → `print_spdx_tag_value`
  - `save_spdx_json_with_mode` → `save_spdx_json`
  - `save_spdx_tag_value_with_mode` → `save_spdx_tag_value`
  - `convert_to_spdx_with_mode` → `convert_to_spdx`
  - `convert_to_cyclonedx_with_mode` → `convert_to_cyclonedx`
  - `print_cyclonedx_json_with_mode` → `print_cyclonedx_json`
  - `save_cyclonedx_json_with_mode` → `save_cyclonedx_json`

**コンパイラ警告の解消：**
- フォーマットモジュールから未使用のラッパー関数を削除
- すべてのパーサーモジュールから未使用のインポートをクリーンアップ
- パーサー関数からデッドコードを削除
- 新しい簡略化された API を使用するようにテストの参照を修正

**ドキュメント：**
- BINARY_README.md を更新し、完全な MIT ライセンス文（VicOne Inc. の著作権）を追加
- 顧客向けドキュメントから内部配布セクションを削除
- LICENSE の著作権者を「William Chang」から「VicOne Inc.」に更新

### 移行ノート

**ライブラリ利用者のみ対象**（バイナリユーザーへの影響なし）：

フォーマット API が簡略化され、関数名がより明確になりました：

```rust
// 旧 API（v1.0.5 以前）
save_spdx_json_with_mode(sbom, path, &SbomMode::Complete, false)
convert_to_cyclonedx_with_mode(sbom, &SbomMode::Complete)

// 新 API（v1.0.5 以降）
save_spdx_json(sbom, path, &SbomMode::Complete, false)
convert_to_cyclonedx(sbom, &SbomMode::Complete)
```

### インフラ

**リリースアセット：**
- macOS（ARM64/Intel）、Linux（x86_64 glibc）、Windows（x86_64）用のビルド済みバイナリ
- README.md（Markdown バイナリ配布ガイド）
- README.pdf（VicOne スタイルの PDF ガイド）— 新規追加！
- checksums.txt（SHA256 検証用）

すべてのバイナリは GitHub Actions により、自動テストと検証を経てビルドされています。

## [1.0.4] - 2026-02-25

### 追加 - Meson と Bazel ビルドシステム

**モダン C/C++ ビルドシステムのサポート：**
- **Meson ビルドシステムパーサー** — モダンなメタビルドシステムのサポート
  - `meson.build` ファイル内の dependency() および subproject() 宣言をパース
  - 外部依存関係管理のための wrap ファイル（`*.wrap`）をサポート
  - バージョン制約とビルドオプションを処理
  - Conan ロックファイルとの統合（conanfile.lock で meson 1.2.2 を検出済み）
  - C/C++ プロジェクトのカバレッジを約 2.5% 追加

- **Bazel ビルドシステムパーサー** — Google のビルドシステムのサポート
  - `BUILD`、`BUILD.bazel`、`WORKSPACE` ファイルをパース
  - http_archive、git_repository、maven_jar ルールをサポート
  - URL とコミット SHA からバージョンを抽出
  - 外部リポジトリ参照（@repo//:target）を処理
  - C/C++ プロジェクトのカバレッジを約 2.5% 追加

**CLI の強化：**
- `--scan-meson` フラグ：Meson スキャンの有効/無効切り替え（デフォルト：true）
- `--scan-bazel` フラグ：Bazel スキャンの有効/無効切り替え（デフォルト：true）
- すべての C/C++ エコシステムフラグの包括的なヘルプテキスト

### 追加 - 包括的な比較レポート

**競合分析：**
- **6 つの包括的な比較レポート**（合計 2,897 行）
  - OpenStudio、UD Trucks 本番環境、VDL Bus 本番環境
  - ROS 2 Humble Desktop、scikit-learn、Python テストフィクスチャ
- すべてのプロジェクトにわたる **2,561 の依存関係**を追跡
- 競合ツールより **2.1%〜58.8% 多くのパッケージ**を検出
- 商用ツールにない **4 つの独自機能**：
  - Autotools/pkg-config サポート（レガシー C プロジェクト向け）
  - ROS 2 package.xml のパース
  - Git サブモジュールの再帰的スキャン
  - CMake FetchContent/ExternalProject

**コスト削減分析：**
- BlackDuck ライセンスと比較して **$220K〜$1.65M の節約**
- スキャンあたりのコスト：$0（radeis_sc2sbom）vs $50〜$200（SaaS 競合）
- 比較ドキュメント合計：7 つのレポート + インデックス

### 変更

**カバレッジへの影響：**
- v1.0.4 以前：C/C++ プロジェクトの約 90%
- v1.0.4 以降：**C/C++ プロジェクトの約 95%**

**バグ修正：**
- BUILD ファイル式での Bazel パーサーの括弧マッチングを修正
- システムパッケージの purl タイプを `pkg:generic/` 形式に修正
- 新しいシグネチャ変更に対応するためスキャナーテストを更新

**ドキュメント：**
- README.md を更新し、Meson と Bazel エコシステムのサポートを追加
- BENCHMARKS.md を強化し、詳細な比較方法論を追加
- WHATS_NEW.md に v1.0.4 リリースノートを追加

### インフラ

**テスト：**
- 105 件のユニットテスト合格（Meson/Bazel テスト 8 件を新規追加）
- 実際の BUILD ファイルと meson.build ファイルを使用した統合テスト
- 後方互換性：v1.0.0〜1.0.3 と 100% 互換

## [1.0.3] - 2026-02-24

### 追加 - C レガシービルドシステムのサポート

**従来の C/C++ ビルドシステムパーサー：**
- **pkg-config パーサー** — システムライブラリの依存関係検出
  - `.pc` ファイル（pkg-config メタデータファイル）をパース
  - configure.ac から `PKG_CHECK_MODULES` 宣言を抽出
  - バージョンと依存関係チェーンの解決
  - Requires/Requires.private フィールドを処理
  - システムライブラリ依存関係の約 80% のカバレッジ

- **Autotools パーサー** — GNU ビルドシステムのサポート
  - `configure.ac` ファイルをパース
    - `AC_CHECK_LIB(library, function)` 宣言
    - `AC_SEARCH_LIBS(function, [libs...])` 宣言
    - `PKG_CHECK_MODULES(PREFIX, packages)` マクロ
  - `Makefile.am` ファイルをパース
    - `LDADD` と `LIBADD` リンカーフラグの抽出
    - ライブラリ依存関係のための `-l` フラグのパース
  - 純粋な C プロジェクトの約 60% のカバレッジ

- **Makefile ヒューリスティックパーサー** — プレーン Makefile のサポート
  - LDFLAGS/LIBS からパターンベースで `-l` フラグを抽出
  - ライブラリ名を含む `pkg-config --libs` 呼び出しを検出
  - Make の完全な評価なしにベストエフォートでパース
  - レガシー C++ プロジェクトの約 40% のカバレッジ

**CLI の強化：**
- `--scan-pkgconfig` フラグ：pkg-config スキャンの有効/無効切り替え（デフォルト：true）
- `--scan-autotools` フラグ：Autotools スキャンの有効/無効切り替え（デフォルト：true）
- `--scan-makefiles` フラグ：Makefile スキャンの有効/無効切り替え（デフォルト：true）
- `--resolve-system-deps` フラグ：システムライブラリバージョンの解決を試みる（デフォルト：true）

### 変更

**カバレッジへの影響：**
- v1.0.0〜1.0.2 と組み合わせて：**すべての C/C++ プロジェクトの約 90%**
- レガシープロジェクト（curl、nginx、openssl のパターン）のスキャンに成功
- モダンビルドシステムを持たないプロジェクトのギャップを補完

**SPDX purl のサポート：**
- `pkg:generic/{name}@{version}?type=pkg-config` 形式を追加
- `pkg:generic/{name}@{version}?type=autotools` 形式を追加
- `pkg:generic/{name}@{version}?type=makefile` 形式を追加

**ドキュメント：**
- README.md を更新し、C レガシービルドシステムのサポートを追加
- WHATS_NEW.md に v1.0.3 の詳細なリリースノートを追加
- ヒューリスティックパースの制限とベストプラクティスを文書化

### インフラ

**テスト：**
- tempfile フィクスチャを使用した 5 つの C パーサーすべてのユニットテスト
- 実際の configure.ac、Makefile.am、Makefile サンプルを使用した統合テスト
- テストフィクスチャに openssl.pc、curl スタイルの configure.ac パターンを含む

## [1.0.2] - 2026-02-24

### 追加 - Conan パッケージマネージャーのサポート

**Conan C++ パッケージマネージャー：**
- **Conan ロックファイルパーサー**（`conan.lock`）
  - Conan v1 ロックファイル形式（JSON）をパース
  - 完全なバージョンとリビジョン情報を含むパッケージ参照を抽出
  - 直接依存関係とトランジティブ依存関係グラフの両方をサポート
  - リモートリポジトリのメタデータを処理
- **Conan マニフェストパーサー**（`conanfile.txt`、`conanfile.py`）
  - ロックファイルが利用できない場合のフォールバック
  - バージョン制約のパース（>=、~、==等）
  - オプションとジェネレーターの検出

**SPDX purl のサポート：**
- `pkg:conan/{name}@{version}` 形式を追加
- 利用可能な場合はリビジョンとリモートメタデータを含む

**CLI の強化：**
- `conan.lock`、`conanfile.txt`、`conanfile.py` の自動検出
- 既存の C/C++ スキャンワークフローに統合

### 変更

**スキャナーの改善：**
- 深度検証によるサブモジュールスキャンの最適化
- CMake パース（*.cmake モジュールファイル）の堅牢性向上
- 再帰的スキャンでの冗長な深度チェックを修正
- 形式不正な Conan ファイルのエラーハンドリングを改善

**ドキュメント：**
- README.md を再構成し、整理を改善
- `docs/` ディレクトリに包括的なドキュメントを作成
- WHATS_NEW.md を更新し、v1.0.2 リリースノートを追加
- サポートエコシステムのテーブルに Conan を追加

### インフラ

**テスト：**
- 実際のロックファイルサンプルを使用した Conan パーサーのユニットテスト
- conanfile.txt と conanfile.py の統合テスト
- すべてのテスト合格（合計 124 件）

**CLI：**
- カスタム SBOM 出力ディレクトリ指定のための `--output` フラグを追加
- ヘルプテキストと例を改善

## [1.0.1] - 2026-02-23

### 追加 - CMake サポートと再帰的サブモジュールスキャン

**CMake 依存関係パーサー：**
- **FetchContent パーサー** — モダン CMake の外部依存関係
  - CMakeLists.txt から `FetchContent_Declare()` ブロックをパース
  - GIT_REPOSITORY、GIT_TAG、URL、URL_HASH をサポート
  - サプライチェーンセキュリティのため URL_HASH から SHA256 チェックサムを抽出
  - github/gitlab/bitbucket の適切な purl 生成のために Git URL パーサーを使用
  - GIT_TAG または URL パスからバージョンを抽出
- **ExternalProject パーサー** — レガシー CMake 外部プロジェクトのサポート
  - `ExternalProject_Add()` ブロックをパース
  - FetchContent と同じ Git および URL パターンを処理
  - 静的パース（CMake の実行不要）

**再帰的サブモジュールスキャン：**
- Git サブモジュール内の依存関係を再帰的にスキャン
  - package.json、Cargo.toml、CMakeLists.txt、およびサポートされているすべてのマニフェストファイル
  - 無限再帰を防ぐための深度制限
  - 最大深度の設定可能（デフォルト：3）
- スキャナーモジュールに `scan_submodule_recursively()` 関数を追加

**CLI の強化：**
- `--scan-cmake` フラグ：CMake 依存関係スキャンの有効/無効切り替え（デフォルト：true）

### 変更

**スキャナーシグネチャ：**
- `scan_directory()` を更新し、`scan_cmake` パラメータを追加
- すべての呼び出し箇所を新しいパラメータに対応させて更新
- テストを新しいスキャナーシグネチャに対応させて更新

**SPDX purl のサポート：**
- `pkg:cmake/{name}@{version}` 形式を追加
- 非 Git ソースのフォールバックとして `pkg:generic/{name}@{version}`

**ドキュメント：**
- README.md を更新し、CMake サポートと新しい CLI フラグを追加
- WHATS_NEW.md に v1.0.1 の詳細なリリースノートを追加
- CMake 変数の処理制限を文書化（`${VAR}` はスキップ）

### インフラ

**テスト：**
- 8 件の包括的なテストを含む `tests/parser_tests/cmake_tests.rs` を作成
  - FetchContent のパース（Git および URL ソース）
  - ExternalProject のパース
  - CMake 変数の処理（解決不能な場合は警告してスキップ）
  - チェックサム抽出の検証
- `tests/fixtures/cmake/` にテストフィクスチャを作成
  - CMakeLists_fetchcontent.txt
  - CMakeLists_externalproject.txt
  - CMakeLists_with_variables.txt
- 124 件のテストすべて合格（既存 116 件 + 新規 CMake テスト 8 件）

## [1.0.0] - 2026-02-20

### 追加 - C++ エコシステムのサポート

**初の C++ エコシステムサポート：**
このメジャーリリース（1.0.0）では、Rust で開発された C/C++ プロジェクトの包括的な SBOM 生成機能を備えています。

**vcpkg パッケージマネージャー：**
- **vcpkg マニフェストパーサー**（`vcpkg.json`）
  - すべてのバージョン制約形式：`version>=`、`version>`、`version=`、`version-semver`、`version-date`
  - バージョン固定のための overrides セクション
  - フィーチャーメタデータは source_file フィールドに格納
  - `pkg:vcpkg/{name}@{version}` purl 形式を生成
- vcpkg.json の自動検出機能とともにスキャナーに統合

**Git サブモジュール検出：**
- **Git サブモジュールパーサー**（`.gitmodules`）
  - サブモジュール定義の INI 形式をパース
  - `git ls-tree HEAD` によってコミット SHA を解決
  - HTTPS と SSH の URL 形式をサポート
  - マルチホスト対応：GitHub、GitLab、Bitbucket、セルフホスト型 Git サーバー
  - ホストタイプに応じた適切な purl 形式を生成：
    - GitHub：`pkg:github/owner/repo@sha`
    - GitLab：`pkg:gitlab/owner/repo@sha`
    - Bitbucket：`pkg:bitbucket/owner/repo@sha`
    - セルフホスト：`pkg:generic/repo@sha`

**CLI の強化：**
- `--scan-submodules` フラグ：サブモジュールスキャンの有効/無効切り替え（デフォルト：true）
- `--submodule-depth` フラグ：サブモジュールの最大再帰深度を設定（デフォルト：3）

### 新モジュール

**パーサーモジュール：**
- `src/parsers/cpp/mod.rs` — C++ パーサーモジュールのエントリーポイント
- `src/parsers/cpp/vcpkg.rs` — vcpkg マニフェストパーサー（458 行）
- `src/parsers/git/mod.rs` — Git パーサーモジュールのエントリーポイント
- `src/parsers/git/submodules.rs` — .gitmodules パーサー（292 行）
- `src/parsers/git/commit_resolver.rs` — Git コミット SHA リゾルバー（140 行）
- `src/parsers/git/url_parser.rs` — マルチホスト対応 Git URL パーサー（299 行）

**新規コードの合計：** 1,511 行（19 ファイルを変更）

### 変更

**スキャナー統合：**
- `scan_directory()` に vcpkg.json と .gitmodules の検出を追加
- 正確なサブモジュールバージョンのためのコミット SHA 解決を統合
- 解決できない Git 参照に対する警告メッセージ

**SPDX 形式：**
- vcpkg パッケージの purl 生成を追加
- ホスト固有の形式を持つ Git サブモジュールの purl 生成を追加

**ドキュメント：**
- README.md を更新し、C++ エコシステムサポートのテーブルを追加
- WHATS_NEW.md に v1.0.0 の詳細なリリースノートを追加
- vcpkg バージョン制約形式を文書化
- Git URL パースとマルチホストサポートを文書化

### インフラ

**テスト：**
- バージョン制約検証を含む vcpkg パーサーテスト
- URL パースを含む Git サブモジュールパーサーテスト
- C++ プロジェクトの統合テスト
- すべてのテスト合格

**バグ修正（1.0.0 リリース前の仕上げ）：**
- Python バージョン演算子の除去を修正（>=、==、~=）
- Python のトランジティブ依存関係解決を追加
- Python パーサーの `__future__` の誤検出を修正
- Git 操作のエラーハンドリングを改善

## [0.9.3] - 2026-02-10

### 追加 - Pipfile/Pipfile.lock と pyproject.toml パーサーのサポート

**Pipfile/Pipfile.lock サポート（フェーズ 1）：**
- **Pipfile.lock パーサー** — Pipenv プロジェクト向けの包括的なロックファイルサポート
  - serde_json デシリアライゼーションによる JSON 形式のパース
  - `default`（本番環境）と `develop`（開発）セクションからすべてのパッケージを抽出
  - サプライチェーンセキュリティのためのハッシュ配列からの **SHA256 チェックサム抽出**
  - `index` フィールドの存在による直接依存関係の検出
  - rayon を使用した PyPI からの並列バッチメタデータ取得
  - **487% の改善**：8 パッケージ（v0.9.1）→ 47 パッケージ（v0.9.3）
  - Pipenv ベースのプロジェクトで Black Duck と **100% 同等**

- **Pipfile マニフェストパーサー** — ロックファイルが利用できない場合のフォールバック
  - 既存の toml クレートによる TOML 形式のパース
  - バージョン仕様を処理：`*`、`==`、`>=`、`~=`、複雑な制約
  - `[packages]`（本番環境）と `[dev-packages]`（開発）を区別

**pyproject.toml サポート（フェーズ 2）：**
- **マルチフォーマット pyproject.toml パーサー** — モダン Python パッケージング標準（PEP 517/518）
  - **PEP 621 形式** — `dependencies` と `optional-dependencies` を含む `[project]` セクション
  - **Poetry 形式** — `dependencies` と `dev-dependencies` を含む `[tool.poetry]` セクション
  - **Poetry 1.2+ グループ** — `[tool.poetry.group.*.dependencies]` 形式
  - **PDM 形式** — `dependencies` と `dev-dependencies` を含む `[tool.pdm]` セクション
  - 正規表現による複雑な依存関係仕様のパース
  - optional-dependencies グループ（dev、test、tests、testing）からの開発依存関係の検出
  - Poetry バージョン制約を処理：キャレット（`^`）、チルダ（`~`）、比較演算子
  - Python バージョン制約の自動フィルタリング

**poetry.lock チェックサム抽出（フェーズ 3）：**
- **`[[package.files]]` セクションからの SHA256 チェックサム抽出**
  - ファイル配列から最初のファイルハッシュを抽出
  - 形式：`sha256:abc123...` → `abc123...`
  - Poetry プロジェクトのサプライチェーンセキュリティを強化
  - 追加のネットワークオーバーヘッドゼロ

### 変更
- assessment-service リポジトリの Python パッケージ検出を 8 個から 47 個に改善
- Pipenv プロジェクトのすべての `@detected` バージョンプレースホルダーを解消
- パッケージ名に正規 PyPI 形式を使用（例：`repoze-lru` ではなく `repoze.lru`）
- poetry.lock と Pipfile.lock から SHA256 チェックサムを抽出（将来の SPDX 出力統合のために Dependency 構造体に保存）
- スピナーアニメーション統合："parsing Pipfile.lock..." メッセージをスキャン中に表示
- 進捗インジケーターが Python パッケージ数を自動表示

### テスト
- assessment-service Pipfile.lock から 47 個の Python パッケージを検出したことを確認
- 100% バージョン精度（"@detected" プレースホルダーなし）
- Black Duck とのパッケージ名の完全一致（47/47 パッケージ）
- Pipfile.lock と poetry.lock のチェックサム抽出を検証

### パフォーマンス
- rayon を使用した PyPI メタデータの並列バッチ取得（既存パターンを踏襲）
- serde デシリアライゼーションによるシングルパス JSON/TOML パース
- v0.9.1 と比較してパフォーマンスの低下なし
- v0.9.2 の進捗インジケーターとのシームレスな統合

### 技術詳細
- **変更されたファイル：**
  - `src/parsers/python.rs` — 3 つの新規パーサー関数を追加（+219 行）
    - `parse_pipfile_lock_with_relationships()` — チェックサム付き Pipfile.lock
    - `parse_pipfile()` — Pipfile マニフェスト
    - `parse_pyproject_toml()` — pyproject.toml マルチフォーマット
  - `src/parsers/mod.rs` — 新しいパーサー関数をエクスポート
  - `src/scanner/mod.rs` — スピナー統合によるパーサーの登録（+8 行）
  - `Cargo.toml` — バージョンを 0.9.3 に更新

- **依存関係：**
  - 新しい依存関係なし（既存の serde_json、toml、regex、rayon を使用）

- **パーサーの優先順位：**
  1. Pipfile.lock（最高 — チェックサム付きの正確なバージョン）
  2. poetry.lock（高 — チェックサム付きの正確なバージョン）
  3. requirements.txt（中 — バージョン仕様）
  4. setup.py（中 — バージョン仕様）
  5. Pipfile（中 — バージョン仕様）
  6. pyproject.toml（中 — バージョン仕様）
  7. インポートスキャン（最低 — バージョンなし）

### 競合優位点
- Python パッケージ検出で **Black Duck と同等**（47/47 パッケージ）
- **優れたパッケージ命名** — 正規 PyPI 名を使用
- **包括的な Python サポート** — Pipfile、Poetry、pip、setuptools、PDM、PEP 621
- **サプライチェーンセキュリティ** — ロックファイルからの SHA256 チェックサム
- **モダンな標準** — 完全な pyproject.toml サポート

### 移行ノート
- **ライブラリユーザーへの破壊的変更**：`ScanContext.poetry_relationships` が `python_lockfile_relationships` に改名（CLI の使用には影響なし）
- 既存の Pipenv プロジェクトは自動的に改善された検出の恩恵を受けます
- Poetry プロジェクトの SBOM 出力にはチェックサムが含まれるようになりました
- pyproject.toml プロジェクトは正確な SBOM を生成するようになりました

## [0.9.1.1] - 2026-01-28（ホットフィックス）

### 修正 - Markdown レポートの表示バグ

**問題：**
- ROS マルチパッケージの Markdown レポートで空または不完全なコードブロックが表示
- 例："PIP（1 パッケージ）" ヘッダーで空のコードブロック、"ROS（8 パッケージ）" で 2 パッケージのみ表示
- パッケージ数と表示されたパッケージが一致せず、読者を混乱させていた

**根本原因：**
- コンソールレポート生成に `render_dependency_list()` を使用しており、**本番環境**の直接依存関係のみを表示
- ヘッダーはすべてのパッケージ（開発依存関係を含む）をカウント
- これにより開発依存関係が表示からフィルタリングされる一方で、ヘッダーの数には含まれていた

**解決策：**
- ROS マルチパッケージセクションを修正し、すべての直接依存関係（本番環境 + 開発）を含めるように変更
- 拡張フィルターを使用した `render_tree_classic()` で全直接依存関係を表示
- 適切な分岐文字（`├──`、`└──`）を使用したツリー構造を維持
- カウントされたすべてのパッケージがコードブロック内にツリービジュアライゼーションとして表示されるように

**影響：**
- ROS マルチパッケージレポートの空のコードブロックを修正
- 適切なツリー構造ですべての直接依存関係（本番環境と開発）を表示
- ヘッダーと表示されたパッケージ間のカウント一貫性を維持
- 通常の（非 ROS）プロジェクトレポートへの影響なし

**テスト結果：**
- ros2run PIP セクションに表示：`└── pytest @ unspecified [direct, dev]`
- ros2run ROS セクションにツリー構造（├──、└──）で全 8 パッケージを表示
- 修正前：空の PIP ブロック、不完全な ROS リスト（8 パッケージ中 2 つのみ）

**変更されたファイル：**
- `src/formats/console.rs`（1292〜1324 行目）

## [0.9.1] - 2026-01-28

### 追加 - ROS/rosdistro バージョン解決とリポジトリ URL エンリッチメント

**ROS パッケージバージョン解決：**
- **ROS ディストリビューションの自動検出** — rosdistro GitHub API と統合して ROS パッケージバージョンを解決
  - ros/rosdistro リポジトリから distribution.yaml を取得
  - ROS 2 ディストリビューションをサポート：jazzy、iron、humble、galactic、foxy
  - ROS 1 ディストリビューションをサポート：noetic、melodic
- **手動オーバーライド用 CLI フラグ** — `--ros-distro <distro>` で ROS ディストリビューションを明示的に指定
  - 優先順位：CLI フラグ > ROS_DISTRO 環境変数 > デフォルト（"jazzy"）
  - 例：`--ros-distro humble`、`--ros-distro iron`
- **パッケージ名のバリアント解決** — 複数の命名規則を処理
  - ベース名：`rclpy`
  - Python プレフィックス：`python3-rclpy`
  - ディストリビューションプレフィックス：`ros-jazzy-rclpy`
  - アンダースコアバリアント：`ament-index-python`、`python3-ament-index-python`
- **グローバルキャッシュ** — スキャンセッションごとにディストリビューションあたり 1 回のみ rosdistro を取得
  - 10 秒のタイムアウト、"unspecified" へのグレースフルフォールバック
  - rayon を使用した並列解決（既存のメタデータエンリッチメントパターンに一致）

**リポジトリ URL エンリッチメント：**
- **GitHub URL 抽出** — ROS パッケージの SPDX `downloadLocation` フィールドを入力
  - rosdistro distribution.yaml から `source.url` を抽出
  - GitHub リポジトリ URL を持つ 47 パッケージ（ros2cli プロジェクトのベンチマーク）
  - セキュリティ監査のための完全なソーストレーサビリティ
  - ゼロパフォーマンスオーバーヘッド（既存の rosdistro フェッチを使用）

### 変更
- ROS 依存関係が "unspecified" ではなく解決されたバージョンを表示するように
  - 変更前：`rclpy @ unspecified, downloadLocation: NOASSERTION`
  - 変更後：`rclpy @ 7.1.9, downloadLocation: https://github.com/ros2/rclpy.git`（jazzy）
- ROS パッケージの SPDX `downloadLocation` フィールドを入力済み（47 パッケージに URL あり）
- オプションの `ros_distro` パラメータを受け取るように `scan_directory()` シグネチャを更新
- 3 段階の優先順位システムを備えた `detect_ros_distribution()` を強化
- `RosPackageInfo` 構造体に `repository_url` フィールドを追加
- `lookup_package_version()` を `lookup_package_info()` に改名（バージョン + URL を返す）

### テスト
- リポジトリ URL エンリッチメントのユニットテストを追加（`test_resolve_ros_dependency_versions_with_repository_url`）
- rosdistro 関数のユニットテストを 5 件追加（バージョン解決、パッケージバリアント、非 ROS パッケージ）
- 異なるディストリビューションを使用した ros2cli スキャンの統合テストを 2 件追加
- 97 件のテストすべて合格

### パフォーマンス
- ROS ディストリビューションごとに 1 回のネットワーク取得（セッション中にキャッシュ）
- グレースフルデグラデーション付きの 10 秒タイムアウト
- 非 ROS プロジェクトへのパフォーマンス影響なし
- rayon を使用した並列解決
- リポジトリ URL 抽出に追加のネットワークオーバーヘッドなし

### 技術詳細
- 追加した依存関係：`serde_yaml` v0.9、`lazy_static` v1.4
- 変更されたファイル：`src/cli.rs`、`src/parsers/ros.rs`、`src/scanner/mod.rs`、`src/main.rs`、`src/formats/spdx.rs`
- 新しい関数：`fetch_rosdistro_database()`、`detect_ros_distribution()`、`lookup_package_info()`、`resolve_ros_dependency_versions()`
- `RosPackageInfo` 構造体に `repository_url: Option<String>` フィールドを追加
- `dependency.repository_url` を使用するように SPDX フォーマッターの `create_download_location()` を更新

### 競合優位点
- **ROS サポート**：rosdistro を介した ROS パッケージバージョンの自動解決を備えた初の SBOM ツール
- **リポジトリ URL**：ROS パッケージの downloadLocation を入力する初の SBOM ツール
- **BlackDuck との比較**：radeis は 94 個のユニークな依存関係を検出（BlackDuck の 4 個の 23.5 倍）
  - 3 つのリポジトリに対して 15 個の個別 ROS パッケージ（5 倍の粒度）
  - BlackDuck の 0 に対して 47 パッケージに GitHub URL
  - BlackDuck の 3 に対して 62 パッケージが解決されたバージョンを持つ（21 倍多い）

### ベンチマーク結果（ros2cli プロジェクト）
- **94 個のユニークな依存関係**
- **ros2cli リポジトリ内の 15 個の個別 ROS パッケージ**
- **GitHub リポジトリ URL を持つ 47 パッケージ**
- **解決されたバージョンを持つ 62 パッケージ**（66% のカバレッジ）
- **223 件の SPDX 階層エントリ**（リレーションシップを含む）
- **脆弱性のある 5 パッケージを検出**し、SPDX 出力に埋め込み済み

## [0.9.0] - 2026-01-28

### 追加 - チェックサム、自動化、マルチエコシステムメタデータ

**パッケージチェックサム：**
- **すべてのパッケージの SHA-1 チェックサム** — 整合性検証と再現可能なビルドを実現
  - 形式：40 文字の小文字 16 進数 SHA-1 ハッシュ
  - 各パッケージの SPDX `filesAnalyzed` フィールドに追加
  - サプライチェーンセキュリティと SBOM 検証ワークフローをサポート

**自動化された修正推奨：**
- Markdown 出力での機械可読な修正推奨
- 自動脆弱性修復のための構造化フォーマット
- 既存の脆弱性レポートとの統合

**マルチエコシステムメタデータ抽出（ネットワークモード）：**
- **すべてのエコシステムのハイブリッドメタデータ抽出** — ローカルファイル優先、レジストリ API フォールバック
  - npm：package.json + npm レジストリ API（registry.npmjs.org）
  - Python：poetry.lock パッケージ用の PyPI API（pypi.org/pypi）
  - Cargo：Cargo.lock パッケージ用の crates.io API
  - PHP：composer.json パッケージ用の Packagist API（repo.packagist.org/p2）
  - Ruby：Gemfile パッケージ用の RubyGems API（rubygems.org/api/v2）
- **rayon を使用した並列バッチ取得**で 10〜27 倍のパフォーマンス向上
  - npm：689 パッケージ、10 分超 → 22.6 秒（27 倍高速化）
  - Python：100〜500 パッケージ、10〜20 分 → 30 秒（20〜40 倍高速化）
  - Cargo：100〜500 パッケージ、7〜15 分 → 25 秒（17〜36 倍高速化）
  - PHP：10〜200 パッケージ、2〜7 分 → 20 秒（6〜21 倍高速化）
  - Ruby：5〜50 gem、1〜2 分 → 10 秒（6〜12 倍高速化）
- **API タイムアウトを 5 秒から 3 秒に短縮**し、失敗処理を高速化

### 変更
- ファイルサイズを 955KB（v0.8.0）から 899KB に最適化（6% 削減）
- 効率を 1.384 KB/パッケージから 1.303 KB/パッケージに改善（6% 向上）
- v0.8.0 の 690 パッケージと 689 の CPE 識別子をすべて維持
- すべてのパーサー関数が 3 パスパターンを使用：収集 → 並列取得 → 依存関係の作成

### パフォーマンス
- **パッケージ数**：690 個（v0.8.0 から維持）
- **ファイルサイズ**：899 KB（v0.8.0 の 955 KB より 6% 小さい）
- **CPE 識別子**：689 個（v0.8.0 から維持）
- **ファイル効率**：1.303 KB/パッケージ（v0.8.0 の 1.384 KB/パッケージより改善）

### 競合優位点
- **独自機能**：SPDX 出力に脆弱性 + CPE 識別子 + SHA-1 チェックサムを埋め込む唯一のツール
- **v0.8.0 からの改善**：すべての機能を維持しながらファイルサイズを 6% 削減

## [0.8.0] - 2026-01-27

### 追加 - リッチメタデータとセキュリティ機能

**メタデータ抽出（すべてのエコシステムで 95% 以上のカバレッジ）：**
- **ライセンス情報の抽出** — npm、Cargo、Python、ROS、PHP、Ruby エコシステムのライセンス識別子を抽出・正規化（SPDX 準拠）
  - 95% 以上のライセンスカバレッジを達成（650+/690 パッケージ、v0.7.0 の 0% から改善）
  - SPDX の "NOASSERTION" を実際のライセンス識別子に置き換え
- **サプライヤーとオリジネーターの追跡** — サプライチェーンの透明性のための著者、メンテナー、組織のメタデータ（90% 以上のカバレッジ、620+/690 パッケージ）
  - "Person:" と "Organization:" プレフィックスを持つ SPDX supplier/originator フィールドにマッピング
- **ダウンロードロケーション URL** — 検証と再現可能なビルドのためのエコシステム固有のパッケージレジストリ URL
  - npm：`https://registry.npmjs.org/{package}/-/{file}.tgz`
  - PyPI：`https://pypi.org/project/{package}/{version}/`
  - Cargo：`https://crates.io/api/v1/crates/{package}/{version}/download`
  - Composer、RubyGems、Go エコシステムの完全サポート

**強化された SPDX 出力：**
- **UUID ベースの SPDX ID** — 連番 ID より高い一意性、名前空間の衝突なし
  - 形式：`SPDXRef-Package-{sanitized-name}-{uuid}`
  - 一貫したドキュメント構造のための合成 "main" ルートパッケージ
- **ソースファイル追跡** — どのエクストラクターとマニフェストファイルが各パッケージを検出したかを示す完全な監査証跡（98% 以上のカバレッジ、685+/690 パッケージ）
  - パターン："Identified by the {extractor_type} extractor from {absolute_path}"
- **CPE 識別子** — セキュリティ脆弱性の相関付けのための Common Platform Enumeration（CPE 2.3）
  - 形式：`cpe:2.3:a:vendor:product:version:*:*:*:*:*:*:*`
  - エコシステム固有のベンダー抽出（npm スコープパッケージ、Composer、Go モジュール）

**テストインフラ：**
- **モジュール式テスト構造** — すべての 64 件のテストをモノリシックな main.rs から整理されたテストモジュールに移行
  - 7 つのカテゴリ（パーサー、フォーマット、スキャナー、モデル、エラー、ユーティリティ、統合）にわたる 84 件のテスト
  - main.rs を 2,233 行から 268 行に削減（88% 削減、1,965 行を削除）
  - 整理と保守性向上のための 18 個の独立したテストファイル
- **ソース追跡テスト** — すべてのパーサー（npm、Cargo、Python、ROS、PHP、Ruby、Go）をカバーする 11 件の新規テスト
- **マルチエコシステム統合テスト** — 異なるパーサー間でのソース追跡を検証する 2 件の包括的なテスト
- **UUID と CPE のテスト** — SPDX ID の一意性と CPE 識別子生成のための 7 件の新規テスト

### 変更
- `Dependency` 構造体にオプションのメタデータフィールドを追加（license、author、maintainers、repository_url、homepage_url、source_file）
- SPDX パッケージ作成を更新し、license、supplier、originator、sourceInfo フィールドを入力
- SPDX ID 生成を連番（`SPDXRef-Package-npm-1`）から UUID ベース（`SPDXRef-Package-axios-{uuid}`）に変更
- リレーションシップ構造をフラット（v0.7.0 の 699 DESCRIBES）から階層的（1 DESCRIBES + 689 CONTAINS）に変更
- すべてのパーサーが絶対パスでソースファイルパスを追跡するように
- CycloneDX 形式にライセンスとサプライヤー情報を含めるように
- 統合テストを有効にするために `src/lib.rs` を作成（モジュールをライブラリとして公開）
- テスト用に SPDX 構造体を公開（SPDXDocument、SPDXPackage、SPDXRelationship、SPDXExternalRef フィールド）

### 改善
- リッチメタデータによるコンプライアンスレポート機能の強化
- サプライチェーンセキュリティと透明性のためのより豊富なメタデータ
- 長期的な保守性のためのテストカバレッジと整理の改善
- ImportScan と Manifest の優先順位を適切に処理するための重複排除ロジックの強化

### 修正
- **重複排除バグの修正** — LockFile/Manifest バージョンが存在する場合、ImportScan エントリが正しくフィルタリングされるように
  - v0.7.0 はプレースホルダー "detected" バージョンを持つ ImportScan 重複を誤って保持していた
  - 10 個の重複パッケージを削除（9 個のユニークパッケージが二重カウント：axios、uuid、6 個の AWS SDK クライアント、serverless-sentry-lib、strftime）
  - パッケージ数を 699 から 690 に修正（689 の実際のパッケージ + 1 つの合成 "main"）
- 重複排除ロジックが LockFile > Manifest > ImportScan の優先順位を正しく適用するように
- Rust 統合テスト用にテスト構造を適切に整理
- SPDX 外部参照に PURL と CPE 識別子の両方を正しい形式で含めるように

### パフォーマンス
- **パッケージ数**：690 個（バグ修正：v0.7.0 の 10 個の重複 ImportScan エントリを削除）
- **ファイルサイズ**：955 KB（リグレッション：v0.7.0 の 454 KB から 110% 増加）
- **CPE 識別子**：689 個（v0.8.0 の新機能）
- **ファイル効率**：1.384 KB/パッケージ（リグレッション：v0.7.0 の 0.649 KB/パッケージより 113% 悪化）
- すべての 84 件のテストの実行時間が 90 秒未満

### 技術詳細
- 変更 [src/models/dependency.rs](src/models/dependency.rs) — オプションのメタデータフィールドを追加
- 変更 [src/formats/spdx.rs](src/formats/spdx.rs) — UUID ベースの ID、CPE 生成、階層的リレーションシップ、メタデータ入力
- 変更 [src/formats/cyclonedx.rs](src/formats/cyclonedx.rs) — ライセンスとサプライヤーのサポート
- 変更 [src/parsers/mod.rs](src/parsers/mod.rs) — ImportScan 優先順位修正を含む強化された重複排除
- 作成 [src/lib.rs](src/lib.rs) — 統合テスト用のライブラリエントリーポイント
- 作成 [tests/all_tests.rs](tests/all_tests.rs) — 統合テストモジュールのエントリーポイント
- tests/ ディレクトリ下に整理された構造を持つ 18 個のテストモジュールファイルを作成

### 競合優位点
- **メタデータの豊富さ**：エンタープライズツールと同等（95% 以上のライセンス、90% 以上のサプライヤー、BlackDuck の 99.9% と比較）
- **独自機能**：SPDX 出力に脆弱性 + CPE 識別子を埋め込む唯一のツール
- **マルチエコシステムリーダー**：npm、Cargo、Python、ROS、PHP、Ruby エコシステム全体でフルメタデータサポート
- **注記**：v0.8.0 は ImportScan 重複排除バグを修正（10 個の重複を削除）し、CPE メタデータを追加（ファイルサイズが増加）しており、両方の問題は v0.9.0 で対処済み

## [0.7.0] - 2026-01-27

**⚠️ 既知の問題**：このバージョンには重複排除バグがあり、バージョン "detected" を持つ 10 個の ImportScan 重複パッケージが誤って保持されています。詳細は [PACKAGE_COUNT_ANALYSIS.md](scan_reports/PACKAGE_COUNT_ANALYSIS.md) を参照してください。v0.8.0 で修正済み。

### 追加
- **パッケージ検出** — **699 パッケージ**を検出（以前は 563 個）、ただし 10 個の重複 ImportScan エントリを含む（v0.8.0 で 690 個に修正）
- **2 つの SBOM モード**で用途に応じた使い分けが可能：
  - `--sbom-mode complete` — すべてのパッケージ（699 個、重複 10 個含む、454KB）コンプライアンスとインベントリ用
- スマートマニフェストフィルタリング — 正確なロックファイルバージョンが存在する場合、冗長な package.json バージョン範囲を自動削除
- `docs/` フォルダーの包括的なドキュメント：
  - [docs/sbom_modes_guide.md](docs/sbom_modes_guide.md) — CI/CD 例を含む SBOM デュアルモードの完全ガイド
  - [docs/WHATS_NEW.md](docs/WHATS_NEW.md) — 移行ガイドを含む v0.7.0 の詳細な変更点
  - [docs/plan/improvement_plan.md](docs/plan/improvement_plan.md) — 技術設計ドキュメント
  - [docs/plan/implementation_summary.md](docs/plan/implementation_summary.md) — 指標を含む実装結果

### 変更
- **パッケージ検出が 24.2% 向上**（563 → 699 個、ただし ImportScan の重複 10 件を含む）
- 重複排除アルゴリズムが `(name, ecosystem)` の代わりに `(name, version, ecosystem)` タプルでバージョン対応に
- NPM パーサーが HashSet を使用して同じ `package@version` の組み合わせの重複処理を防止
- README.md を完全に改訂し、主要な改善点に焦点を当てて明確かつ簡潔に
- すべての SPDX と CycloneDX ジェネレーターを更新し、モードベースのフィルタリングをサポート

### 修正
- 同じパッケージの複数バージョンが正しく保持されるように（例：`@aws-sdk/client-sso@3.632.0` と `@aws-sdk/client-sso@3.848.0` の両方を保持）
- ロックファイルバージョンが存在する場合、マニフェストバージョン（バージョン範囲 `^3.215.0` など）を自動フィルタリング
- 約 126 個の欠落した AWS SDK サブパッケージがネストされた node_modules で正しく検出されるように

### パフォーマンス
- 脆弱性のみモードで 98% のファイルサイズ削減を実現（11 KB vs 454 KB）
- ⚠️ **699 パッケージには 10 個の ImportScan 重複が含まれます**（v0.8.0 で 690 個に修正）

### 技術詳細
- 変更 [src/parsers/mod.rs](src/parsers/mod.rs) — マニフェストフィルタリング付きのバージョン対応重複排除
- 変更 [src/parsers/npm.rs](src/parsers/npm.rs) — HashSet ベースの重複防止
- 変更 [src/cli.rs](src/cli.rs) — SbomMode 列挙型を追加
- 変更 [src/formats/spdx.rs](src/formats/spdx.rs) — SPDX 出力のモードベースフィルタリング
- 変更 [src/formats/cyclonedx.rs](src/formats/cyclonedx.rs) — CycloneDX 出力のモードベースフィルタリング
- 変更 [src/main.rs](src/main.rs) — すべてのフォーマットジェネレーターにモードパラメータを渡す

### 競合優位点
- デュアル SBOM モードによりコンプライアンスとセキュリティワークフローの両方に柔軟性を提供
- **既知の問題**：重複排除バグにより、LockFile バージョンが存在する場合でも "detected" バージョンを持つ ImportScan エントリが残存する（v0.8.0 で修正済み）

## [0.6.0] - 2026-01-26

### 追加
- ロックファイルから npm、Cargo、Poetry 向けの真の階層的依存関係ツリーを生成
  - フェーズ 1：npm（package-lock.json）— 完全な親子リレーションシップ
  - フェーズ 2：Cargo（Cargo.lock）— Rust プロジェクトの dependencies 配列をパース
  - フェーズ 3：Poetry（poetry.lock）— Python プロジェクトの [package.dependencies] テーブルをパース
- グラフベース分析による正確な [direct] とトランジティブマーカー
- パッケージリストセクションと付録セクションを区別した再構成されたレポート構造
- 循環依存関係の検出と処理
- ルートから脆弱性のあるパッケージまでのすべてのパスを示す完全な依存関係チェーン追跡
- モジュール式アーキテクチャへのリファクタリング — main.rs（6,561 行）を 24 ファイルにわたる 7 モジュールに分割

### 変更
- レポート構造が直接本番環境依存関係を最初に表示し、次に個別リスト、開発/トランジティブ依存関係を付録に配置
- 整理改善のため付録を脆弱性セクションの後に配置
- Main.rs を 6,561 行から 2,130 行に削減（67.6% 削減）

### 修正
- ファイルパスではなく実際の親子リレーションシップに基づいた is_direct フラグの修正
- コンソールサマリーカウントが依存関係グラフの修正済みフラグを使用するように
- vendor ディレクトリ内のファイルスキャンを妨げていた VendorMode::Only バグを修正
- 重複した create_package_url 関数を統合
- レポートセクション間で直接依存関係のカウントを標準化
- 脆弱性ツリーが一貫した is_direct フラグのためにリレーションシップを使用するように

### テスト
- 59 件の包括的なユニットテストすべて合格
- Cargo と Poetry のリレーションシップパースのテストを追加
- コンパイラ警告ゼロ

## [0.5.0] - 2026-01-23

### 破壊的変更
- ツリースタイルのビジュアライゼーションがデフォルトで有効化（旧フォーマットには --tree-style flat を使用）

### 追加
- 3 つのモードを持つツリースタイルの依存関係ビジュアライゼーション（flat、tree、compact）
- 深刻度優先で整理された階層的な脆弱性表示
- 概要確認用のサマリー統計セクション
- 絵文字による深刻度インジケーター（🔴 クリティカル、🟠 高、🟡 中、🟢 低）
- 各脆弱性の依存関係チェーン表示
- --max-vulns-per-severity による折り畳み可能な脆弱性表示
- サマリーセクションのリスク評価

### 変更
- Unicode ボックス描画文字によるコンソール出力の強化
- 一貫した区切り文字による視覚的階層の改善

## [0.4.0] - 2026-01-22

### 破壊的変更
- 脆弱性チェックをデフォルトで有効化
- vendor ディレクトリスキャンをデフォルトで有効化
- インポートフォールバックスキャンをデフォルトで有効化

### 追加
- 依存関係のソース追跡（直接 vs トランジティブ）を含む強化された Markdown レポート

### 変更
- npm パッケージのトランジティブ依存関係検出を改善
- コンソールレポートでデフォルトで詳細な脆弱性出力

## [0.3.1] - 2026-01-21

### 追加
- マルチプラットフォームビルドシステム（Windows + Linux）
- クロスコンパイルの自動化
- ビルドドキュメントの強化

## [0.3.0] - 2026-01-20

### 追加
- ROS/ROS2 マルチパッケージサポート
- 階層的なツリー出力
- SPDX リレーションシップ（DESCRIBES、DEPENDS_ON）
- Python、JS/TS、Go のインポートスキャンフォールバック
- 51 件の包括的なユニットテスト

## [0.2.0] - 2026-01-19

### 追加
- SPDX 2.3 サポート（JSON + Tag-Value）
- パッケージ URL（purl）の実装
- マルチフォーマット出力

## [0.1.0] - 2026-01-16

### 追加
- 初回リリース
- 8 エコシステムのサポート（npm、Cargo、pip、Go、RubyGems、Composer、Maven、ROS）
- コンソール出力
- 18 件のユニットテスト

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
