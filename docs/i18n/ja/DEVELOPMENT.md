# Episteme 開発ガイド

**プロジェクト:** Episteme v0.1.0
**言語:** Rust（エディション2024）
**最終更新:** 2026-05-03

---

## 現在のステータス

| コンポーネント | ステータス | 詳細 |
|-----------|--------|---------|
| **ナレッジベース** | 完了 | 22パターン、66リファクタリング、56法則、23スメル、201リレーション |
| **コードスメル検出** | 本番稼働 | 16検出関数、10言語 |
| **REST API** | 本番稼働 | 17エンドポイント（axum）、レート制限、認証 |
| **MCPサーバー** | 本番稼働 | 6ツール、stdio + HTTPトランスポート |
| **RAGパイプライン** | 本番稼働 | SQLite + FTS5 + fastembed（ONNX） |
| **グラフ可視化** | 本番稼働 | D3-forceによるインタラクティブWeb UI |

---

## アーキテクチャ

ヘキサゴナル（ポート＆アダプター）アーキテクチャ：

```
src/
├── commands/          # CLIサブコマンドハンドラー（clap）
│   ├── analysis.rs    # analyze, infer
│   ├── build.rs       # build（RAGパイプライン）
│   ├── explore.rs     # explore（検索/REPL）
│   ├── graph.rs       # graphクエリ
│   ├── install.rs     # インストールウィザード（TUI）
│   ├── service.rs     # MCP HTTPデーモン管理
│   └── other.rs       # api, mcp, web, telemetry, hooks
├── adapters/          # インフラストラクチャ層
│   ├── regex_parsers.rs   # GenericParser（10言語、OnceLock正規表現キャッシュ）
│   ├── python_ast_parser.rs  # Python AST（rustpython-parser）
│   ├── search_engines.rs  # FTS5キーワード + コサイン類似度
│   ├── service.rs         # MCP HTTPデーモン
│   ├── sqlite_db.rs       # SQLite接続プール
│   ├── cache.rs           # Redisキャッシュ（オプション）
│   └── ...
├── domain/            # ビジネスロジック（外部依存なし）
│   ├── graph.rs       # KnowledgeGraph（BFS、サブグラフ、矛盾、Jaccard）
│   ├── detectors.rs   # TieredAccum付き16スメル検出器
│   ├── engine.rs      # RefactoringInferenceEngine + RefactoringRanker
│   ├── summarizer.rs  # 詳細レベルのレスポンス最適化
│   └── types.rs       # EntityType、RelationType、コア型
├── server/            # HTTP層（axum）
│   ├── api_routes.rs  # 17 RESTエンドポイント
│   ├── mcp_handler.rs # MCP薄いファサード
│   ├── mcp_search.rs  # 検索サービス
│   ├── mcp_graph.rs   # グラフサービス
│   └── mcp_analysis.rs # コード分析サービス
└── ports/             # トレイト（ヘキサゴナル境界）
    ├── parser.rs      # CodeParserトレイト
    ├── search.rs      # SearchEngineトレイト
    ├── graph.rs       # GraphStoreトレイト
    └── embeddings.rs  # EmbeddingProviderトレイト
```

---

## 技術スタック

| コンポーネント | 技術 | 目的 |
|-----------|-----------|---------|
| **言語** | Rust（エディション2024） | 安全性、パフォーマンス、単一バイナリ |
| **Webフレームワーク** | axum | REST API + MCP HTTPトランスポート |
| **データベース** | rusqlite（バンドルSQLite） | ナレッジグラフ + ベクトルストア |
| **検索** | FTS5 + コサイン類似度 | キーワード + セマンティックハイブリッド検索 |
| **エンベディング** | fastembed（ONNX Runtime） | ローカル、ゼロ設定のエンベディング生成 |
| **CLI** | clap（derive） | 15サブコマンド |
| **Python AST** | rustpython-parser | ASTベースのPythonスメル検出 |
| **その他言語** | regex（OnceLockキャッシュ） | GenericParserフレームワーク |

