# Episteme REST API ドキュメント

**バージョン:** 0.1.0
**ベースURL:** `http://localhost:8000`

---

## クイックスタート

```bash
# サーバー起動
epis api

# またはカスタムホスト/ポート指定
epis api --host 0.0.0.0 --port 8000

# Docker
docker-compose up -d
```

---

## 認証

`/`、`/health`、`/live`、`/ready`を除くすべてのエンドポイントでAPIキー認証が必要です。

### APIキー認証

**ヘッダー:** `X-API-Key: <your-api-key>`

**モード:**

1. **本番モード** - `EPISTEME_API_KEYS`環境変数を設定
   - カンマ区切りの有効なAPIキーリスト
   - 保護されたエンドポイントすべてで有効なキーが必要
   - 無効または欠落の場合は401 Unauthorizedを返却

2. **開発モード** - `EPISTEME_API_KEYS`を空または未設定にする
   - 認証不要

### APIキーの生成

```bash
openssl rand -base64 32
```

### リクエスト例

```bash
# 認証あり（本番）
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"code": "def long_method(): pass", "min_confidence": 0.5}'

# 認証なし（開発モード）
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"code": "def long_method(): pass"}'
```

---

## レート制限

すべてのエンドポイントはIPアドレスごとにTTLベースのバケット退去でレート制限されています。

| エンドポイント | レート制限 | 理由 |
|----------|------------|--------|
| `/analyze` | 20回/分 | CPU集約的 |
| `/refactor` | 20回/分 | CPU集約的 |
| `/search` | 50回/分 | エンベディング計算 |
| `/stats`、`/graph/*` | 100回/分 | 標準 |
| `/`、`/health` | 無制限 | パブリック |

制限超過時は429を返し、`Retry-After`ヘッダーを含みます。

---

## エンドポイント

### ヘルスチェック ＆ 情報

#### `GET /`

サービス情報。

**レスポンス:**
```json
{
  "name": "episteme",
  "version": "0.1.0",
  "description": "Software engineering knowledge graph",
  "endpoints": ["analyze", "search", "graph", "refactor", "stats"]
}
```

#### `GET /health`

コンポーネントステータス付きヘルスチェック。

**レスポンス:**
```json
{
  "status": "healthy",
  "components": {
    "knowledge_graph": "ok",
    "rag_database": "ok",
    "embedding_provider": "local"
  }
}
```

#### `GET /live`

ライブネスプローブ: `{"status": "alive"}`

#### `GET /ready`

レディネスプローブ: `{"status": "ready"}`（準備できていない場合は503）

#### `GET /stats`

グラフ統計。

**レスポンス:**
```json
{
  "total_entities": 161,
  "total_edges": 201,
  "by_type": {
    "refactoring": 66,
    "law": 56,
    "pattern": 22,
    "smell": 17
  }
}
```

---

### コード分析

#### 対応コードスメル（16検出器）

| ID | 名前 | 対応言語 |
|---|---|---|
| SMELL-01 | Long Method | 全言語 |
| SMELL-02 | Long Parameter List | 全言語 |
| SMELL-03 | Primitive Obsession | Python |
| SMELL-04 | Large Class | 全言語 |
| SMELL-05 | Data Clumps | 全言語（スタブ） |
| SMELL-06 | Switch Statements | 全言語 |
| SMELL-07 | Data Class | 全言語 |
| SMELL-09 | Shotgun Surgery | 全言語（スタブ） |
| SMELL-10 | Divergent Change | 全言語 |
| SMELL-11 | Lazy Class | 全言語 |
| SMELL-12 | Speculative Generality | 全言語 |
| SMELL-13 | Duplicate Code | 全言語（部分） |
| SMELL-14 | Middle Man | 全言語 |
| SMELL-18 | Feature Envy | 全言語 |
| SMELL-20 | Message Chains | 全言語 |
| SMELL-21 | God Object | 全言語 |

#### `POST /analyze`

コードスメルを検出します。

**リクエスト:**
```json
{
  "code": "def long_method():\n    ...",
  "language": "python",
  "min_confidence": 0.5
}
```

