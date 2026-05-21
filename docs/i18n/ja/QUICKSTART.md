# Episteme — クイックスタートガイド

2分以内にEpistemeを使い始めましょう。

---

## 前提条件

- **Rust 1.95以上**（エディション2024が必要）— [rustupでインストール](https://rustup.rs)
- インターネット接続（初期データダウンロード用）

---

## オプション1: AIツール統合（推奨）

**対象:** Claude Code、Cursor、Codex、Geminiユーザー

```bash
# 1. Epistemeをインストール
cargo install --git https://github.com/epicsagas/Episteme

# 2. AIツールにインストール（データダウンロード、MCP設定、エージェントコピー）
epis install claude      # Claude Code
epis install cursor      # Cursor
epis install codex       # OpenAI Codex
epis install gemini      # Antigravity
epis install all         # すべてのツールに一括インストール
```

> `epis install claude`でデータのダウンロードに失敗する場合は、下記のソースからのインストールを使用してください。

**以上です。** AIツールを再起動するとEpistemeが有効になります。

---

## オプション2: Docker（Rust不要）

```bash
docker-compose up -d

# アクセス先
# API:       http://localhost:8000
# ヘルスチェック:    http://localhost:8000/health
```

Docker経由でMCP統合する場合、MCP設定に以下を追加してください：
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

---

## オプション3: ソースからビルド

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme

# ビルド
cargo build --release

# データをシードしてベクトルDBを構築（ビルドは自動的に実行）
./target/release/epis install --local
```

---

## グラフ可視化

EpistemeにはインタラクティブなD3-forceグラフビューアーが含まれています：

```bash
episteme web               # デフォルト: http://localhost:8080
episteme web --port 9001   # カスタムポート
episteme web --host 0.0.0.0 --port 8080  # ネットワークに公開
```

---

## よく使うコマンド

```bash
# コードのスメルを分析
epis analyze my_code.py --language python
epis analyze my_code.py --json

# リファクタリング提案を取得
episteme infer my_code.py --top-k 5

# ナレッジグラフを探索
epis explore "strategy pattern"
epis graph path DP-005 RF-001

# サーバーを起動
epis api              # REST API :8000
episteme mcp --http       # MCPサーバー :43175
episteme web --port 8080  # Web UI

# バックグラウンドMCPデーモン（HTTPプロキシ）
epis service start
epis service status
epis service stop

# リリースアーカイブを作成
episteme dist --out-dir release
```

---

## トラブルシューティング

### "Database not found"
```bash
epis install claude   # データアーカイブを再ダウンロード
# または
epis install --local
```

### "Port already in use"
```bash
episteme web --port 9001
epis api --port 9000
```

---

## 次のステップ

- **[README](../../README.md)** — 機能の完全な概要とアーキテクチャ
- **[MCP統合ガイド](./mcp-integration-guide.md)** — ツールリファレンスとエージェント例
- **[APIリファレンス](./api.md)** — RESTエンドポイント
- **[コントリビュート](../../CONTRIBUTING.md)** — 開発ワークフロー
