# MCP統合ガイド

> EpistemeのナレッジグラフをClaude Code、Cursor、その他のMCP対応AIツールに統合する

## Rust MCP HTTPモード（現在）
スタンドアロンHTTPトランスポートを直接使用:

```bash
# HTTP経由でMCPを起動
episteme mcp --http --host 127.0.0.1 --port 43175
```

認証の動作:
- `EPISTEME_API_KEYS`が設定されている場合、リクエストに以下を含める必要があります:
```http
Authorization: Bearer <api-key>
```
- キーが設定されていない場合、認証はスキップされます（開発モード）。
- `GET /health`はヘルスチェックのため常に公開されています。

注意:
- `epis service`は同じMCP HTTPモードをバックグラウンドで管理します（`start|stop|status|enable|disable`）。
- 古い`--proxy`の例は非推奨です。`mcp --http`/`service`を直接使用してください。

## MCPとは？

[Model Context Protocol（MCP）](https://modelcontextprotocol.io)は、AIアシスタントが外部ツールやデータソースにアクセスできるようにするオープン標準です。Epistemeは6つのMCPツールを提供し、AIエージェントがソフトウェアエンジニアリングの知識に直接アクセスできるようにします。

---

## クイックスタート（Claude Code）

### 1. Epistemeのインストール

```bash
# インストール（Rust 1.95以上が必要）
cargo install --git https://github.com/epicsagas/Episteme

# エージェントとMCPサーバーをClaude Codeにインストール
# （データをシードし、MCPを自動設定）
epis install claude
```

> データのダウンロードに失敗する場合は、ソースからインストール: `git clone` → `cargo build --release` → `epis install --local`

### 2. インストールの確認

`~/.claude/claude_desktop_config.json`を確認:

```json
{
  "mcpServers": {
    "episteme": {
      "command": "episteme",
      "args": ["mcp"]
    }
  }
}
```

### 3. 利用開始

Claude Codeを再起動します。6つのEpistemeツールにアクセスできるようになります:

```
ユーザー: "God Objectスメルを修正する最善の方法は？"

Claude（search_knowledgeツールを使用）:
  → "God Object"のリファクタリングを検索
  → 結果: RF-018 (Extract Class)、RF-023 (Move Method)

Claude: "God Objectアンチパターン（SMELL-03）は単一責任の原則（LAW-001）に違反しています。
ベストなリファクタリング:

1. Extract Class（RF-018） - 関連するメソッド/フィールドを新しいクラスに移動
2. Move Method（RF-023） - メソッドを適切なクラスに再配置

どちらもSOLID原則を強制し、テスト容易性を向上させます。"
```

---

## MCPツールリファレンス

### 1. `search_knowledge`

**目的**: 全エンティティ（パターン、法則、リファクタリング、スメル）のセマンティック検索

**パラメータ:**
```typescript
{
  query: string          // 自然言語クエリ
  top_k?: number         // 返却する結果数（デフォルト: 5）
  filter_type?: string   // "pattern", "law", "refactoring", "smell"
}
```

**戻り値:**
```typescript
{
  results: [{
    entity_id: string     // 例: "DP-023"
    title: string
    entity_type: string
    similarity: number    // 0.0-1.0
    summary: string
  }]
}
```

**会話例:**
```
ユーザー: "コードのテスト容易性を向上させるにはどうすればいい？"

Claude呼び出し: search_knowledge({
  query: "improve testability",
  top_k: 3
})

結果:
- LAW-001: 単一責任の原則
- DP-018: 依存性の注入
- RF-042: インターフェースの抽出

Claude: "テスト容易性を向上させる3つのアプローチ:
1. SRP（LAW-001）の適用 - 1つのクラス、1つの変更理由
2. 依存性の注入（DP-023）の使用 - 依存関係を注入
3. インターフェースの抽出（RF-042） - 外部依存をモック化"
```

---

### 2. `get_entity`

**目的**: IDで特定のエンティティの詳細を取得

**パラメータ:**
```typescript
{
  entity_id: string   // 例: "DP-023", "RF-001", "SMELL-01"
}
```

**戻り値:**
```typescript
{
  entity_id: string
  title: string
  type: string
  description: string
  implementation: string    // コード例
  when_to_use: string
  benefits: string[]
  trade_offs: string[]
  related_entities: {
    relation_type: string
    target_id: string
    description: string
  }[]
}
```

**会話例:**
```
ユーザー: "Strategyパターンについて詳しく説明して"

Claude呼び出し: get_entity({ entity_id: "DP-023" })

結果:
- Strategyパターンの完全な説明
- 使用場面（複数アルゴリズム、ランタイム切替）
- 実装（Context、Strategyインターフェース、ConcreteStrategies）
- 利点: 開放閉鎖の原則、ランタイム柔軟性
- トレードオフ: クラス数の増加
- 関連: LAW-002（開放閉鎖の原則）、RF-034（条件分岐をポリモーフィズムで置換）

Claude: "Strategyパターン（DP-023）はアルゴリズムを別々のクラスにカプセル化します...
[コード例付きの詳細な説明]

関連概念:
- 開放閉鎖の原則（LAW-002）を強制
- 条件分岐をポリモーフィズムで置換（RF-034）で適用されることが多い"
```

---

### 3. `get_neighbors`

**目的**: 特定のエンティティに接続されているエンティティを探索

**パラメータ:**
```typescript
{
  entity_id: string
  relation_type?: string   // "solves", "enforces", "violates", "relates_to"
  max_depth?: number       // デフォルト: 1
}
```

**戻り値:**
```typescript
{
  neighbors: [{
    entity_id: string
    title: string
    relation_type: string
    distance: number
  }]
}
```

**会話例:**
```
ユーザー: "Long Methodスメルを解決するリファクタリングは？"

Claude呼び出し: get_neighbors({
  entity_id: "SMELL-01",
  relation_type: "solved_by"
})

結果:
- RF-001: Extract Method
- RF-034: Decompose Conditional
- RF-056: Replace Temp with Query

Claude: "Long Method（SMELL-01）は以下で解決できます:
1. Extract Method（RF-001） - より小さなメソッドに分割
2. Decompose Conditional（RF-034） - 複雑な条件を簡略化
3. Replace Temp with Query（RF-056） - 一時変数を排除"
```

---

### 4. `find_path`

**目的**: 2つのエンティティ間の接続を見つける（例: 法則がパターンにどう関係するか）

**パラメータ:**
```typescript
{
  from_id: string
  to_id: string
  max_depth?: number   // デフォルト: 5
}
```

**戻り値:**
```typescript
{
  path: [{
    entity_id: string
    title: string
    relation_to_next: string
  }],
  path_found: boolean
  hops: number
}
```

**会話例:**
```
ユーザー: "単一責任の原則はExtract Classリファクタリングにどう関係していますか？"

Claude呼び出し: find_path({
  from_id: "LAW-001",
  to_id: "RF-018"
})

結果のパス:
LAW-001 (SRP) --enforces--> DP-007 (Facade) --achieved_via--> RF-018 (Extract Class)

Claude: "SRP（LAW-001）はExtract Class（RF-018）に2ホップで接続されています:
1. SRPはFacadeパターン（DP-007）を強制
2. FacadeはExtract Class（RF-018）で実現されることが多い

インサイト: クラスが複数の責任を持つ場合、
クラスの抽出（RF-018）はSRP（LAW-001）の充足に役立ちます。"
```

---

### 5. `analyze_code`

**目的**: AST分析によるコードスメル検出

**パラメータ:**
```typescript
{
  code: string
  language: string   // "python", "java", "typescript"など
  min_confidence?: number   // デフォルト: 0.5
}
```

**戻り値:**
```typescript
{
  smells_detected: number,
  detections: [{
    smell_id: string
    smell_name: string
    confidence: number
    location: string
    metrics: {
      loc: number
      cyclomatic_complexity: number
      parameter_count: number
    }
  }]
}
```

**会話例:**
```
ユーザー: "この決済検証コードをレビューして"
[コードを貼り付け]

Claude呼び出し: analyze_code({
  code: "...",
  language: "python"
})

結果:
- SMELL-01: Long Method（信頼度 0.87、LOC=45）
- SMELL-08: Long Parameter List（信頼度 0.92、params=9）

Claude: "2つのコードスメルを発見しました:

1. Long Method（SMELL-01、信頼度 87%）
   - 45行のコード（閾値: 20）
   - 高いサイクロマティック複雑度（12）
   - 推奨: Extract Method（RF-001）

2. Long Parameter List（SMELL-08、信頼度 92%）
   - 9個のパラメータ（閾値: 4）
   - 推奨: Introduce Parameter Object（RF-029）"
```

---

### 6. `suggest_refactorings`

**目的**: 検出されたスメルに対するランク付けされたリファクタリング提案を取得

**パラメータ:**
```typescript
{
  code: string
  language: string
  top_k?: number   // 提案数（デフォルト: 3）
  min_confidence?: number
}
```

**戻り値:**
```typescript
{
  smells_analyzed: number,
  results: [{
    smell: {
      smell_id: string
      smell_name: string
      confidence: number
    },
    suggestions: [{
      refactoring_id: string
      title: string
      priority_score: number
      severity: string
      effort: string
      principles: string[]
    }]
  }]
}
```

**会話例:**
```
ユーザー: "このコードのどこをリファクタリングすべき？"
[大きなクラスを貼り付け]

Claude呼び出し: suggest_refactorings({
  code: "...",
  language: "java",
  top_k: 3
})

God Objectスメルに対するランク付けされた提案:
1. RF-018: Extract Class（優先度 0.89、重要度 High、労力 Medium）
2. RF-023: Move Method（優先度 0.76、重要度 Medium、労力 Low）
3. RF-042: Extract Interface（優先度 0.65、重要度 Low、労力 Low）

Claude: "このコードはGod Object（SMELL-03）の特徴を示しています。

リファクタリングの優先順位:

1. Extract Class（RF-018） - 優先度 0.89
   - 重要度: High
   - 労力: Medium
   - 決済関連のメソッドをPaymentProcessorクラスに移動
   - 強制: SRP（LAW-001）、SoC（LAW-012）

2. Move Method（RF-023） - 優先度 0.76
   - 重要度: Medium
   - 劳力: Low
   - 検証メソッドをValidatorクラスに再配置
   - 凝集度向上のクイックウィン

まず#2（Move Method）で即座に改善し、
その後#1（Extract Class）で長期的な設計改善を行ってください。"
```

---

## AIエージェント

Epistemeには4つの専用エージェントが含まれており、接続されたシステムとして連携動作します。各エージェントはナレッジグラフツールを使用して分析を行い、他のエージェントに引き継げるインタラクティブなフォローアップオプションを提示します。

### エージェントネットワーク

```
code-reviewer ←→ episteme-advisor
      ↕                ↕
architecture-analyst ←→ episteme-researcher
      ↕
refactoring-expert（外部）
```

各エージェントはレポートの最後に**次のステップ**を提示します。これは他のエージェントやツールに接続するインタラクティブなオプションです。これにより、検出から修復までエンドツーエンドのワークフローが構築されます。

---

### 1. `code-reviewer`

**使用場面**: コードのスメル、SOLID違反、リファクタリングの優先順位付けのレビュー

**他との違い**: 検出にとどまらず、スメル間の因果関係チェーン（どのスメルが根本原因で、どれが下流の症状か）を追跡し、言語エコシステムの慣習（Rust、Goなど）に対して結果を検証し、コンテキストに応じて重大度を調整します。

**使用ツール**: `analyze_code` → `suggest_refactorings` → `get_entity` → `get_neighbors` → `find_path`

**プロンプト例:**
```
"この認証モジュールをレビューして"
[コードを貼り付け]
```

**エージェントの応答:**
```
# Epistemeコードレビュー

## サマリー
AuthenticationModuleには根本原因となるGod Objectスメルがあり、
3つの下流の問題を引き起こしています。根本原因を修正することで
改善が波及します。

## スメル検出
| スメル | 場所 | 重大度 | 信頼度 | 関連原則 |
|-------|----------|----------|------------|-------------------|
| God Object (SMELL-03) | auth.rs:1-420 | Critical | confirmed | LAW-001 (SRP) |
| Long Method (SMELL-01) | auth.rs:88-142 | High | confirmed | LAW-001 (SRP) |
| Divergent Change (SMELL-09) | auth.rs:200-380 | Medium | confirmed | LAW-012 (SoC) |
| Feature Envy (SMELL-15) | auth.rs:290-333 | Medium | confirmed | LAW-003 (LSP) |

## 因果分析
根本原因:
- SMELL-03 → 原因 → SMELL-01, SMELL-09
  (God Objectの修正で改善が波及)

独立:
- SMELL-15 — 単独、下流への影響なし

## ランク付けされたリファクタリング
1. **Extract Class** (RF-018) — 優先度: Critical
   - 対象: SMELL-03（根本原因）
   - 波及効果: SMELL-01、SMELL-09も解決
   - やること: SessionManager、EmailVerifierクラスを抽出

## 次のステップ
1. リファクタリング — RF-018（Extract Class）を適用 → refactoring-expertエージェントを使用
2. 深掘り — SMELL-03の根本原因チェーンを探索 → episteme-advisorを使用
3. アーキテクチャ確認 — 構造的影響を評価 → architecture-analystを使用
4. 詳細を学ぶ — SMELL-03の完全な説明を取得 → get_entity(detail_level="full")
```

**検証**: スメルを確認する前に、エージェントは言語のイディオム（例: Rustでは大きな構造体は普通）、ドメインコンテキスト（テストファイル、生成コード）、実際の影響（表面的か開発阻害か）とクロスチェックします。

---

### 2. `episteme-advisor`

**使用場面**: エンジニアリングの意思決定（パターン選択、アーキテクチャのトレードオフ、根本原因の深掘り）

**使用ツール**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**プロンプト例:**
```
"ConfigManagerクラスにSingletonを使うべき？"
```

**エージェントの応答:**
```
# アドバイザリー: ConfigManager向けSingleton vs DI

## 適用可能なエンティティ
### 1. Singletonパターン (DP-006) - 主要
- **概要**: クラスのインスタンスが1つのみであることを保証
- **あなたへの意味**: グローバル設定アクセスだが隠れた結合
- **推奨**: 避ける — テスト容易性に違反

### 2. 依存性の注入 (DP-018) - 代替
- **概要**: コンストラクタ経由で依存関係を注入
- **あなたへの意味**: 明示的な依存関係、モック化が容易
- **推奨**: ConfigManagerにはこちらを推奨

## トレードオフ
- Singleton (DP-006) は利便性があるが DI (DP-018) に違反
- DI はボイラープレートが増えるが LAW-005 (DIP) 準拠を可能に
- 解決策: 設定プロバイダーインターフェース付きDIを使用

## アクションプラン
1. 即時: ConfigManagerをコンストラクタ経由で注入
2. 短期: ConfigProviderインターフェースを抽出
3. 長期: 環境別設定戦略の検討

## 次のステップ
1. 実装 — DIパターンを適用 → code-reviewerで検証
2. 代替案の探索 — アプローチを比較 → episteme-researcherを使用
3. アーキテクチャへの影響 — 下流の影響を評価 → architecture-analystを使用
```

---

### 3. `episteme-researcher`

**使用場面**: ナレッジグラフ内の関係性の探索、代替案の検索

**使用ツール**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**プロンプト例:**
```
"Observerパターンが強制する法則は？"
```

**エージェントの応答:**
```
# 調査レポート: Observerパターンが強制する法則

## 主要エンティティ
1. **Observerパターン** (DP-012) - 振る舞い
   - 定義: オブジェクト間の1対多の依存関係を定義
   - 1つのオブジェクトの変更が他のオブジェクトに自動的に伝播

## 接続マップ
DP-012 --enforces--> LAW-002 (開放閉鎖の原則)
DP-012 --enforces--> LAW-005 (依存関係逆転の原則)
DP-012 --enforces--> LAW-001 (単一責任の原則)

## 矛盾とトレードオフ
- Observer (DP-012) は疎結合を可能にするが、予期しない更新チェーンを
  引き起こす可能性 — 中央化するMediator (DP-014) と対照的

## 次のステップ
1. アドバイスを取得 — 結果をガイダンスに変換 → episteme-advisorを使用
2. コードを確認 — コードベースでObserverを検出 → code-reviewerを使用
3. アーキテクチャを評価 — 構造的適合性を評価 → architecture-analystを使用
```

---

### 4. `architecture-analyst`

**使用場面**: 法則、パターン、構造的リスクに対するシステムアーキテクチャの評価

**使用ツール**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**プロンプト例:**
```
"このマイクロサービスアーキテクチャはコンウェイの法則に従っていますか？"
[アーキテクチャを説明]
```

**エージェントの応答:**
```
# アーキテクチャ分析: マイクロサービス

## 法則と原則の準拠状況
| 原則 | ステータス | 証拠 | 影響 |
|-----------|--------|----------|--------|
| コンウェイの法則 (LAW-042) | 違反 | 出荷が2チームにまたがる | 調整オーバーヘッド |
| SRP (LAW-001) | リスクあり | Analyticsがすべてに依存 | 密結合 |
| 境界付けられたコンテキスト (LAW-031) | 違反 | 明確なドメイン境界なし | データ共有の混乱 |

## 主要な緊張関係
- コンウェイの法則 (LAW-042) はチーム↔サービスの整合を要求
  しかしShippingサービスがCommerce + Platformチームにまたがっている
- 経路: LAW-042 → related_to → LAW-001 → enforced_by → DP-026 (Strangler Fig)

## アーキテクチャ推奨
1. **Critical**: ShippingをCommerceチームに移動 — LAW-042は調整の失敗を予測
2. **High**: Analytics向けにEvent Busを導入 — 非同期イベントで疎結合化
3. **Medium**: 境界付けられたコンテキストを定義 — ドメインにサービス境界を整合

## 準拠スコア
- 全体: 5/10 | 構造: 4/10 | スケーラビリティ: 6/10 | 保守性: 5/10

## 次のステップ
1. アドバイスを取得 — 主要な緊張関係を解決 → episteme-advisorを使用
2. コードを確認 — 構造的スメルを検出 → code-reviewerを使用
3. 代替案を調査 — より良いパターンを検索 → episteme-researcherを使用
```

---

## ワークフローチェーン

エージェントとツールはエンドツーエンドのパイプラインに接続されます。各チェーンはレポートを生成し、その後にインタラクティブなフォローアップオプションが続きます。

### チェーン1: コードレビューパイプライン
```
analyze_code → suggest_refactorings → get_neighbors("solved_by")
  → find_path(smell_A, smell_B) → 因果グラフ付きレポート
  → ユーザー選択: 修正を適用 / 深掘り / アーキテクチャ確認 / 詳細を学ぶ
```

### チェーン2: アーキテクチャレビューパイプライン
```
search_knowledge → get_entity → get_neighbors("enforces")
  → get_neighbors("violates") → find_path → 準拠レポート
  → ユーザー選択: リファクタリング計画 / アドバイザリー / 代替案の調査
```

### チェーン3: 問題診断パイプライン
```
search_knowledge(症状) → get_entity → get_neighbors("solved_by")
  → 根本原因レポート → ユーザー選択: 修正を適用 / アドバイザリー / 検証
```

### チェーン4: 学習パイプライン
```
search_knowledge(トピック) → get_entity → get_neighbors("related_to")
  → 概念マップ → ユーザー選択: コード例 / コードに適用 / 比較
```

### クロスツールチェーンルール

すべてのツール呼び出しは自然に次に繋がります:

| 呼び出し後... | 必ず次に実行... |
|-------------------|--------------------------|
| `analyze_code` | 検出されたスメルに対する`suggest_refactorings` |
| `suggest_refactorings` | 代替案のための`get_neighbors(smell_id, "solved_by")` |
| `search_knowledge` | 上位1-2件の結果に対する`get_entity` |
| `get_entity`（スメル） | 影響を受ける原則のための`get_neighbors(id, "violates")` |
| `get_entity`（パターン） | 強制される法則のための`get_neighbors(id, "enforces")` |
| 複数スメル検出時 | 因果マッピングのための`find_path(smell_A, smell_B)` |

---

## その他のツール向けインストール

### Cursor

```bash
epis install cursor
```

`~/.cursor/mcp.json`にMCP設定を追加:
```json
{
  "mcpServers": {
    "episteme": {
      "command": "episteme",
      "args": ["mcp"]
    }
  }
}
```

### Codex（OpenAI）

```bash
epis install codex
```

プロジェクトルートにエージェント定義付きの`AGENTS.md`を生成。

### カスタムMCP統合

ツールがMCPをサポートしている場合、手動で設定:

```json
{
  "mcpServers": {
    "episteme": {
      "command": "/path/to/episteme",
      "args": ["mcp"],
      "env": {
        "EPISTEME_DATA_DIR": "~/.episteme/data",
        "EPISTEME_DB_PATH": "~/.episteme/db/episteme.db"
      }
    }
  }
}
```

---

## バックグラウンドサービスとしての実行

パフォーマンスを向上させるため、Episteme MCPを永続的なHTTPプロキシとして実行:

```bash
# バックグラウンドサービスを起動
epis service start

# ステータス確認
epis service status
# 出力: Running on http://localhost:43175 (PID 12345)

# 起動時の自動開始を有効化（macOS）
epis service enable

# サービス停止
epis service stop
```

HTTPプロキシを使用するようにMCP設定を更新:

```json
{
  "mcpServers": {
    "episteme": {
      "command": "episteme",
      "args": ["mcp", "--proxy", "http://localhost:43175"]
    }
  }
}
```

ログ: `~/.episteme/logs/mcp.out.log`

---

## トラブルシューティング

### ツールがClaudeに表示されない

1. 設定ファイルの存在確認: `cat ~/.claude/claude_desktop_config.json`
2. epistemeがPATHにあることを確認: `which episteme`
3. MCPを直接テスト: `episteme mcp`
4. ログを確認: `tail -f ~/.episteme/logs/mcp.err.log`

### "Database not found"エラー

```bash
# ナレッジデータベースを再構築
epis build --rebuild
```

### 検索レスポンスが遅い

```bash
# GPUアクセラレーションを使用
epis build --gpu

# またはバックグラウンドサービスとして実行（ウォームアップが速い）
epis service start
```

### エージェントがツールを使用しない

エージェントにツール呼び出し機能があることを確認してください。Claude Codeの場合:
```
ユーザー: "Epistemeを使ってリトライロジックのパターンを見つけて"
              ^^^^ ツールの使用を明示的に指定
```

---

## 高度な設定: カスタムナレッジ統合

Episteme（一般的なナレッジ）とAlcove（チームナレッジ）を組み合わせる:

```json
{
  "mcpServers": {
    "episteme": {
      "command": "episteme",
      "args": ["mcp"]
    },
    "alcove": {
      "command": "npx",
      "args": ["-y", "@joshuarileydev/alcove-mcp"]
    }
  }
}
```

デュアルソースパターンについては[Alcove統合ガイド](./alcove-integration.md)を参照してください。

---

## API代替手段

AIツールがMCPをサポートしていない場合、REST APIを使用:

```bash
# APIサーバーを起動
docker-compose up -d

# 任意のツールから使用
curl http://localhost:8000/search?q=strategy+pattern
```

エンドポイントについては[APIドキュメント](./api.md)を参照してください。

---

## 自動トリガー（Claude Code）

自然言語で問題を説明すると、Claude Codeが自動的に意図を検出し、適切なEpistemeツールを呼び出します。**Epistemeを明示的に指定する必要はありません。** 以下に正確なトリガーパターンと例を示します。

### 仕組み

```
自然言語の入力
    ↓ Claudeがキーワード/パターンを検出
    ↓ Epistemeツールが自動的に呼び出される
    ↓ ナレッジグラフが検証済みデータを返す
    ↓ (デザインパターン · コードスメル · リファクタリング手法 · エンジニアリング法則)
    ↓ Claudeの回答が証拠に基づいている
```

> **注意:** これはプロンプトベースの自動トリガーであり、ハードフックではありません。呼び出しを保証するには、`/episteme`スキルを直接使用してください。

### コード構造の問題

| 発言例 | Epistemeが検出するもの | 自動ツール呼び出し |
|-------------------------|-----------------------|---------------------|
| "このクラスはやりすぎ", "このファイルは300行を超えている" | God Class、Large Class、単一責任 | `search_knowledge("god class large class single responsibility")` |
| "この関数は長すぎる", "このメソッドの行数が多すぎる" | Long Method | `search_knowledge("long method extract method")` |
| "コードが複雑すぎる", "追うのが難しい" | 複雑さ、認知的過負荷 | `search_knowledge("complexity smell cognitive overload")` |
| "あちこちにコピペした", "重複したロジックがある" | 重複コード、クローン | `search_knowledge("duplicated code clone smell")` |

### 結合と依存関係の問題

| 発言例 | Epistemeが検出するもの | 自動ツール呼び出し |
|-------------------------|-----------------------|---------------------|
| "ビジネスロジックがDBを直接呼んでいる" | 結合、永続化、リポジトリ | `search_knowledge("coupling persistence repository data access layer")` |
| "Xを変更するとYが壊れる", "変更があちこちに波及する" | もろい結合、変更伝播 | `search_knowledge("brittle coupling change propagation rigidity")` |
| "新しい型を追加するたびにあちこちを触る", "switch-caseが増え続ける" | 開放閉鎖、Strategy、ポリモーフィズム | `search_knowledge("open closed principle strategy polymorphism")` |

### テストと品質の問題

| 発言例 | Epistemeが検出するもの | 自動ツール呼び出し |
|-------------------------|-----------------------|---------------------|
| "これはテストしづらい", "これのユニットテストが書けない" | テスト容易性、依存性の注入 | `search_knowledge("testability dependency injection mockability")` |

### パフォーマンスと並行性の問題

| 発言例 | Epistemeが検出するもの | 自動ツール呼び出し |
|-------------------------|-----------------------|---------------------|
| "APIが遅い", "レスポンスタイムが高すぎる" | N+1クエリ、遅延読み込み、キャッシュ | `search_knowledge("N+1 query lazy loading caching performance")` |
| "これはスレッドセーフ？", "並行性の問題はある？" | スレッドセーフ性、競合状態 | `search_knowledge("thread safety race condition shared mutable state")` |

### コードレビューと分析

| 発言例 | 自動ツール呼び出し |
|-------------------------|---------------------|
| コードスニペットを共有してレビューを依頼 | `analyze_code(code)` → `suggest_refactorings(code)` |
| "このコードを改善して", "これをリファクタリングして" | `suggest_refactorings(code)` |

### アーキテクチャ議論

| 発言例 | 自動ツール呼び出し |
|-------------------------|---------------------|
| "マイクロサービスかモノリスか", "どう分割すべき？" | `search_knowledge("monolith microservice decomposition bounded context")` |
| "このアーキテクチャは大丈夫？", "この設計をレビューして" | `search_knowledge("layered architecture coupling responsibility")` |

### 実際の会話例

**例1 — God Objectの検出:**

```
あなた: "UserServiceが手に負えなくなってきた。認証、セッション管理、
        メール確認、パスワードリセットを全部扱っている。"

Claude（search_knowledgeを自動呼び出し）:
  → 検出: God Object / 単一責任違反
  → 結果: SMELL-03 (God Object), LAW-001 (SRP), RF-018 (Extract Class)

Claude: "UserServiceはGod Objectの特徴を示しています（SMELL-03）。
  単一責任の原則（LAW-001）に違反しています。

  推奨されるリファクタリング:
  1. Extract Class（RF-018） — セッションロジック用にSessionManagerを作成
  2. Move Method（RF-023） — メール確認をEmailServiceに移動
  3. Extract Interface（RF-042） — AuthServiceインターフェースを定義

  まず#2でクイックウィンを得て、その後#1で構造的な改善を行ってください。"
```

**例2 — パフォーマンス調査:**

```
あなた: "商品一覧APIが本当に遅い。個別だと問題ないけど
        一覧ページは5秒以上かかる。"

Claude（search_knowledgeを自動呼び出し）:
  → 検出: N+1クエリ問題
  → 結果: パフォーマンスパターン、遅延読み込み、キャッシュ戦略

Claude: "これはN+1クエリ問題のようです — 各商品が個別のDB呼び出しを
  トリガーしており、バッチクエリになっていません。

  ナレッジグラフからの解決策:
  1. バッチ読み込み / Eager読み込み — 関連データを1クエリで取得
  2. Data Mapper（DP-019） — クエリロジックをドメインオブジェクトから分離
  3. 遅延読み込み（RF-050）の適用 — 関連はオンデマンドでのみ読み込み

  コードのN+1パターンを確認しましょうか？"
```

**例3 — スニペット付きコードレビュー:**

```
あなた: [Pythonコードを貼り付け]
       "これをレビューして？何かがおかしい気がする。"

Claude（analyze_code → suggest_refactoringsを自動呼び出し）:
  → analyze_codeが検出: Long Method（信頼度 0.87）、Long Parameter List（0.92）
  → suggest_refactoringsがランク付けされた提案を返却

Claude: "2つのコードスメルを発見しました:

  1. Long Method（SMELL-01） — 45行、サイクロマティック複雑度 12
     → Extract Method（RF-001）、優先度 0.89

  2. Long Parameter List（SMELL-08） — 9パラメータ
     → Introduce Parameter Object（RF-029）、優先度 0.92

  まずRF-029（Parameter Object）から始めてください — 最も優先度が高く、
  その後のExtract Methodがやりやすくなります。"
```

---

## 次のステップ

1. **エージェントを試す**: episteme-advisorに"Singletonを使うべき？"と聞く
2. **コードを分析**: 関数を貼り付けてcode-reviewerにスメルを確認させる
3. **グラフを探索**: episteme-researcherを使ってパターン間の関係を見つける
4. **カスタムワークフロー**: ツールを組み合わせる（analyze → suggest → search）

その他の例は以下を参照:
- [Alcove統合](./alcove-integration.md) — チームナレッジ + Episteme
- [モニタリング設定](../../monitoring/README.md) — パターン利用状況の追跡
- [APIリファレンス](./api.md) — RESTエンドポイント
