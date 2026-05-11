# 發行封裝（Rust CLI）

本指南說明如何使用 Rust CLI 為其他使用者建立發行資料封存檔。

## 命令

```bash
episteme dist
```

## `episteme dist` 包含的內容
- `raw/`
- `meta/`
- `data/`（若存在）
- `db/episteme.db`（嵌入資料庫）

輸出封存檔：
- `dist/episteme-data-<version>.tar.gz`

## 自動建置行為
- 若 `~/.episteme/db/episteme.db` 不存在，`episteme dist` 會自動先執行 `epis build`。
- 建置完成的資料庫也會複製到專案本地的 `db/` 目錄，以便包含在封存檔中。
- `epis install --local` 會從封存檔（或回退至原始碼目錄）植入資料，並自動建置 RAG 索引至 `~/.episteme/`。

## 選項
- `--out-dir <DIR>`：輸出目錄（預設：`dist`）
- `--no-db`：跳過資料庫包含
- `--skip-build`：若資料庫缺少時不自動建置

範例：

```bash
# 預設封裝至 dist/
episteme dist

# 自訂輸出目錄
episteme dist --out-dir release

# 僅封裝元資料（不含資料庫）
episteme dist --no-db

# 嚴格模式：若資料庫缺少則失敗
episteme dist --skip-build
```

## 驗證
產生封存檔後，驗證結構：

```bash
tar -tzf dist/episteme-data-*.tar.gz | head -n 30
```

您應該會看到以下路徑下的項目：
- `episteme-data-<version>/raw/...`
- `episteme-data-<version>/meta/...`
- `episteme-data-<version>/db/episteme.db`（除非使用 `--no-db`）
