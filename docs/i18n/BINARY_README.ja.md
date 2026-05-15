# radeis_sc2sbom - バイナリ配布

ソースコードからソフトウェア部品表（SBOM）を生成するための事前ビルド済みバイナリです。

## 利用可能なバイナリ


### macOS（Apple Silicon - M1/M2/M3）
- **ファイル：** `radeis_sc2sbom-macos-arm64`
- **アーキテクチャ：** ARM64 (aarch64)
- **プラットフォーム：** macOS 11.0 以降

### macOS（Intel）
- **ファイル：** `radeis_sc2sbom-macos-x86_64`
- **アーキテクチャ：** x86_64
- **プラットフォーム：** macOS 10.12 以降

### Linux（x86_64 静的）
- **ファイル：** `radeis_sc2sbom-linux`
- **アーキテクチャ：** x86_64
- **プラットフォーム：** あらゆる Linux x86_64（静的バイナリ、glibc 不要 — Ubuntu 22.04+、24.04+、Alpine などで動作）

### Windows（x86_64）
- **ファイル：** `radeis_sc2sbom-windows.exe`
- **アーキテクチャ：** x86_64
- **プラットフォーム：** Windows 10 以降

## インストール

### macOS / Linux

1. お使いのプラットフォームに対応したバイナリをダウンロードします
2. 実行権限を付与します：
   ```bash
   chmod +x radeis_sc2sbom-macos-arm64
   ```
3. （オプション）PATH が通ったディレクトリに移動します：
   ```bash
   sudo mv radeis_sc2sbom-macos-arm64 /usr/local/bin/radeis_sc2sbom
   ```

### Windows

1. `radeis_sc2sbom-windows.exe` をダウンロードします
2. コマンドプロンプトまたは PowerShell から実行します
3. （オプション）アクセスを容易にするために PATH に追加します

## クイックスタート

**注意：** `<binary-name>` をお使いのプラットフォームに対応したバイナリ名に置き換えてください：
- macOS ARM：`radeis_sc2sbom-macos-arm64`
- macOS Intel：`radeis_sc2sbom-macos-x86_64`
- Linux：`radeis_sc2sbom-linux`
- Windows：`radeis_sc2sbom-windows.exe`

```bash
# 基本的な使い方 - カレントディレクトリをスキャン
./<binary-name> --path .

# 特定のプロジェクトをスキャン
./<binary-name> --path /path/to/project

# SPDX 形式で生成
./<binary-name> --path . --format spdx
```

## 主なオプション

| オプション | 説明 | 例 |
|-----------|------|-----|
| `--path <PATH>` | スキャン対象のパス（デフォルト：カレントディレクトリ） | `--path ./my-project` |
| `--format <FORMAT>` | 出力形式：console、spdx、cyclonedx、all | `--format spdx` |
| `--output <DIR>` | 出力ディレクトリ（デフォルト：./out） | `--output ./sbom-reports` |
| `--tree-style <STYLE>` | 依存関係ツリーのスタイル：classic、compact、flat | `--tree-style compact` |
| `--vendor` | vendor ディレクトリを含める | `--vendor` |
| `--exclude <PATTERN>` | 除外パターン（複数回指定可能） | `--exclude "test/*"` |
| `--bsw-config <PATH>` | カスタム AUTOSAR BSW モジュール設定（YAML） | `--bsw-config ./bsw.yaml` |
| `--supplier-config <PATH>` | AUTOSAR コンポーネントとサプライヤーのマッピング（YAML） | `--supplier-config ./suppliers.yaml` |

## 対応エコシステム

### ロックファイル（階層的な依存関係ツリーあり）
- **npm** - package-lock.json
- **Cargo**（Rust）- Cargo.lock
- **Poetry**（Python）- poetry.lock

### マニフェストファイル
- **npm** - package.json
- **Cargo**（Rust）- Cargo.toml
- **Python** - requirements.txt、setup.py、pyproject.toml
- **Go** - go.mod
- **Maven**（Java）- pom.xml
- **ROS** - package.xml
- **JavaScript/TypeScript** - ソースコードのインポート

