# Alcoveエコシステム — アーキテクチャと機能分析

> Epistemeの暗黙知層（TK-*）とAlcoveドキュメントエコシステムの詳細な比較。ストレージモデル、検索機能、ライフサイクル管理、ユースケースガイダンスを網羅します。

---

## 1. アーキテクチャ概要

### Episteme 暗黙知（TK-*）

| 側面 | 詳細 |
|--------|--------|
| **ストレージ** | SQLite単一ファイル（`~/.episteme/user_knowledge.db`） |
| **スキーマ** | 5テーブル: `user_entities`、`user_relations`、`user_embeddings`、`user_entities_fts`（FTS5仮想テーブル）、`insight_seq` |
| **単位** | 1インサイト = 1つの`UserEntity`行（TK-xxx ID） |
| **グラフ** | 実行時に`CompositeGraph`で正準グラフとマージ — クロスレイヤーの経路探索を可能に（TK-001 → DP-005 → SMELL-01） |
| **並行性** | MCP + CLIの同時アクセス向けに`Mutex<Connection>` + WALモード |

### Alcoveドキュメントシステム

| 側面 | 詳細 |
|--------|--------|
| **ストレージ** | ファイルシステム上のMarkdownファイル + Tantivy BM25インデックス + sqlite-vecエンベディング |
| **構造** | 3層分類: コア（7）、補足（19）、パブリック（15）ファイル/プロジェクト |
| **単位** | 1つの構造化Markdownファイル（PRD、ARCHITECTURE、DECISIONSなど） |
| **グラフ** | wikilink + ファイルパスベースの疎な接続 |
| **並行性** | ドキュメントルートごとのファイルベースのロック（`.index_lock`）、ボルトごとのインデックス分離 |
| **ボルト** | Obsidian PARAフォルダへの3つのシンボリックリンク: areas（8ドキュメント）、resources（71）、zettelkasten（17） |

---

## 2. ストレージモデル比較

### Episteme TK-* スキーマ

```sql
-- コアテーブル
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,           -- TK-001, TK-002, ...
    title TEXT,                    -- 自動: 最初の行、最大80文字
    content TEXT,                  -- フリーテキスト（最大長なし）
    author TEXT DEFAULT 'user',
    confidence REAL DEFAULT 0.5,   -- 確認リンクごとに+0.05、上限1.0
    evidence_count INTEGER DEFAULT 0,
    last_validated TEXT DEFAULT '',
    tags TEXT DEFAULT '[]',        -- JSON配列
    relations TEXT DEFAULT '{}',   -- JSON HashMap<relation_type, Vec<entity_id>>
    link_provenance TEXT DEFAULT '{}',
    created_at TEXT,
    updated_at TEXT
);

-- 正規化されたリレーション（derives_from、applies_to、supersedes）
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT,
    relation_type TEXT,
    to_id TEXT,
    UNIQUE(from_id, relation_type, to_id)
);

-- FTS5全文検索
CREATE VIRTUAL TABLE user_entities_fts USING fts5(title, content, tags, content=user_entities);
```

### Alcoveファイル構造

```
~/.alcove/
  config.toml                    # グローバル設定（docs_root、core/team/publicファイルリスト、エンベディングモデル）
  docs -> symlink                # → Obsidian/SecondBrain/99-Archives/projects
  vaults/
    areas -> symlink             # → Obsidian/02-Areas (8ドキュメント)
    resources -> symlink         # → Obsidian/03-Resources (71ドキュメント)
    zettelkasten -> symlink      # → Obsidian/10-Zettelkasten (17ドキュメント)
  models/                        # キャッシュされたONNXエンベディングモデル
  logs/

<docs_root>/<project>/
  .alcove/
    index/                       # Tantivy BM25インデックスファイル
    index_meta.json              # ファイルフィンガープリント（mtime + size）
    vectors.db                   # sqlite-vecエンベディング
  PRD.md                         # プロダクト要件
  ARCHITECTURE.md                # システム設計
  PROGRESS.md                    # マイルストーンとステータス
  DECISIONS.md                   # アーキテクチャ決定記録
  CONVENTIONS.md                 # コーディング標準
  SECRETS_MAP.md                 # 環境変数とシークレット
  DEBT.md                        # テクニカルデッドレジスター
```

---

## 3. ナレッジの性質

| 側面 | Episteme TK-* | Alcove |
|-----------|---------------|--------|
| **タイプ** | 瞬時のインサイト、教訓、チームの決定 | 構造化されたプロジェクトドキュメント（要件、アーキテクチャ、決定） |
| **変更可能性** | 可変（SQLite CRUD） | 可変（ファイル編集 + インデックス再構築） |
| **ソース** | ユーザー貢献のフリーテキスト | ユーザー執筆 + テンプレートからのエージェント生成 |
| **権威性** | 個人/チームの観察 | チームの決定 / 組織ポリシー |
| **粒度** | 原子的（エントリごとに1インサイト） | セクション化（DECISIONS.mdに複数のADR） |
| **リンク** | 正準エンティティへの自動検出（キーワードスコアリング） | 手動のwikilink + Markdownリンク |
| **バージョン管理** | なし（SQLiteのみ） | Gitベース（ファイル = 信頼の情報源） |

