# 暗黙知アーキテクチャ

Epistemeは2つの異なるナレッジ層を管理しています: **正準**（不変、キュレーション済み）と**暗黙知**（可変、ユーザー貢献）です。このドキュメントでは、2データベースアーキテクチャ、データフロー、インサイトライフサイクルについて説明します。

## 概要

| | 正準ナレッジ | 暗黙知（インサイト） |
|---|---|---|
| **ストレージ** | `~/.episteme/db/episteme.db` | `~/.episteme/user_knowledge.db` |
| **変更可能性** | 読み取り専用（`epis build`で再構築） | 読み書き可能（MCP経由でリアルタイム） |
| **IDプレフィックス** | `DP-NNN`、`RF-NNN`、`LAW-NNN`、`SMELL-NNN` | `TK-NNN` |
| **ソース** | `raw/`内のキュレーション済みMarkdownファイル | MCP `add_insight`ツール / CLI `epis insight` |
| **エンティティ** | 22パターン、66リファクタリング、56法則、23スメル | 無制限のユーザーインサイト |

これら2つのデータベースは物理的には分離されていますが、実行時に単一のトラバーサル可能なグラフにマージされます。

## 2データベース設計

```
┌─────────────────────────────────┐     ┌──────────────────────────────┐
│  正準DB (episteme.db)     │     │  ユーザーナレッジDB           │
│                                 │     │  (user_knowledge.db)         │
│  ┌───────────┐  ┌────────────┐  │     │  ┌────────────────────────┐  │
│  │  chunks   │  │ embeddings │  │     │  │  user_entities         │  │
│  │  (914)    │  │  (914)     │  │     │  │  (TK-xxxエントリ)      │  │
│  └───────────┘  └────────────┘  │     │  ├────────────────────────┤  │
│                                 │     │  │  user_relations        │  │
│  構築: epis build           │     │  ├────────────────────────┤  │
│  入力: raw/*.md       │     │  │  user_embeddings       │  │
│                                 │     │  ├────────────────────────┤  │
│  実行時は不変           │     │  │  user_entities_fts     │  │
│                                 │     │  │  (FTS5検索インデックス)   │  │
└──────────────┬──────────────────┘     │  ├────────────────────────┤  │
               │                        │  │  insight_seq           │  │
               │                        │  │  (原子IDカウンター)   │  │
               │                        │  └────────────────────────┘  │
               │                        │                              │
               │                        │  書き込み: MCP add_insight │
               │                        │  読み取り: search_insights    │
               │                        └──────────────┬───────────────┘
               │                                       │
               └───────────────┬───────────────────────┘
                               │
                    ┌──────────▼──────────┐
                    │   CompositeGraph    │
                    │   (インメモリマージ) │
                    │                     │
                    │  - 統一エンティティ   │
                    │    ルックアップ           │
                    │  - クロスレイヤーBFS  │
                    │  - クロスレイヤー      │
                    │    隣接クエリ │
                    │                     │
                    │  全MCPツールリクエスト  │
                    │  を処理              │
                    └─────────────────────┘
```

### なぜデータベースを分離するのか？

1. **保護** — ユーザー入力がキュレーション済みの正準ナレッジを破損することはありません。
2. **独立したライフサイクル** — 正準ナレッジはビルドパイプラインで更新され、暗黙知はリアルタイムで更新されます。
3. **ポータビリティ** — 正準層に触れることなく、`user_knowledge.db`をマシン間やチーム間で共有できます。

## CompositeGraph

`CompositeGraph`構造体（`src/domain/composite_graph.rs`内）は、起動時に両方の層を単一の`GraphRepository`インターフェースにマージします:

- `relations.json`から正準`KnowledgeGraph`を読み込み
- `UserGraphStore`経由で`user_knowledge.db`を開く
- 両層にわたる統一`get_entity()`、`get_neighbors()`、`find_path()`を提供
- ユーザー操作が正準グラフを変更することはありません

### グレースフルフォールバック

`user_knowledge.db`を開けない場合（ファイルの欠落、権限エラー）、システムは正準のみのモードにフォールバックします。6つの正準MCPツールは引き続き動作し、3つの暗黙知ツールはエラーを返します。

## ユーザーナレッジスキーマ