---

## コードスメル検出器（16）

| ID | スメル | 検出方法 |
|----|-------|-----------|
| SMELL-01 | Long Method | LOC閾値 |
| SMELL-02 | Long Parameter List | パラメータ数 |
| SMELL-03 | Primitive Obsession | プリミティブパラメータ比率 |
| SMELL-04 | Large Class | メソッド + フィールド数 |
| SMELL-05 | Data Clumps | 繰り返しパラメータグループ（スタブ） |
| SMELL-06 | Switch Statements | switch/match数 |
| SMELL-07 | Data Class | メソッド対フィールド比 |
| SMELL-08 | Temporary Field | 条件付きフィールド使用（スタブ） |
| SMELL-09 | Shotgun Surgery | 変更結合度（スタブ） |
| SMELL-10 | Divergent Change | メソッド凝集度メトリクス |
| SMELL-11 | Lazy Class | 低LOC + メソッド数 |
| SMELL-12 | Speculative Generality | 抽象のみで具象なし |
| SMELL-13 | Duplicate Code | ハッシュベース類似度（部分） |
| SMELL-14 | Middle Man | 委譲比率 |
| SMELL-15 | Parallel Inheritance Hierarchies | 階層ミラーリング（スタブ） |
| SMELL-16 | Comments | コメント対コード比（スタブ） |
| SMELL-17 | Dead Code | 到達不能/未使用検出（スタブ） |
| SMELL-18 | Feature Envy | 外部呼び出し比率 |
| SMELL-19 | Inappropriate Intimacy | クラス間プライベートアクセス（スタブ） |
| SMELL-20 | Message Chains | 呼び出しチェーン深度 |
| SMELL-21 | God Object | 複合: LOC + メソッド + 結合度 |
| SMELL-22 | Refused Bequest | オーバーライド対空比率（スタブ） |
| SMELL-23 | Alternative Classes with Different Interfaces | インターフェース分岐（スタブ） |

---

## 開発環境のセットアップ

```bash
# クローンとビルド（Rust 1.95以上が必要）
git clone https://github.com/epicsagas/Episteme.git
cd Episteme
cargo build

# テスト実行
cargo test

# リント
cargo clippy -- -D warnings

# ローカルにインストール（データをシードし、DBを自動構築）
cargo install --path .
epis install --local
```

---

## APIエンドポイント（17）

| メソッド | パス | 説明 |
|--------|------|-------------|
| GET | `/` | サービス情報 |
| GET | `/health` | ヘルスチェック |
| GET | `/live` | ライブネスプローブ |
| GET | `/ready` | レディネスプローブ |
| GET | `/stats` | グラフ統計 |
| POST | `/analyze` | コードスメル検出 |
| POST | `/refactor` | リファクタリング提案 |
| GET | `/search` | ナレッジ検索 |
| POST | `/search` | ナレッジ検索（POST） |
| GET | `/graph/{id}` | エンティティ取得 |
| GET | `/graph/{id}/neighbors` | 隣接エンティティ取得 |
| POST | `/graph/neighbors` | 隣接エンティティ取得（POST） |
| POST | `/graph/subgraph` | サブグラフ抽出 |
| GET | `/graph/path` | 最短経路 |
| GET | `/graph/contradictions` | 矛盾の発見 |
| POST | `/graph/infer-transitive` | 推移的リレーションの推論 |
| GET | `/metrics` | Prometheusメトリクス |

---

## 今後のロードマップ

- **IDEプラグイン** — VSCode、IntelliJネイティブ統合
- **カスタムエンティティ** — チーム固有のパターン/スメルの追加
- **チームメトリクス** — 組織全体でのパターン利用状況の集計
- **多言語ドキュメント** — 韓国語、日本語、中国語でのナレッジベース
- **インタラクティブチュートリアル** — MCPツールのアプリ内ガイド付きツアー

---

*最終更新: 2026-05-03*