### インサイトライフサイクル（Episteme TK-*）

```
add_insight(text, tags?, project?, linked_entities?)
  │
  ├── TK-xxx IDの生成（原子シーケンス）
  ├── detect_canonical_links() — キーワードマッチング → 上位5正準エンティティ
  │     score >= 0.5 → 自動リンク (derives_from)
  │     score < 0.5 → 提案リンク
  ├── FTS5重複検出 → DuplicateCandidate[]
  ├── SQLite + インメモリキャッシュに永続化
  └── 返却: { id, auto_links, suggested_links, duplicates, confidence }

confirm_links(id, accepted[], rejected[])
  │
  ├── derives_from/applies_toリレーションを追加
  ├── link_provenanceソースを"manual"に更新
  ├── 信頼度をブースト（+0.05/リンク、上限1.0）
  └── 更新を永続化

search_insights(query, limit?)
  │
  └── FTS5 MATCHクエリ → ランク付けされた結果
```

### ドキュメントライフサイクル（Alcove）

```
init_project(project_name, project_path?)
  │
  ├── テンプレートから7つのコアドキュメントを作成（PRD、ARCHITECTURE、...）
  ├── オプションでパブリックドキュメントを作成（README、CHANGELOG、...）
  └── 検索インデックスを再構築

validate_docs()
  │
  ├── 必須ファイルの存在確認
  ├── テンプレートプレースホルダーの確認（TODO、FIXME）
  ├── 必須セクション見出しの確認
  ├── 最小リスト項目数の確認
  └── ファイルごとに pass/warn/fail を返却

lint_project()
  │
  ├── 破損した[[wikilinks]]とMarkdownリンクを検出
  ├── 孤立ファイル（どのドキュメントからもリンクされていない）を見つける
  ├── 古いマーカー（WIP、TODO、FIXME、DRAFT、DEPRECATED）を見つける
  └── 古い年号参照（2年以上前）を見つける

audit_project()
  │
  ├── プライベートdoc-repoで欠落している必須ドキュメントをスキャン
  ├── パブリックプロジェクトrepoで公開されている内部ドキュメントをスキャン
  ├── ファイルを層に分類
  └── suggested_actions[]を返却
```

---

## 4. 検索機能

| 機能 | Episteme TK-* | Alcove |
|------------|---------------|--------|
| **エンジン** | FTS5（キーワードマッチ） | Tantivy BM25 + sqlite-vecコサイン類似度 |
| **フュージョン** | なし | RRF（Reciprocal Rank Fusion、k=60） |
| **CJK対応** | 特別なサポートなし | NgramTokenizer（min=2、max=3） |
| **チャンキング** | 該当なし（1行 = 1インサイト） | 200–500文字チャンク |
| **インクリメンタル** | 該当なし（単一テーブル） | mtime + sizeフィンガープリント比較 |
| **ベクトル検索** | スキーマは存在（`user_embeddings`）するが**未接続** | 完全に稼働（MultilingualE5Small、384d） |
| **スコープ** | 単一データベース | プロジェクト単位またはグローバル（クロスプロジェクト） |
| **フォールバック** | なし | インデックスなしの場合のgrep部分文字列マッチ |

---

## 5. 機能の完全性

| 機能 | Episteme TK-* | Alcove |
|---------|---------------|--------|
| 作成 | `add_insight` | `init_project`、ファイル編集 |
| 読み取り | `search_insights`（検索のみ、IDでの取得なし） | `get_doc_file`、`search_project_docs` |
| 更新 | MCP経由で未公開 | 直接ファイル編集 + `rebuild_index` |
| 削除 | MCP経由で未公開 | ファイル削除 + `rebuild_index` |
| 検証 | なし | `validate_docs`、`lint_project` |
| 監査 | なし | `audit_project`（パブリック/プライベート分離） |
| バックアップ | なし | `backup_vault`（Gitコミットスナップショット） |
| インポート | なし | `promote_document`（Obsidian → doc-repo） |
| ポリシー | なし | 強制レベル付き`policy.toml` |
| テンプレート | なし | 7コア + 19補足 + 15パブリック |

---

## 6. Alcoveボルトシステム

Obsidian PARA構造にシンボリックリンクされた3つのボルト:

| ボルト | リンク先 | ドキュメント数 | 目的 |
|-------|--------|------|---------|
| `areas` | `02-Areas` | 8 | ドメイン領域: MCPエージェント、DevOps、Rust、LLM/RAG、オープンソース |
| `resources` | `03-Resources` | 71 | リファレンス: AWS、ソフトウェアエンジニアリングの法則、技術ドキュメント |
| `zettelkasten` | `10-Zettelkasten` | 17 | 原子的ノート: AIアーキテクチャ、BM25、ナレッジグラフ、Rustパターン |

各ボルトは独立した以下を持ちます:
- BM25インデックス（Tantivy）
- ベクトルデータベース（sqlite-vec）
- ファイルフィンガープリント追跡（`index_meta.json`）
- キャッシュ分離（独立した`OnceLock<Mutex<HashMap>>`）

