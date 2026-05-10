<p align="center">
<img src="../assets/icon.png" alt="Episteme" width="60%" />
</p>

<p align="center"><sub>Episteme (συν ταγμα) — ギリシャ語で「組織化された体系」または「識別力」を意味する</sub></p>

<p align="center">オフラインファーストの単一バイナリナレッジグラフ。デザインパターン、リファクタリング手法、ソフトウェアの法則を意味的な関係性を通じて接続します。<br><b>AIエージェントファーストで構築</b> — ソフトウェアエンジニアリングの専門知識をClaude Code、Cursor、その他のMCP互換ツールに直接統合できます。</p>

<p align="center">Rustで記述 · 単一バイナリ · 完全オフライン</p>

<p align="center">
    <a href="https://github.com/epicsagas/Episteme/actions"><img src="https://img.shields.io/github/actions/workflow/status/epicsagas/Episteme/ci.yml?branch=main&label=CI" alt="CI" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.95+-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/episteme"><img src="https://img.shields.io/badge/crates.io-v0.1.0-orange.svg" alt="crates.io" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

<p align="center">
  <a href="../../README.md">English</a> |
  日本語 |
  <a href="README.ko.md">한국어</a> |
  <a href="README.de.md">Deutsch</a> |
  <a href="README.fr.md">Français</a> |
  <a href="README.zh-CN.md">简体中文</a> |
  <a href="README.zh-TW.md">繁體中文</a> |
  <a href="README.pt.md">Português</a> |
  <a href="README.es.md">Español</a> |
  <a href="README.hi.md">हिन्दी</a>
</p>

---

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/features.png">
  <img src="../assets/features.png" align="center" width="100%" alt="Episteme Features Overview" />
</picture>

---

## クイックスタート

