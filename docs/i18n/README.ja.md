<a href="../../README.md">English</a> | <a href="README.zh-Hans.md">简体中文</a> | <a href="README.zh-Hant.md">繁體中文</a> | 日本語

<p align="center">
  <img src="../../docs/images/icon.png" alt="Sourcecode to SBOM" width="80">
</p>

# radeis_sc2sbom

**高速 SBOM ジェネレーター。** マルチエコシステム対応に加え、独自の機能を備えています：C/C++ Autotools、ROS 2、Git サブモジュール、CMake ExternalProject、AUTOSAR BSW をサポートする**唯一のツール**です。

## VicOne xZETA で SBOM の価値を最大化

`radeis_sc2sbom` が生成する SBOM は **SPDX 2.3** および **CycloneDX 1.5** に完全準拠しており、業界をリードする車載向け脆弱性・SBOM 管理プラットフォーム **[VicOne xZETA](https://vicone.com/products/xzeta)** に直接インポートできます。


**[VicOne xZETA の詳細はこちら →](https://vicone.com/products/xzeta)**

---

## なぜ radeis_sc2sbom を選ぶのか？

- **最も包括的** — プロジェクトの種類によって競合他社より 2.1%〜58.8% 多くのパッケージを検出
- **5+ の独自機能** — Autotools、ROS 2、Git サブモジュール、CMake ExternalProject、AUTOSAR BSW をサポートする唯一のツール
- **AI モデルサポート** — GGUF・Safetensors 解析、CycloneDX `machine-learning-model` および `pkg:huggingface` PURL 対応
- **標準準拠** — SPDX 2.3（JSON + Tag-Value）および CycloneDX 1.5

## クイックスタート

```bash
# ビルド
git clone <repository-url>
cd radeis_sc2sbom
cargo build --release

# スキャンして全フォーマット生成
./target/release/radeis_sc2sbom --path . --output ./out

# 単一フォーマットをファイルに出力
./target/release/radeis_sc2sbom --path . --format spdx-json --output ./out

# 単一フォーマットを標準出力に出力
./target/release/radeis_sc2sbom --path . --format cyclonedx-json
```

## 対応エコシステム

| カテゴリ | エコシステム | 主要ファイル |
|---------|------------|------------|
| **AI/ML モデル** | Safetensors | `*.safetensors`, `config.json`, `generation_config.json`, `tokenizer_config.json` |
| **AI モデル** | GGUF | `*.gguf`, `config.json`, `README.md` |
| **AUTOSAR** | BSW モジュール + SWC コンポーネント | `*.arxml`、`*.epd`、BSW ディレクトリ、CMake/Makefile トークン；バージョン取得元：REVISION-LABEL + Doxygen ヘッダー — **唯一のツール** |
| **ロボティクス** | ROS/ROS2 | `package.xml`, `setup.py` — **唯一の SBOM ツール** |
| **C/C++** | Autotools | `configure.ac`, `Makefile.am` — **唯一のツール** |
| **C/C++** | CMake | `CMakeLists.txt`, `*.cmake` — ExternalProject **唯一のツール** |
| **C/C++** | Conan / vcpkg / pkg-config | ロックファイル、`.pc`、`conanfile.*` |
| **C/C++** | Meson / Bazel | `meson.build`, `WORKSPACE`, `MODULE.bazel` |
| **C/C++** | Makefile / .mk ファイル | ヒューリスティック解析、変数解決対応 |
| **バージョン管理** | Git サブモジュール | `.gitmodules` (commit SHA 追跡) — **唯一のツール** |
| **Python** | pip / Poetry / Pipenv | `requirements.txt`, `poetry.lock`, `pyproject.toml` |
| **JavaScript** | npm | `package.json`, `package-lock.json`, `yarn.lock` |
| **Rust** | Cargo | `Cargo.toml`, `Cargo.lock` |
| **Java** | Gradle / Maven | `build.gradle`, `build.gradle.kts`, `pom.xml` |
| **Go / Ruby / PHP** | 標準 | `go.mod`, `Gemfile`, `composer.json` |

全エコシステムでライセンス抽出、サプライヤーメタデータ、CPE 識別子をサポート。

## 主なコマンド

```bash
# 全フォーマットで完全スキャン
./target/release/radeis_sc2sbom --path . --output ./out

# プロダクション SBOM（ランタイム + オプション依存のみ）
./target/release/radeis_sc2sbom --path . \
  --production \
  --format spdx-json

# AUTOSAR プロジェクト（サプライヤーマッピング付き）
./target/release/radeis_sc2sbom --path . \
  --supplier-config suppliers.yaml \
  --format cyclonedx-json --output ./out
```


## 主なオプション

```
--path <PATH>                  スキャンするディレクトリ（必須）
--format <FORMAT>              console | spdx-json | spdx-tag-value | cyclonedx-json | all（デフォルト：all）
--output <DIR>                 出力ディレクトリ；単一フォーマット時は省略で標準出力
--production                   ランタイム + オプション依存のみを含める
--scope-filter <SCOPE>         runtime | build | test | dev | optional | provided
--supplier-config <PATH>       AUTOSAR コンポーネント名をサプライヤー文字列にマッピングする YAML
--bsw-config <PATH>            内蔵 AUTOSAR BSW モジュール設定を上書き
--ros-distro <DISTRO>          jazzy | iron | humble（デフォルト：jazzy）
--target-arch <ARCH>           .mk 条件解決のターゲットアーキテクチャ
--scan-submodules              Git サブモジュールスキャン（デフォルト：true）
--scan-c-build-systems         CMake、pkg-config、Autotools、Makefile、.mk（デフォルト：true）
--scan-ai-models               GGUF + Safetensors AI モデルスキャン（デフォルト：true）
--compact-spdx                 SPDX 出力を約 30% 削減
```


完全なリファレンスは [docs/CLI.md](../CLI.md) を参照。

## 新機能



- **スタンドアロン C プロジェクト対応** — パッケージマニフェストがないプロジェクト（例：NIST Juliet テストスイート）は自動検出されてフォールバックモードでスキャンされます
- **内部機能ゲート** — スキャナーは `--features internal` でビルドした場合のみコンパイルされます；公開バイナリは影響を受けません
<!-- END_INTERNAL -->


- **AUTOSAR 検出** — `.arxml` ファイル、BSW/MCAL/RTE ディレクトリ名、またはビルドファイルの `AUTOSAR_VERSION` トークンで AUTOSAR プロジェクトを自動検出
- **AUTOSAR 分類** — BSW コンポーネントに `autosar:layer`、`autosar:platform` を CycloneDX プロパティと SPDX ExternalRef としてタグ付け
- **サプライヤーマッピング** — `--supplier-config <yaml>` でコンポーネント名をサプライヤー文字列にマッピング；`autosar:supplier` を出力（未マッピング時は `NOASSERTION` にフォールバック）
- **単一フォーマットでの `--output`** — `--format spdx-json --output ./out` でファイルに書き込み；`--output` 省略時は標準出力に出力

### v1.0.14 — 信頼性と SBOM 品質

- シンボリックリンク切れの許容 — 壊れたシンボリックリンクでスキャンが中断しない
- Makefile `$(VAR)` フィルタリング — 変数参照が `versionInfo` に漏れない
- C/C++ ライセンス解決 — `.pc` の `License:` フィールド + 24 エントリの既知ライブラリテーブル
- 静的 Linux バイナリ（musl）— glibc 依存なし、Ubuntu 22.04+・Alpine・任意の x86_64 Linux で動作

### 過去のリリース

完全な履歴は [CHANGELOG.md](../../CHANGELOG.md) および [docs/WHATS_NEW.md](../WHATS_NEW.md) を参照。

## ベンチマーク概要

| リポジトリ | radeis パッケージ数 | 競合他社最高 | 優位性 |
|-----------|-----------------|------------|-------|
| curl（C） | 44 | 41（Syft） | +3 — 唯一の Autotools サポート |

詳細は [docs/BENCHMARKS.md](../BENCHMARKS.md) を参照。

## 出力ファイル

デフォルト出力先：`./out/`

```
<project>_report.md          # コンソールレポート（Markdown）
<project>_spdx.json          # SPDX 2.3 JSON
<project>_spdx.spdx          # SPDX 2.3 Tag-Value
<project>_cyclonedx.json     # CycloneDX 1.5 JSON
```

## ドキュメント

- [CLI リファレンス](../CLI.md)
- [使用ガイド](../USAGE.md)
- [新機能](../WHATS_NEW.md)
- [スコープ分類](../SCOPE_CLASSIFICATION.md)
- [アーキテクチャ](../ARCHITECTURE.md)
- [ベンチマーク](../BENCHMARKS.md)

## 動作要件

- Rust 1.70+

## ライセンス

MIT — [LICENSE](../../LICENSE) 参照

## 作者

Amean Lin · William Chang

## サポート

- **バグ報告・機能要望** — [Issue を開く](../../issues)
- **エンタープライズ・製品に関するお問い合わせ** — [VicOne へのお問い合わせ](https://vicone.com/lab_r7/contact-us/)

---

<p align="center">
  Made with ❤️ for supply chain security
</p>