### C / C++（ビルドシステムおよびパッケージマネージャー）
- **Makefile** - GNU Make ビルドファイル
- **Makefile.am** - Automake ビルドファイル
- **configure.ac** - Autotools 設定（pkg-config 検出）
- **.mk files** - アーキテクチャ対応の Make フラグメントファイル
- **pkg-config (.pc files)** - ライブラリ依存関係ディスクリプタ
- **Shared libraries (.so scanner)** - 動的ライブラリ依存関係の検出
- **Vendored 3rd-party** - `3rdparty/`、`3rd_party/`、`third_party/` ディレクトリの検出
- **Conan** - conanfile.txt / conanfile.py
- **vcpkg** - vcpkg.json
- **CMake** - CMakeLists.txt（FetchContent および ExternalProject_Add 経由）

### AUTOSAR
- **`.arxml` ファイル** - 自動検出；BSW コンポーネントをレイヤー・プラットフォーム・サプライヤーで分類
- **BSW モジュール設定** - バンドル済みデフォルト値；`--bsw-config` で上書き可能
- **サプライヤーマッピング** - `--supplier-config` で任意の YAML ファイルを指定

## 使用例

**注意：** 使用例では `<binary-name>` をプレースホルダーとして使用しています。お使いのプラットフォームに対応したバイナリ名に置き換えてください。

### Node.js プロジェクトの完全な SBOM を生成
```bash
./<binary-name> \
  --path ./my-node-app \
  --format all \
  --tree-style classic
```

### Rust プロジェクトをスキャン
```bash
./<binary-name> \
  --path ./my-rust-app \
  --format cyclonedx
```

### テストディレクトリを除外して Python プロジェクトをスキャン
```bash
./<binary-name> \
  --path ./my-python-app \
  --exclude "tests/*" \
  --exclude "venv/*" \
  --format spdx
```

### コンパクトなツリースタイルでコンソールレポートのみ生成
```bash
./<binary-name> \
  --path . \
  --format console \
  --tree-style compact \
  --output ./reports
```

## 出力形式

### Console 形式
人間が読みやすい Markdown レポート（以下を含む）：
- プロジェクトの概要
- エコシステム別の依存関係統計
- 階層的な依存関係ツリー

### SPDX 形式（spdx.json）
業界標準の SBOM 形式。以下と互換性あり：
- SPDX ツールおよびバリデーター
- ライセンスコンプライアンスツール
- サプライチェーンセキュリティプラットフォーム

### CycloneDX 形式（cdx.json）
軽量な SBOM 形式（以下を含む）：
- コンポーネントインベントリ
- 依存関係


## ヘルプ

コマンドラインリファレンスの全文を確認するには：
```bash
./<binary-name> --help
```

## ソースコードと問題報告

- **リポジトリ：** https://github.com/VicOne-RD/radeis_sc2sbom
- **問題報告：** https://github.com/VicOne-RD/radeis_sc2sbom/issues
- **ドキュメント：** 詳細なドキュメントはメインリポジトリの README を参照してください

## バージョン情報

バイナリのバージョンを確認するには：
```bash
./<binary-name> --version
```

## ライセンス

MIT License

Copyright (c) 2026 VicOne Inc.

本ソフトウェアおよび関連ドキュメントファイル（以下「本ソフトウェア」）のコピーを取得したすべての人に対し、使用、複製、変更、統合、公開、配布、サブライセンス、および販売する権利を含むがこれに限定されない、無償かつ制限なしに本ソフトウェアを取り扱う許可を、以下の条件に従って付与します：

上記の著作権表示および本許可表示は、本ソフトウェアのすべてのコピーまたは重要な部分に含める必要があります。

本ソフトウェアは「現状のまま」提供され、商品性、特定目的への適合性、および非侵害性に関する保証を含むがこれに限定されない、明示または黙示のいかなる保証もありません。いかなる場合においても、著者または著作権者は、契約行為、不法行為、またはその他の行為から生じる本ソフトウェアまたはその使用もしくは取引に関連した、いかなるクレーム、損害、またはその他の責任についても責任を負いません。