---

## 7. Alcove設定システム

### グローバル: `~/.alcove/config.toml`

```toml
docs_root = "/path/to/Obsidian/SecondBrain/99-Archives/projects"

[core]
files = ["PRD.md", "ARCHITECTURE.md", "PROGRESS.md", "DECISIONS.md",
         "CONVENTIONS.md", "SECRETS_MAP.md", "DEBT.md"]

[team]
files = ["ENV_SETUP.md", "ONBOARDING.md", "DATA_MODEL.md", "SCHEMA.md",
         "DEPLOYMENT.md", "RUNBOOK.md", "PLAYBOOK.md", "MONITORING.md", ...]  # 19ファイル

[public]
files = ["README.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md", ...]  # 15ファイル

[embedding]
model = "MultilingualE5Small"
auto_download = true
enabled = true
```

### プロジェクトごと: `alcove.toml`

グローバルのデフォルトをオーバーライド: `diagram_format`、`core_files`、`team_files`、`public_files`。

### ポリシー: `policy.toml`

定義内容:
- `enforce`レベル: `strict` | `warn` | `off`
- 必須ドキュメントとセクション見出しおよび最小項目数
- 命名規則（`UPPER_SNAKE`、`lower_snake`、`kebab`、`free`）
- 優先順位: プロジェクト > チーム > 組み込みデフォルト

---

## 8. ユースケース判定マトリクス

| 状況 | 推奨ツール | 理由 |
|-----------|-----------------|-----------|
| "本番インシデントからの教訓を記録したい" | **Episteme TK-*** | 関連するスメル/法則に自動リンク、将来のクロスリファレンスに活用 |
| "新しいプロジェクトのドキュメントを始めたい" | **Alcove** `init_project` | 7つのコアテンプレートが自動生成 |
| "古いドキュメントがないか確認したい" | **Alcove** `lint_project` | WIP/TODO/DEPRECATED/古い日付を自動検出 |
| "認証ミドルウェアについてチームがどう決めたか知りたい" | **Alcove** `search_project_docs` | BM25 + ベクトルで構造化されたDECISIONS.mdを検索 |
| "モジュールのコードスメルを検出したい" | **Episteme** `analyze_code` | パターン/正規表現ベースのスメル検出 |
| "PRDに必須セクションが揃っているか確認したい" | **Alcove** `validate_docs` | ポリシーベースのセクション・項目数検証 |
| "インサイトをStrategyパターンにリンクしたい" | **Episteme** `confirm_links` | 正準エンティティへの`derives_from`エッジを作成 |
| "Obsidianノートをエージェントアクセス用にインポートしたい" | **Alcove** `promote_document` | 自動プロジェクト検出でdoc-repoにインポート |
| "SRPとExtract Classの関係を見つけたい" | **Episteme** `find_path` | エンティティタイプをまたぐマルチホップグラフトラバーサル |
| "プロジェクトドキュメントの状態をバックアップしたい" | **Alcove** `backup_vault` | タイムスタンプ付きGitコミットスナップショット |
| "パブリックrepoに内部ドキュメントが公開されていないか監査したい" | **Alcove** `audit_project` | プライベートとパブリックの両方の場所をスキャン |
| "コードのランク付けされたリファクタリング提案を取得したい" | **Episteme** `suggest_refactorings` | 複合スコア: 重要度 × 労力 × 原則適合性 |

---

## 9. 補完的な役割

```
Episteme TK-*                     Alcove
"ここに適用される普遍的な          "これについてチームは
 原則は？"                          どう決めた？"

 瞬時のインサイト ←────────────→ 構造化された決定記録
 キーワード自動リンク               テンプレートベースのスキャフォールディング
 クロスレイヤーグラフトラバーサル   クロスプロジェクトドキュメント検索
 コード分析 → スメル検出            ドキュメント分析 → 古さの検出
```

**両方が有効な場合**: Epistemeは普遍的な「なぜ」（法則、パターン）を提供し、Alcoveはプロジェクト固有の「チームがどう決めたか」（ADR、規約）を提供します。エージェントは両方のソースを引用し、チームルールが一般的なガイダンスと矛盾する場合はAlcoveが優先されます。

---

## 10. スケールとパフォーマンス

| 指標 | Episteme TK-* | Alcove |
|--------|---------------|--------|
| **設計容量** | 数百のインサイト | ~10,000ファイル |
| **検索レイテンシ** | FTS5即時（インメモリ） | BM25概要取得で500ms未満 |
| **トークン効率** | 結果ごとに1インサイト | 上位5チャンクで約1.5kトークン（grepでは約8k） |
| **インデックス再構築** | 不要（FTS5トリガー） | インクリメンタル: 変更されたファイルのみ |
| **モデルサイズ** | 該当なし（未接続） | 15MB（ArcticEmbedXS）〜 2.3GB（BGE-M3） |

---

*関連: [Alcove統合ガイド](./alcove-integration.md)に使用パターンとワークフロー例があります。*