> **前提条件:** [rustup](https://rustup.rs)経由でRust 1.95以上 · **Rustがインストールされていない場合:** [Docker](#option-3-docker-rust不要)または[プリビルトバイナリ](#option-4-プリビルトバイナリrust不要)を参照してください。

**1. Rustのインストール（未インストールの場合）**

| OS | コマンド |
|----|---------|
| **macOS / Linux** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Windows** | [`rustup-init.exe`](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe)をダウンロードして実行 |

インストール後、**新しいターミナル**を開いてください（macOS/Linuxの場合は`source "$HOME/.cargo/env"`を実行）。

**2. Epistemeのインストール（初回ビルドは3〜5分）**

```bash
# Install cargo-binstall if missing
cargo install cargo-binstall

# Fast path (downloads prebuilt binary when available)
cargo binstall episteme

# Fallback (build from source)
cargo install --git https://github.com/epicsagas/Episteme
```

**3. データのシード + AIツールの設定**

```bash
epis install claude    # または: cursor, codex, gemini
```

**4. 動作確認**

```bash
epis --version
epis stats
```

以上です。Claude Codeを再起動すればEpistemeツールが使用可能になります。

### 30秒で試す

**方法A — CLI:** プロジェクト内の任意のファイルを指定します。

```bash
epis analyze src/domain/engine.rs
```

```
✓ 2 smells detected in src/domain/engine.rs

  SMELL-07 (Large Class) — RefactoringRanker, 743 lines
  → RF-018 Extract Class          priority 0.89  effort: medium
  → RF-001 Extract Method         priority 0.76  effort: small
  → Violates: LAW-001 Single Responsibility Principle

  SMELL-01 (Long Method) — rank_refactorings(), 58 lines
  → RF-001 Extract Method         priority 0.92  effort: small
  → Violates: LAW-001 SRP, LAW-004 DRY
```

**方法B — Claude Code:** プロジェクト内の任意のファイルを開き、自然な言葉で質問します。

```
Find code smells in this project and suggest refactorings.
```

Epistemeが自動的に起動します — 特別な構文は不要です。説明内容をナレッジグラフにマッピングし、ランク付けされた引用可能な結果を返します。

---

## なぜEpistemeか？

LLMはすでにStrategyパターンが何かを知っています。SOLID原則を暗唱し、GoFパターンを列挙し、コードスメルを説明できます。では、なぜこのプロジェクトが存在するのでしょうか？

**足りないのは知識ではなく、構造化された関連付けられた推論です。**

LLMに「God Objectの修正方法」を尋ねると、もっともらしい回答が得られます。しかし、その回答は会話のたびに変わり、トレーサビリティがなく、問題を根本原因や下流の影響と結びつけません。Epistemeは孤立した事実を走査可能なグラフに変え、すべての推奨事項が根拠を持ち、引用可能で、より広範な設計領域と結びついています。

### 丁寧なLLMプロンプトとどう違うのか？

| | 丁寧に作成されたLLMプロンプト | Episteme + LLM |
|---|---|---|
| 能動的検出 | ユーザーが正しい質問をした場合のみ | 問題の説明に対して自動起動 |
| トークン効率 | 長い説明 + 複数回のフォローアップ | 1回のツール呼び出しで構造化結果を返す |
| 関係性の走査 | 多くても1ホップ、しばしばハルシネーション | マルチホップのグラフ走査、検証済み |
| 相互参照 | 手動、エラーが発生しやすい | 201の意味的関係により自動化 |
| 一貫性 | 会話ごとに変動 | 毎回同じ構造化された回答 |
| 引用可能性 | 「Extract Classを使うべきだと思います」 | 「Extract Class (RF-018)、優先度 0.89」 |
| オフライン / エアギャップ | 最高の結果を得るにはインターネットが必要 | 完全にローカル、単一バイナリ |

### どのような場合に役立つか？

<details>
<summary><b>1. AIエージェントに質問を待たずに能動的に問題を検出させたい場合</b></summary>

MCP統合は問題の説明に対して自動起動します。ユーザーが「このクラスはやりすぎている」と言ったとき、エージェントがGod Objectについて質問する必要はありません — Epistemeが不満を`SMELL-03`にマッピングし、ランク付けされたリファクタリングを提示し、違反を第一原理にまで遡ります。これにより、曖昧な不満が構造化された改善計画に変わります。
</details>

<details>
<summary><b>2. トークン消費を削減したい場合 — 説明に浪費しない</b></summary>

Epistemeなしでは、LLMは「God Objectの修正方法」に対して、スメルの説明、リファクタリングの列挙、SOLID原則の説明、各オプションの解説を行います — 回答ごとに数百のトークンを消費します。Epistemeを使えば、1回のMCPツール呼び出しで`SMELL-03 → RF-018 (0.89) → LAW-001`が返ります。同等の専門知識がトークン予算のほんの一部で得られます。
</details>

<details>
<summary><b>3. 検出だけでなく、修正につながるコード分析が必要な場合</b></summary>

SonarQubeのようなツールはスメルを検出します。LLMはパターンを提案できます。Epistemeはその両方を行い、それらを結びつけます：Long Methodを検出 → 違反している法則を追跡 → それを解決するリファクタリングをランク付け → それらのリファクタリングを強制するパターンを表示。
</details>

<details>
<summary><b>4. 孤立したパターンの知識では不十分で、関係性が必要な場合</b></summary>

Extract Methodが何をするかを知っているのは基本です。それがLong Method (SMELL-01)を*解決し*、それがSingle Responsibility (LAW-001)に*違反し*、それがFacade Pattern (DP-012)によって*強制される*ということを知る — これがLLMが単独では確実に構築できない推論チェーンです。Epistemeの201の意味的関係により、AIエージェントはこれらのパスを決定論的に走査できます。
</details>

<details>
<summary><b>5. アーキテクチャの決定において、意見ではなく証拠が必要な場合</b></summary>

「マイクロサービスを使うべきか？」 — Epistemeはこの質問をConway's Law (LAW-017)、SRP (LAW-001)、Strangler Figパターン (DP-026)に結びつけ、それらがどう関係しているかを示します。決定はブログ記事ではなく、エンジニアリングの法則に遡って追跡可能になります。
</details>

<details>
<summary><b>6. 一貫した引用可能なエンジニアリングアドバイスが必要で、ハルシネーションによる推奨は不要な場合</b></summary>

すべての発見は明示的なエンティティID（`DP-005`、`RF-001`、`LAW-021`）を参照します。推奨事項には優先度スコアと工数見積もりが付きます。同じクエリには常に同じ構造化された回答が返ります。
</details>

<details>
<summary><b>7. エアギャップ環境や制限されたネットワークで作業している場合</b></summary>

Epistemeは完全にオフラインで動作します：単一バイナリ、ローカルSQLiteデータベース、fastembed (ONNX Runtime)によるローカル埋め込み。テレメトリなし、電話ホームなし、外部API呼び出しなし。コードと分析結果はマシンから一切外部に送信されません。
</details>

---

## インストール

### 方法1：ワンコマンド（推奨）

```bash
# 初回ビルドは3〜5分かかります — これは正常です
cargo install --git https://github.com/epicsagas/Episteme
epis install claude    # データのシード + MCPの設定 + エージェントのインストール
```

> `epis install claude`の後、MCPツールとエージェントを表示するには**Claude Codeを再起動**してください。

### 方法2：ソースからビルド

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme && cargo build --release
```

プラットフォームに応じてバイナリを実行してください：

| プラットフォーム | コマンド |
|----------|---------|
| **macOS / Linux** | `./target/release/epis install --local claude` |
| **Windows** | `.\target\release\episteme.exe install --local claude` |

### 方法3：Docker（Rust不要）

```bash
docker-compose up -d
```

MCP設定ファイルに以下を追加してください：

| ツール | 設定ファイルのパス |
|------|-----------------|
| Claude Code | `~/.claude.json` |
| Cursor | `.cursor/mcp.json` |
| VS Code (Copilot) | `.vscode/mcp.json` |

```json
{
  "mcpServers": {
    "episteme": {
      "command": "docker",
      "args": ["exec", "-i", "episteme-api", "episteme", "mcp"]
    }
  }
}
```

### 方法4：プリビルトバイナリ（Rust不要）

[GitHub Releases](https://github.com/epicsagas/Episteme/releases)からプラットフォームに合った最新バイナリをダウンロードしてください：

| プラットフォーム | ファイル |
|----------|------|
| **macOS** (Apple Silicon) | `episteme-aarch64-apple-darwin.tar.gz` |
| **macOS** (Intel) | `episteme-x86_64-apple-darwin.tar.gz` |
| **Linux** (x86_64) | `episteme-x86_64-unknown-linux-gnu.tar.gz` |
| **Linux** (ARM64) | `episteme-aarch64-unknown-linux-gnu.tar.gz` |
| **Windows** (x86_64) | `episteme-x86_64-pc-windows-msvc.zip` |

```bash
# macOS / Linux
tar xzf episteme-*.tar.gz
sudo mv episteme /usr/local/bin/

# Windows — zipを展開し、episteme.exeをPATHに追加
```

その後、インストールを実行：
```bash
epis install claude    # または: cursor, codex, gemini
```

### 動作確認

```bash
epis --version
epis stats
epis explore "strategy pattern"    # ナレッジグラフの探索
```

---

## MCPツールとエージェント

> **MCPとは？** [Model Context Protocol](https://modelcontextprotocol.io)は、AIツールが外部サービスを呼び出すためのオープン標準です。EpistemeはナレッジグラフをMCPツールとして公開し、Claude Code、Cursor、その他の互換エディタが自動的に呼び出せるようにします。

### 9つのMCPツール

#### 正規知識（6つのツール）

| ツール | 用途 | 使用例 |
|------|---------|-------------|
| **`search_knowledge`** | 全エンティティのセマンティック検索 | "リトライロジックのパターンを検索" |
| **`get_entity`** | IDによるエンティティ詳細の取得 | "Strategyパターン (DP-023) の説明" |
| **`get_neighbors`** | 関連エンティティの探索 | "Long Methodを解決するリファクタリングは？" |
| **`find_path`** | 2つのエンティティ間の接続パスを検索 | "SRPとExtract Classの関係は？" |
| **`analyze_code`** | 正規表現/AST分析によるコードスメル検出 | "この決済検証コードをレビューして" |
| **`suggest_refactorings`** | ランク付けされたリファクタリングの提案 | 「このクラスで何をリファクタリングすべき？」 |

#### 暗黙知（3つのツール）

| ツール | 用途 | 使用例 |
|------|---------|-------------|
| **`add_insight`** | チームの決定、教訓を記録 | "ポーリングではなくイベント駆動を選択した理由" |
| **`search_insights`** | 過去のチーム知識を検索 | "認証ミドルウェアについて何を決定しましたか？" |
| **`confirm_links`** | 自動検出された正規エンティティへのリンクを検証 | TK-001がSMELL-03に関連することを確認 |

Epistemeは暗黙知を別のデータベース（`~/.episteme/user_knowledge.db`）に保存し、ランタイム時にコンポジットレイヤーを介して正規グラフとマージします。チームのインサイトはパターン、法則、スメルに自動的にリンクされ、経験がトラバース可能な知識に変換されます。

詳細な設計については、[暗黙知アーキテクチャ](../../docs/tacit-knowledge.md)を参照してください。

### 4つの専門エージェント（連携ネットワーク）

エージェントは連携して動作します — 各分析は最後に他のエージェントに引き継ぐ**次のステップ**オプションで終了します。

| エージェント | 使用タイミング | 主な機能 | 引き継ぎ先 |
|-------|-------------|----------------|--------------|
| **`code-reviewer`** | コードスメル、SOLID違反 | 因果関係分析（根本原因 → 下流の症状） | advisor, architecture-analyst, refactoring-expert |
| **`episteme-advisor`** | エンジニアリングの決定、トレードオフ | アクションプラン付きのマルチエンティティトレードオフチェーン | code-reviewer, architecture-analyst, researcher |
| **`episteme-researcher`** | ナレッジグラフの探索 | パターン、法則、スメル間の接続マップ | advisor, code-reviewer |
| **`architecture-analyst`** | 法則に基づくアーキテクチャ評価 | リスク加重評価付きのコンプライアンススコアリング | advisor, code-reviewer, researcher |

**ワークフロー例**: `code-reviewer`がGod Objectを検出 → 3つの下流スメルへの因果関係を追跡 → 「RF-018を適用」（→ refactoring-expert）または「根本原因を深掘り」（→ episteme-advisor）または「アーキテクチャチェック」（→ architecture-analyst）を提案。

[MCP統合ガイド（全文）](../../docs/mcp-integration-guide.md)

---

## CLIの使用方法

```bash
# コードのスメル分析
epis analyze my_code.py --language python --json
episteme infer my_code.py

# ナレッジグラフの探索
epis explore "strategy pattern"
epis graph path DP-005 RF-001   # 例: Factory Method → Extract Method

# RAGインデックスのビルド
epis build

# サーバーの起動
epis api              # REST API on :8000
episteme mcp --http       # MCPサーバー on :43175
episteme web --port 8080  # Web UI（インタラクティブなグラフエクスプローラー）

# 配布パッケージング
episteme dist --out-dir release/
```

---

## 機能

| | 機能 | 重要性 |
|--|------|--------|
| 🧠 | **22のGoFデザインパターン** | 実例を含む完全なカタログ |
| 🔧 | **66のリファクタリング手法** | コードサンプル付きのFowlerのカタログ |
| ⚖️ | **56のソフトウェア法則・原則** | SOLID、コンウェイの法則、CAP定理など |
| 👃 | **17のコードスメルタイプ** | Long Method、God Object、Feature Envyなど ¹ |
| 🔗 | **201のセマンティック関係** | 「解決する」「強制する」「違反する」「関連する」 |
| 🤖 | **9つのMCPツール + 4つのエージェント** | 高忠実度AIエージェント連携とエージェント間ハンドオフ |
| 🌍 | **10言語サポート** | Python（AST）、Java、TypeScript、Go、Rust、C++、C#、PHP、Ruby、Kotlin |
| 📊 | **決定論的分析** | ASTベースPython + 正規表現マルチ言語、常に同じ結果 |
| 🏷️ | **引用可能なナレッジ** | すべての発見が明示的エンティティID（`RF-001`、`LAW-021`）にリンク |
| 🌐 | **REST API（17エンドポイント）** | 認証、レート制限、ヘルスプローブ、Prometheusメトリクス |
| 📦 | **単一バイナリ** | ランタイム不要、クロスプラットフォーム（macOS、Linux、Windows） |
| 🔌 | **ローカル埋め込み** | fastembed（ONNX Runtime）、ゼロ設定セマンティック検索 |
| 🐳 | **Dockerサポート** | ヘルスチェック付きマルチステージビルド |

> ¹ Duplicate Code（SMELL-13）とShotgun Surgery（SMELL-09）は複数ファイルのコンテキストが必要なため、単一ファイルモードではスキップされます。

---

## ドキュメント

| ドキュメント | 説明 |
|----------|-------------|
| [クイックスタート](../../QUICKSTART.md) | ステップバイステップのセットアップ、初回実行、トラブルシューティング |
| [MCP統合ガイド](../../docs/mcp-integration-guide.md) | ツールリファレンス、エージェントの使用例、会話フロー |
| [APIリファレンス](../../docs/api.md) | RESTエンドポイント、認証、使用例 |
| [配布](../../docs/distribution.md) | リリースパッケージングとデプロイメント |
| [開発とコントリビューション](../../DEVELOPMENT.md) | アーキテクチャ、コントリビューション方法 |
| [変更履歴](../../CHANGELOG.md) | リリース履歴とバージョンノート |

---

## 設定

### 環境変数

```bash
# データの場所
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# APIサーバー
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEY=your-secret-key

# MCPサーバー
EPISTEME_MCP_HOST=127.0.0.1
EPISTEME_MCP_PORT=43175
```

---

## トラブルシューティング

**インストール後に`episteme`コマンドが見つからない場合**

| プラットフォーム | 解決方法 |
|----------|-----|
| **macOS / Linux** | `export PATH="$HOME/.cargo/bin:$PATH"` — 永続化するには`~/.bashrc`または`~/.zshrc`に追加 |
| **Windows** | `%USERPROFILE%\.cargo\bin`をシステムPATHに追加するか、新しいターミナルを開く |

**MCPツールがClaude Code / Cursorに表示されない場合**

`epis install`を実行した後、エディタを再起動してください。それでも表示されない場合、設定が書き込まれたか確認してください：
```bash
cat ~/.claude.json   # Claude Code
```

**ポートが既に使用されている場合**
```bash
episteme mcp --http --port 43176   # 別のポートを使用
```

**初回起動が遅い場合**

Epistemeは初回実行時にローカル埋め込みインデックスを構築します。これには30〜60秒かかり、一度だけのコストです。2回目以降の起動は瞬時です。

**`cargo install`中にコンパイルエラーが発生する場合**

Rust 1.95以上がインストールされていることを確認してください：
```bash
rustup update stable
rustup show   # アクティブなツールチェーンを確認
```

> さらなるヘルプ：[QUICKSTART.mdのトラブルシューティングセクション](../../QUICKSTART.md#troubleshooting) · [イシューを開く](https://github.com/epicsagas/Episteme/issues)

---

## ロードマップ

- [ ] **カスタムエンティティ** — チーム固有のパターン/スメルの追加
- [ ] **インタラクティブチュートリアル** — MCPツールのアプリ内ガイドツアー
- [ ] **多言語メタデータ** — エンティティのタイトルと概要の韓国語・日本語・中国語対応（README翻訳は完了）
- [ ] **MCPツール説明** — IDE専用プラグインに代わる強化されたツール説明
- [ ] **チームメトリクス** — 組織全体でのパターン使用状況の集計

---

## コントリビューション

コントリビューションを歓迎します！アーキテクチャの概要とコントリビューションガイドは[DEVELOPMENT.md](../../DEVELOPMENT.md)を参照してください。

```bash
# テストの実行
cargo test

# リント
cargo clippy -- -D warnings

# フォーマット
cargo fmt
```

質問がありますか？[ディスカッションを開く](https://github.com/epicsagas/Episteme/discussions)か[イシューを提出](https://github.com/epicsagas/Episteme/issues)してください。

---

## ライセンス

Apache 2.0 — 詳細は[LICENSE](../../LICENSE)を参照してください。