**レスポンス:**
```json
{
  "count": 2,
  "smells": [
    {
      "smell_id": "SMELL-01",
      "smell_name": "Long Method",
      "confidence": 0.90,
      "location": "temp.py:1",
      "function_name": "long_method",
      "metrics": {
        "loc": 94,
        "cyclomatic_complexity": 27,
        "nesting_depth": 5,
        "parameter_count": 9
      },
      "reasons": ["LOC=94 exceeds 30", "CC=27 exceeds 10"]
    }
  ]
}
```

#### `POST /refactor`

検出されたスメルに対するランク付けされたリファクタリング提案を取得します。

**リクエスト:**
```json
{
  "code": "def long_method():\n    ...",
  "top_k": 3,
  "min_confidence": 0.5
}
```

**レスポンス:**
```json
{
  "count": 1,
  "analyses": [
    {
      "smell": { "smell_id": "SMELL-01", "smell_name": "Long Method" },
      "suggestions": [
        {
          "refactoring_id": "RF-001",
          "title": "Extract Method",
          "priority_score": 0.79,
          "effort": "medium",
          "principles_enforced": ["LAW-040", "LAW-042-S"]
        }
      ]
    }
  ]
}
```

---

### 検索

#### `GET /search`

クエリパラメータで検索: `/search?q=strategy+pattern&top_k=5`

#### `POST /search`

ナレッジベース全体のセマンティック検索。

**リクエスト:**
```json
{
  "query": "How to fix Long Method?",
  "top_k": 5,
  "entity_type": "refactoring"
}
```

**レスポンス:**
```json
{
  "count": 3,
  "results": [
    {
      "entity_id": "RF-001",
      "title": "Extract Method",
      "category": "refactoring",
      "similarity": 0.85,
      "content": "Extract Method is a refactoring technique..."
    }
  ]
}
```

---

### ナレッジグラフ

#### `GET /graph/{id}`

IDでエンティティの詳細を取得します。

**例:** `GET /graph/DP-005`

#### `GET /graph/{id}/neighbors`

エンティティの隣接エンティティを取得: `/graph/SMELL-01/neighbors?relation_type=solved_by`

#### `POST /graph/neighbors`

隣接エンティティを取得（POST）。

**リクエスト:**
```json
{
  "entity_id": "SMELL-01",
  "relation_type": "solved_by"
}
```

#### `GET /graph/path`

最短経路: `/graph/path?from_id=SMELL-01&to_id=LAW-042-S&max_depth=5`

#### `POST /graph/subgraph`

サブグラフを抽出します。

**リクエスト:**
```json
{
  "entity_id": "DP-005",
  "depth": 2
}
```

#### `GET /graph/contradictions`

矛盾するリレーションを持つエンティティを検索します。

#### `POST /graph/infer-transitive`

推移的な強制リレーションシップを推論します。

---

### モニタリング

#### `GET /metrics`

Prometheus形式のメトリクス（以下を含む）:
- `http_requests_total` — メソッド、エンドポイント、ステータス別
- `episteme_smells_detected_total` — smell_id別
- `episteme_searches_total` — entity_type別
- `episteme_analysis_duration_seconds` — ヒストグラム

---

## パフォーマンス

| エンドポイント | 平均レイテンシ | 備考 |
|----------|-------------|-------|
| `/analyze` | ~5ms | 正規表現 + ASTパース（OnceLockキャッシュ） |
| `/refactor` | ~10ms | グラフトラバーサル含む |
| `/search` | ~20ms | FTS5 + コサイン類似度 |
| `/graph/neighbors` | ~1ms | インメモリグラフ |
| `/graph/path` | ~5ms | 深さ5までのBFS |

---

## エラーハンドリング

| ステータス | 意味 |
|--------|---------|
| 200 | 成功 |
| 400 | 不正なリクエスト |
| 401 | APIキーが無効または欠落 |
| 404 | エンティティが見つかりません |
| 429 | レート制限超過 |
| 500 | 内部エラー |

---

## 環境変数

```bash
# サーバー
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEYS=key1,key2

# データ
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# ロギング
RUST_LOG=info
```

---

## ライセンス

APACHE-2.0ライセンス