```sql
-- コアエンティティテーブル
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,                    -- 例: "TK-001"
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author TEXT NOT NULL DEFAULT 'user',
    confidence REAL NOT NULL DEFAULT 0.5,   -- 0.0 から 1.0
    evidence_count INTEGER NOT NULL DEFAULT 0,
    last_validated TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',        -- JSON配列
    relations TEXT NOT NULL DEFAULT '{}',   -- JSON: type -> [target_ids]
    created_at TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT '',
    link_provenance TEXT NOT NULL DEFAULT '{}'  -- JSON: entity_id -> metadata
);

-- 明示的なリレーションエッジ
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    to_id TEXT NOT NULL,
    UNIQUE(from_id, relation_type, to_id)
);

-- エンベディングベクトル（f32、リトルエンディアン）
CREATE TABLE user_embeddings (
    entity_id TEXT PRIMARY KEY,
    embedding BLOB NOT NULL
);

-- 全文検索インデックス
CREATE VIRTUAL TABLE user_entities_fts USING fts5(
    title, content, tags,
    content=user_entities, content_rowid=rowid
);

-- 原子IDシーケンス
CREATE TABLE insight_seq (key TEXT PRIMARY KEY, val INTEGER NOT NULL);
```

## MCPツール

### add_insight

フリーテキストから`TK-NNN`エンティティを作成します。システムは自動的に以下を実行:

1. **正準エンティティリンクの検出** — 二段階キーワードマッチング（ストップワード除去 + 複合スコアリング）で関連するパターン、法則、スメルを見つけます。
2. **重複チェック** — 既存のインサイトと比較します。
3. **`derives_from`リレーションの作成** — 高信頼度のリンク（スコア >= 0.5）の場合、自動的に正準エンティティにリンクします。
4. **相関関係の計算** — Jaccard類似度を使用して関連インサイトを見つけます。

パラメータ:
- `text`（必須） — フリーテキストのインサイト内容
- `project`（オプション） — プロジェクト名タグ
- `tags`（オプション） — カテゴリタグ
- `linked_entities`（オプション） — 明示的にリンクするエンティティID（例: `["DP-005", "SMELL-01"]`）

### search_insights

ユーザー貢献のインサイトのFTS5キーワード検索。内容とリレーション付きで一致する`TK-*`エンティティを返します。

パラメータ:
- `query`（必須） — 自然言語検索クエリ
- `limit`（オプション） — 最大結果数（デフォルト10、最大20）

### confirm_links

インサイトと正準エンティティ間の自動検出されたリンクを検証または拒否します。各確認は以下を実行:

- インサイトの信頼度スコアをブースト（確認リンクごとに+0.05、上限1.0）
- リンクの出所を記録（ソース、スコア、タイムスタンプ）
- インサイト間のマージ/置き換えリレーションをサポート

パラメータ:
- `insight_id`（必須） — `TK-NNN`のID
- `accepted`（必須） — 有効なリンクとして確認するエンティティID
- `rejected`（オプション） — 拒否するエンティティID
- `merged_with`（オプション） — マージ/置き換え先のインサイトID

## インサイトライフサイクル

```
1. add_insight("マイクロサービス分離時にドメイン境界を先に識別することに決定")
       │
       ▼
2. リンクの自動検出: CONWAY-001 (Conwayの法則)、DP-026 (Strangler Fig)
       │
       ▼
3. derives_from → LAW-017、DP-026 で TK-001 を作成
       │
       ▼
4. confirm_links(insight_id="TK-001", accepted=["LAW-017"])
       │
       ▼
5. 信頼度ブースト: 0.5 → 0.55
       │
       ▼
6. 後日: search_insights("マイクロサービス分離") → TK-001を返却
       │
       ▼
7. find_path("TK-001", "SMELL-03") → クロスレイヤーグラフをトラバース
```

## リレーションタイプ

| リレーション | 方向 | 説明 |
|----------|-----------|-------------|
| `derives_from` | TK → 正準 | インサイトが正準エンティティに基づいている |
| `applies_to` | TK → 正準 | インサイトがパターン/法則を特定のコンテキストに適用 |
| `supersedes` | TK → TK | より新しいインサイトが古いものを置き換える |
| `related_to` | TK → TK/正準 | 一般的なセマンティックなつながり |

## CLI使用法

```bash
# インサイトを追加
epis insight add "チームでGod Classリファクタリング時にExtract ClassよりFacade Patternが効果的だった"

# インサイトを検索
epis insight search "認証ミドルウェア"

# 全インサイトを一覧表示
epis insight list
```

## 主要ソースファイル

| ファイル | 役割 |
|------|------|
| `src/domain/composite_graph.rs` | 正準 + ユーザー層のランタイムマージ |
| `src/adapters/user_graph_store.rs` | SQLiteベースの`MutableGraphRepository` |
| `src/server/mcp_insight.rs` | 3つの暗黙知ツールのMCPハンドラー |
| `src/adapters/insight_utils.rs` | ID生成、タイムスタンプ、テキストユーティリティ |
| `src/domain/types.rs` | `UserEntity`、`LinkProvenance`、`EntityType::Insight` |
| `src/ports/graph.rs` | `MutableGraphRepository`トレイト（14メソッド） |
