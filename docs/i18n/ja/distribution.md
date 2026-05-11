# 配布パッケージング（Rust CLI）

このガイドでは、Rust CLIを使用して他のユーザー向けのリリースデータアーカイブを作成する方法を説明します。

## コマンド

```bash
episteme dist
```

## `episteme dist`に含まれるもの
- `raw/`
- `meta/`
- `data/`（存在する場合）
- `db/episteme.db`（エンベディングDB）

出力アーカイブ:
- `dist/episteme-data-<version>.tar.gz`

## 自動ビルドの動作
- `~/.episteme/db/episteme.db`が存在しない場合、`episteme dist`が先に自動的に`epis build`を実行します。
- 構築されたDBはアーカイブに含めるため、プロジェクトローカルの`db/`ディレクトリにもコピーされます。
- `epis install --local`はアーカイブ（またはソースツリーフォールバック）からデータをシードし、RAGインデックスを`~/.episteme/`に自動構築します。

## オプション
- `--out-dir <DIR>`: 出力ディレクトリ（デフォルト: `dist`）
- `--no-db`: DBの同梱をスキップ
- `--skip-build`: DBが存在しない場合に自動ビルドしない

例:

```bash
# デフォルトでdist/にパッケージング
episteme dist

# カスタム出力ディレクトリ
episteme dist --out-dir release

# メタデータのみパッケージング（DBなし）
episteme dist --no-db

# 厳格モード: DBが存在しない場合は失敗
episteme dist --skip-build
```

## 検証
アーカイブ生成後、構造を確認:

```bash
tar -tzf dist/episteme-data-*.tar.gz | head -n 30
```

以下のエントリが表示されるはずです:
- `episteme-data-<version>/raw/...`
- `episteme-data-<version>/meta/...`
- `episteme-data-<version>/db/episteme.db`（`--no-db`の場合を除く）
