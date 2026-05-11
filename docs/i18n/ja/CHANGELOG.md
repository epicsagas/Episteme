# 変更履歴

Epistemeに関する注目すべき変更はすべてこのファイルに記録されます。

フォーマットは[Keep a Changelog](https://keepachangelog.com/en/1.1.0/)に基づいており、
本プロジェクトは[セマンティックバージョニング](https://semver.org/spec/v2.0.0.html)に準拠しています。

## [Unreleased]

### Changed

- CLI: `explore`を`search`に名称変更（旧名称は非推奨エイリアスとして動作）
- CLI: `mcp`と`api`がサービスライフサイクル全体を管理（`start`、`stop`、`restart`、`status`、`enable [--now]`、`disable [--now]`）
- CLI: `service`トップレベルコマンドは非推奨 — `mcp start/stop/restart/status/enable/disable`を使用してください
- CLI: `mcp --http`は非推奨 — HTTPデーモンモードには`mcp start`を使用してください
- CLI: `launchd-install/uninstall/status`は非推奨 — `mcp enable/disable/status`を使用してください
- `enable/disable`がクロスプラットフォーム対応: macOS（launchd）およびLinux（systemdユーザーユニット）

### Added

- `api start/stop/restart/status/enable/disable` — REST APIデーモンのライフサイクル管理
- `mcp enable`向けのLinux systemdユーザーユニット生成

- **Claude Code向けMCP HTTPトランスポート** — トランスポートセレクターTUI、HTTPをデフォルトに、launchd自動有効化
- **エージェントプロンプトの自動インストール** — `epis install`がEpistemeエージェントプロンプトを`~/.claude/agents/`にコピー
- **エンティティ説明** — Markdownソースファイルから説明フィールドを自動抽出、Webビューアーの詳細パネルに表示
- **ベンチマーク可視化SPA** — トレンド分析、クエリ内訳ダッシュボード
- **Webビューアーの再設計** — Sankeyダイアグラムレイアウト、サイドバーツリー、詳細パネル、サブグラフ可読性の改善
- **MCP設定アップサート** — `epis install`の再実行時に設定が異なる場合にトランスポートを更新（stdio ↔ HTTP）
- **MCP YAML設定** — `config.yaml`の`mcp.host` / `mcp.port`（yaml → envフォールバック）
- **モニタリング** — 環境変数経由のネイティブおよびリモートPrometheusスクレイプターゲット対応
- **CI強化** — cargo audit、gitleaks、SBOM生成、アクションSHAのピン留め
- **リリースパイプライン** — Windowsターゲット、crates.ioパブリッシュ、Homebrew tap
- `examples/`の**Godモジュールアーキテクチャ診断例**

### Changed

- **インストールウィザード** — 全ステップ（トランスポート、Redis、テレメトリ）をフルスクリーンTUIに移行
- **インストールフロー** — シード後にRAGインデックスを自動構築、DBが既存の場合はスキップ
- **ナレッジグラフ** — エンティティ間のセマンティックリレーションを拡充
- **ライセンス** — MIT → Apache-2.0

### Fixed

- テレメトリの同期`main()`におけるTokioランタイムパニック
- 検索品質 — NDCG計測バグを解決、hit@1精度が100%に向上
- 検索リコール — クロスタイプブースティング、スパースエンティティ処理、インテント同義語
- fastembedモデルキャッシュを`~/.episteme/models`にピン留め
- launchdブートストラップUID置換とポート使用中ハンドリング
- CORSオリジンが`EPISTEME_CORS_ORIGINS`で設定可能に

## [0.1.0] - 2026-05-03

### Added

- **完全なRustリライト** — Pythonコードベースを慣用的なRustで完全置換
- **ヘキサゴナルアーキテクチャ** — `ports/`（トレイト）、`domain/`（ビジネスロジック）、`adapters/`（インフラ）、`server/`（HTTP）
- **GenericParserフレームワーク** — 8つのブレースベースパーサーを`ParserConfig`を持つ`GenericParser`に統合；正規表現パターンは`OnceLock`と`Box::leak`でキャッシュ
- **Python ASTパース** — 正確なPythonスメル検出のための`rustpython-parser`（Long Method、Large Class、God Object）
- **TieredAccum + build_detection()** — `detectors.rs`の14個の同一スメル検出構築を重複排除（1,253 → 591行）
- **MCPモジュール分割** — `EpistemeMCP`（675行）を`mcp_search`、`mcp_graph`、`mcp_analysis`サービスに分割
- **CLIコマンド分割** — `main.rs`（1,741行）をclap定義用の`cli.rs`を持つ`commands/`モジュールに分割
- **APIハンドラー重複排除** — 重複する`search`/`search_post`を共有`do_search()`に統合
- **16のスメル検出関数** — 14個から増加、GoFスメルカテゴリをすべてカバー
- **17のREST APIエンドポイント** — ヘルスプローブ、Prometheusメトリクス、CORS、レート制限
- **レートリミッターTTL退去** — MAX_BUCKETS=10,000、1時間TTLで無制限メモリ増大を防止
- **ReDoS緩和** — 三項演算子正規表現を`[^:]+`から`[^:\n]{1,50}`に制限
- **ローカルエンベディング** — ゼロ設定セマンティック検索のためのfastembed（ONNX Runtime）
- **インタラクティブインストールウィザード** — crossterm、Vimキーバインディング、代替スクリーン付きTUI
- **配布パッケージング** — 自動DBブートストラップ付きリリースアーカイブ作成のための`episteme dist`コマンド
- **クロスプラットフォームCI** — linux/macOS（x86_64 + aarch64）向けGitHub Actionsリリースワークフロー
- **マルチステージDockerfile** — Rustビルダー + 軽量Debianランタイム

### Changed

- **言語**: Python 3.11+ → Rust（エディション2024）
- **Webフレームワーク**: FastAPI → axum
- **データベース**: Python sqlite3 → rusqlite（バンドル付き）
- **エンベディング**: sentence-transformers/PyTorch → fastembed/ONNX Runtime
- **CLI**: argparse → clap（derive）
- **すべての正規表現パターンをキャッシュ** — グローバル`REGEX_CACHE`によるホットパスでの再コンパイルゼロ

### Removed

- Pythonランタイム依存
- ChromaDB依存
- tree-sitter依存
- PyPIパブリッシュワークフロー
- `episteme-hook`スタンドアロンバイナリ（Python専用のPyPIエントリポイントでした） — `episteme hooks ground|sniff|audit`を使用してください

## [0.0.5] - 2026-04-30

### Added

- D3-forceによるグラフ可視化Web UI（`episteme web`）
- リリースアーカイブに事前構築済みベクトルDBを同梱
- 開発ワークフロー向け`epis install --local`フラグ
- 全161エンティティをカバーする650以上のセマンティックリレーション
- リリース時のCI自動ベクトルDB生成

## [0.0.4] - 2026-04-29

### Added

- 6ツール搭載MCPサーバー
- 4つの専用エージェント
- `epis install`コマンド
- `epis service`デーモン管理
- ハイブリッド検索（FTS5 + ベクトル）
- Redisキャッシュ、GPUアクセラレーション
- 10言語コードスメル検出
- Prometheus + Grafanaモニタリング
