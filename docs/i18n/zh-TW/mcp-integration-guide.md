# MCP 整合指南

> 將 Episteme 的知識圖譜整合至 Claude Code、Cursor 及其他 MCP 相容的 AI 工具

## Rust MCP HTTP 模式（目前版本）
直接使用獨立的 HTTP 傳輸：

```bash
# 透過 HTTP 啟動 MCP
episteme mcp --http --host 127.0.0.1 --port 43175
```

身份驗證行為：
- 若已設定 `EPISTEME_API_KEYS`，請求必須包含：
```http
Authorization: Bearer <api-key>
```
- 若未設定金鑰，則跳過身份驗證（開發模式）。
- `GET /health` 供健康檢查使用，一律為公開。

注意：
- `epis service` 在背景管理相同的 MCP HTTP 模式（`start|stop|status|enable|disable`）。
- 較舊的 `--proxy` 範例已棄用；請直接使用 `mcp --http`/`service`。

## 什麼是 MCP？

[模型上下文協定（MCP）](https://modelcontextprotocol.io)是一項開放標準，允許 AI 助理存取外部工具和資料來源。Episteme 提供 6 個 MCP 工具，讓 AI 代理能直接存取軟體工程知識。

---

## 快速開始（Claude Code）

### 1. 安裝 Episteme

```bash
# 安裝（需要 Rust 1.95+）
cargo install --git https://github.com/epicsagas/Episteme

# 安裝代理和 MCP 伺服器至 Claude Code
# （自動植入資料並設定 MCP）
epis install claude
```

> 若資料下載失敗，請使用原始碼安裝：`git clone` → `cargo build --release` → `epis install --local`

### 2. 驗證安裝

檢查 `~/.claude/claude_desktop_config.json`：

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

### 3. 開始使用

重新啟動 Claude Code。您現在擁有 6 個 Episteme 工具的存取權限：

```
使用者：「修復 God Object 壞味道的最佳方法是什麼？」

Claude（使用 search_knowledge 工具）：
  → 搜尋「God Object」的重構方式
  → 回傳：RF-018（Extract Class）、RF-023（Move Method）

Claude：「God Object 反模式（SMELL-03）違反了單一職責原則（LAW-001）。最佳重構方式：

1. Extract Class（RF-018）— 將相關方法/欄位移至新類別
2. Move Method（RF-023）— 將方法重新安置到適當的類別

兩者皆強制實踐 SOLID 原則並提升可測試性。」
```

---

## MCP 工具參考

### 1. `search_knowledge`

**用途**：跨所有實體（模式、法則、重構手法、壞味道）進行語意搜尋

**參數：**
```typescript
{
  query: string          // 自然語言查詢
  top_k?: number         // 回傳結果數（預設：5）
  filter_type?: string   // "pattern"、"law"、"refactoring"、"smell"
}
```

**回傳：**
```typescript
{
  results: [{
    entity_id: string     // 例如 "DP-023"
    title: string
    entity_type: string
    similarity: number    // 0.0-1.0
    summary: string
  }]
}
```

**對話範例：**
```
使用者：「如何讓我的程式碼更容易測試？」

Claude 呼叫：search_knowledge({
  query: "improve testability",
  top_k: 3
})

回傳：
- LAW-001：單一職責原則
- DP-018：依賴注入
- RF-042：擷取介面

Claude：「提升可測試性的三個關鍵方法：
1. 套用 SRP（LAW-001）— 一個類別，一個變更的理由
2. 使用依賴注入（DP-023）— 注入依賴關係
3. 擷取介面（RF-042）— 模擬外部依賴」
```

---

### 2. `get_entity`

**用途**：依 ID 取得特定實體的完整詳細資訊

**參數：**
```typescript
{
  entity_id: string   // 例如 "DP-023"、"RF-001"、"SMELL-01"
}
```

**回傳：**
```typescript
{
  entity_id: string
  title: string
  type: string
  description: string
  implementation: string    // 程式碼範例
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

**對話範例：**
```
使用者：「詳細說明 Strategy Pattern」

Claude 呼叫：get_entity({ entity_id: "DP-023" })

回傳：
- Strategy Pattern 的完整描述
- 使用時機（多種演算法、執行時期切換）
- 實作方式（Context、Strategy 介面、ConcreteStrategies）
- 優點：開放/封閉原則、執行時期彈性
- 權衡：增加類別數量
- 相關概念：LAW-002（開放/封閉）、RF-034（以多型取代條件式）

Claude：「Strategy Pattern（DP-023）將演算法封裝在獨立的類別中...
[提供詳細說明與程式碼範例]

相關概念：
- 強制實踐開放/封閉原則（LAW-002）
- 通常透過以多型取代條件式（RF-034）來套用」
```

---

### 3. `get_neighbors`

**用途**：探索與指定實體相連的實體

**參數：**
```typescript
{
  entity_id: string
  relation_type?: string   // "solves"、"enforces"、"violates"、"relates_to"
  max_depth?: number       // 預設：1
}
```

**回傳：**
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

**對話範例：**
```
使用者：「哪些重構手法可以解決 Long Method 壞味道？」

Claude 呼叫：get_neighbors({
  entity_id: "SMELL-01",
  relation_type: "solved_by"
})

回傳：
- RF-001：Extract Method
- RF-034：Decompose Conditional
- RF-056：Replace Temp with Query

Claude：「Long Method（SMELL-01）可透過以下方式解決：
1. Extract Method（RF-001）— 拆分為更小的方法
2. Decompose Conditional（RF-034）— 簡化複雜條件式
3. Replace Temp with Query（RF-056）— 消除暫時變數」
```

---

### 4. `find_path`

**用途**：尋找兩個實體之間的關聯（例如一項法則如何與一個模式相關）

**參數：**
```typescript
{
  from_id: string
  to_id: string
  max_depth?: number   // 預設：5
}
```

**回傳：**
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

**對話範例：**
```
使用者：「單一職責原則與 Extract Class 重構有何關聯？」

Claude 呼叫：find_path({
  from_id: "LAW-001",
  to_id: "RF-018"
})

回傳路徑：
LAW-001（SRP）--enforces--> DP-007（Facade）--achieved_via--> RF-018（Extract Class）

Claude：「SRP（LAW-001）透過 2 個跳躍連接到 Extract Class（RF-018）：
1. SRP 強制實踐 Facade Pattern（DP-007）
2. Facade 通常透過 Extract Class（RF-018）來達成

洞察：當一個類別具有多項職責時，擷取類別（RF-018）有助於滿足 SRP（LAW-001）。」
```

---

### 5. `analyze_code`

**用途**：透過 AST 分析偵測程式碼壞味道

**參數：**
```typescript
{
  code: string
  language: string   // "python"、"java"、"typescript" 等
  min_confidence?: number   // 預設：0.5
}
```

**回傳：**
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

**對話範例：**
```
使用者：「審查這段付款驗證程式碼」
[貼上程式碼]

Claude 呼叫：analyze_code({
  code: "...",
  language: "python"
})

回傳：
- SMELL-01：Long Method（信心度 0.87，LOC=45）
- SMELL-08：Long Parameter List（信心度 0.92，參數數量=9）

Claude：「發現 2 個程式碼壞味道：

1. Long Method（SMELL-01，信心度 87%）
   - 45 行程式碼（閾值：20）
   - 高迴圈複雜度（12）
   - 建議：Extract Method（RF-001）

2. Long Parameter List（SMELL-08，信心度 92%）
   - 9 個參數（閾值：4）
   - 建議：Introduce Parameter Object（RF-029）」
```

---

### 6. `suggest_refactorings`

**用途**：取得針對已偵測壞味道的排序重構建議

**參數：**
```typescript
{
  code: string
  language: string
  top_k?: number   // 建議數量（預設：3）
  min_confidence?: number
}
```

**回傳：**
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

**對話範例：**
```
使用者：「這段程式碼應該重構什麼？」
[貼上大型類別]

Claude 呼叫：suggest_refactorings({
  code: "...",
  language: "java",
  top_k: 3
})

回傳針對 God Object 壞味道的排序建議：
1. RF-018：Extract Class（優先級 0.89，嚴重性高，工作量中等）
2. RF-023：Move Method（優先級 0.76，嚴重性中，工作量低）
3. RF-042：Extract Interface（優先級 0.65，嚴重性低，工作量低）

Claude：「這段程式碼顯示 God Object 特徵（SMELL-03）。

重構優先順序：

1. Extract Class（RF-018）— 優先級 0.89
   - 嚴重性：高
   - 工作量：中等
   - 將付款相關方法移至 PaymentProcessor 類別
   - 強制實踐：SRP（LAW-001）、SoC（LAW-012）

2. Move Method（RF-023）— 優先級 0.76
   - 嚴重性：中
   - 工作量：低
   - 將驗證方法重新安置到 Validator 類別
   - 提升凝聚力的快速勝利

從 #2（Move Method）開始以獲得立即改善，
再處理 #1（Extract Class）以進行長期設計改善。」
```

---

## AI 代理

Episteme 包含 4 個專門的代理，它們作為一個互連系統協同運作。每個代理使用知識圖譜工具進行分析，並提供可互動的後續選項，這些選項可以交接給其他代理。

### 代理網路

```
code-reviewer ←→ episteme-advisor
      ↕                ↕
architecture-analyst ←→ episteme-researcher
      ↕
refactoring-expert（外部）
```

每個代理在報告結尾提供**後續步驟** — 連接到其他代理或工具的互動選項。這建立了從偵測到修復的端對端工作流程。

---

### 1. `code-reviewer`

**使用時機**：審查程式碼的壞味道、SOLID 違規和重構優先順序

**不同之處**：不僅止於偵測 — 追蹤壞味道之間的因果鏈（哪些壞味道是根本原因，哪些是下游症狀），根據語言生態系慣例（Rust、Go 等）驗證發現結果，並依情境校準嚴重性。

**使用的工具**：`analyze_code` → `suggest_refactorings` → `get_entity` → `get_neighbors` → `find_path`

**提示範例：**
```
「審查這個身份驗證模組」
[貼上程式碼]
```

**代理回應：**
```
# Episteme 程式碼審查

## 摘要
AuthenticationModule 有一個作為根本原因的 God Object 壞味道，
導致 3 個下游問題。修復根本原因將產生連鎖改善。

## 壞味道偵測
| 壞味道 | 位置 | 嚴重性 | 信心度 | 相關原則 |
|--------|------|--------|--------|----------|
| God Object（SMELL-03） | auth.rs:1-420 | 嚴重 | 已確認 | LAW-001（SRP） |
| Long Method（SMELL-01） | auth.rs:88-142 | 高 | 已確認 | LAW-001（SRP） |
| Divergent Change（SMELL-09） | auth.rs:200-380 | 中 | 已確認 | LAW-012（SoC） |
| Feature Envy（SMELL-15） | auth.rs:290-333 | 中 | 已確認 | LAW-003（LSP） |

## 因果分析
根本原因：
- SMELL-03 → 導致 → SMELL-01、SMELL-09
  （修復 God Object 將產生連鎖改善）

獨立問題：
- SMELL-15 — 獨立存在，無下游影響

## 排序重構
1. **Extract Class**（RF-018）— 優先級：嚴重
   - 解決：SMELL-03（根本原因）
   - 連鎖效果：同時解決 SMELL-01、SMELL-09
   - 操作方式：擷取 SessionManager、EmailVerifier 類別

## 後續步驟
1. 重構 — 套用 RF-018（Extract Class）→ 使用 refactoring-expert 代理
2. 深入分析 — 探索 SMELL-03 的根本原因鏈 → 使用 episteme-advisor
3. 架構檢查 — 評估結構影響 → 使用 architecture-analyst
4. 進一步了解 — 取得 SMELL-03 的完整說明 → get_entity(detail_level="full")
```

**驗證**：在確認壞味道之前，代理會交叉檢查語言慣用寫法（例如大型結構體在 Rust 中是正常的）、領域情境（測試檔案、自動產生的程式碼）以及實際影響（表面問題 vs 阻礙開發的問題）。

---

### 2. `episteme-advisor`

**使用時機**：做出工程決策（模式選擇、架構權衡、根本原因深入分析）

**使用的工具**：`search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**提示範例：**
```
「我的 ConfigManager 類別應該使用 Singleton 嗎？」
```

**代理回應：**
```
# 建議：Singleton vs DI 用於 ConfigManager

## 適用實體
### 1. Singleton Pattern（DP-006）— 主要
- **其內容**：確保類別只有一個實例
- **對您的意義**：全域組態存取但隱藏耦合
- **建議**：避免 — 違反可測試性

### 2. 依賴注入（DP-018）— 替代方案
- **其內容**：透過建構子注入依賴關係
- **對您的意義**：明確的依賴關係，易於模擬
- **建議**：ConfigManager 優先使用此方案

## 權衡
- Singleton（DP-006）提供便利性但違反 DI（DP-018）
- DI 增加了樣板程式碼但實現 LAW-005（DIP）合規
- 解決方案：使用 DI 搭配組態提供者介面

## 行動計畫
1. 立即：透過建構子注入 ConfigManager
2. 短期：擷取 ConfigProvider 介面
3. 長期：考慮依環境區分的組態策略

## 後續步驟
1. 實作 — 套用 DI 模式 → 使用 code-reviewer 驗證
2. 探索替代方案 — 比較方法 → 使用 episteme-researcher
3. 架構影響 — 評估下游效果 → 使用 architecture-analyst
```

---

### 3. `episteme-researcher`

**使用時機**：探索知識圖譜中的關聯、尋找替代方案

**使用的工具**：`search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**提示範例：**
```
「Observer Pattern 強制實踐了哪些法則？」
```

**代理回應：**
```
# 研究報告：Observer Pattern 強制實踐的法則

## 主要實體
1. **Observer Pattern**（DP-012）— 行為型
   - 定義：定義物件之間一對多的依賴關係
   - 當一個物件的變更自動改變其他物件時

## 關聯圖
DP-012 --enforces--> LAW-002（開放/封閉）
DP-012 --enforces--> LAW-005（依賴反轉）
DP-012 --enforces--> LAW-001（單一職責）

## 矛盾與權衡
- Observer（DP-012）實現鬆散耦合但可能導致非預期的
  更新連鎖 — 與 Mediator（DP-014）的集中化方式形成對比

## 後續步驟
1. 取得建議 — 將發現轉化為指導 → 使用 episteme-advisor
2. 檢查程式碼 — 在您的程式碼庫中偵測 Observer → 使用 code-reviewer
3. 評估架構 — 評估結構適配性 → 使用 architecture-analyst
```

---

### 4. `architecture-analyst`

**使用時機**：根據法則、模式和結構風險評估系統架構

**使用的工具**：`search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**提示範例：**
```
「這個微服務架構是否遵循了 Conway's Law？」
[描述架構]
```

**代理回應：**
```
# 架構分析：微服務

## 法則與原則合規性
| 原則 | 狀態 | 證據 | 影響 |
|------|------|------|------|
| Conway's Law（LAW-042） | 違反 | Shipping 跨越 2 個團隊 | 協調負擔 |
| SRP（LAW-001） | 有風險 | Analytics 依賴所有模組 | 緊密耦合 |
| Bounded Context（LAW-031） | 違反 | 無明確領域邊界 | 共用資料混淆 |

## 關鍵張力
- Conway's Law（LAW-042）要求團隊↔服務對齊
  但 Shipping 服務跨越 Commerce + Platform 團隊
- 追蹤路徑：LAW-042 → related_to → LAW-001 → enforced_by → DP-026（Strangler Fig）

## 架構建議
1. **嚴重**：將 Shipping 移至 Commerce 團隊 — LAW-042 預測協調失敗
2. **高**：為 Analytics 引入 Event Bus — 透過非同步事件解耦
3. **中**：定義 Bounded Contexts — 將服務邊界與領域對齊

## 合規分數
- 整體：5/10 | 結構：4/10 | 可擴展性：6/10 | 可維護性：5/10

## 後續步驟
1. 取得建議 — 解決關鍵張力 → 使用 episteme-advisor
2. 檢查程式碼 — 偵測結構性壞味道 → 使用 code-reviewer
3. 研究替代方案 — 尋找更好的模式 → 使用 episteme-researcher
```

---

## 工作流程鏈

代理和工具連接成端對端的處理管線。每條鏈產生一份報告，隨後提供互動的後續選項。

### 鏈 1：程式碼審查管線
```
analyze_code → suggest_refactorings → get_neighbors("solved_by")
  → find_path(smell_A, smell_B) → 包含因果圖的報告
  → 使用者選擇：套用修正 / 深入分析 / 架構檢查 / 進一步了解
```

### 鏈 2：架構審查管線
```
search_knowledge → get_entity → get_neighbors("enforces")
  → get_neighbors("violates") → find_path → 合規報告
  → 使用者選擇：重構計畫 / 諮詢建議 / 研究替代方案
```

### 鏈 3：問題診斷管線
```
search_knowledge(症狀) → get_entity → get_neighbors("solved_by")
  → 根本原因報告 → 使用者選擇：套用修正 / 諮詢建議 / 驗證
```

### 鏈 4：學習管線
```
search_knowledge(主題) → get_entity → get_neighbors("related_to")
  → 概念圖 → 使用者選擇：程式碼範例 / 套用至程式碼 / 比較
```

### 跨工具鏈結規則

每次工具呼叫自然地引導至下一步：

| 呼叫...之後 | 務必接續... |
|-------------|------------|
| `analyze_code` | 對已偵測的壞味道執行 `suggest_refactorings` |
| `suggest_refactorings` | `get_neighbors(smell_id, "solved_by")` 以尋找替代方案 |
| `search_knowledge` | 對前 1-2 個結果執行 `get_entity` |
| `get_entity`（壞味道） | `get_neighbors(id, "violates")` 以查看受影響的原則 |
| `get_entity`（模式） | `get_neighbors(id, "enforces")` 以查看強制實踐的法則 |
| 偵測到多個壞味道 | `find_path(smell_A, smell_B)` 以進行因果對應 |

---

## 其他工具的安裝

### Cursor

```bash
epis install cursor
```

將 MCP 設定新增至 `~/.cursor/mcp.json`：
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

在專案根目錄產生 `AGENTS.md`，其中包含代理定義。

### 自訂 MCP 整合

若您的工具支援 MCP，請手動設定：

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

## 以背景服務執行

為了更好的效能，將 Episteme MCP 作為持續性的 HTTP 代理執行：

```bash
# 啟動背景服務
epis service start

# 檢查狀態
epis service status
# 輸出：Running on http://localhost:43175 (PID 12345)

# 啟用開機自動啟動（macOS）
epis service enable

# 停止服務
epis service stop
```

更新 MCP 設定以使用 HTTP 代理：

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

日誌：`~/.episteme/logs/mcp.out.log`

---

## 疑難排解

### 工具未在 Claude 中顯示

1. 檢查設定檔是否存在：`cat ~/.claude/claude_desktop_config.json`
2. 驗證 episteme 是否在 PATH 中：`which episteme`
3. 直接測試 MCP：`episteme mcp`
4. 檢查日誌：`tail -f ~/.episteme/logs/mcp.err.log`

### 「Database not found」錯誤

```bash
# 重建知識資料庫
epis build --rebuild
```

### 搜尋回應緩慢

```bash
# 使用 GPU 加速
epis build --gpu

# 或以背景服務執行（更快的預熱）
epis service start
```

### 代理未使用工具

確保代理具有工具呼叫能力。在 Claude Code 中：
```
使用者：「使用 Episteme 尋找重試邏輯的模式」
      ^^^^ 明確提及工具使用
```

---

## 進階：自訂知識整合

結合 Episteme（通用知識）與 Alcove（團隊知識）：

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

參見 [Alcove 整合指南](./alcove-integration.md) 以了解雙來源模式。

---

## API 替代方案

若您的 AI 工具不支援 MCP，請使用 REST API：

```bash
# 啟動 API 伺服器
docker-compose up -d

# 從任何工具使用
curl http://localhost:8000/search?q=strategy+pattern
```

參見 [API 文件](./api.md) 以了解端點。

---

## 自動觸發（Claude Code）

當您以自然語言描述問題時，Claude Code 會自動偵測意圖並呼叫適當的 Episteme 工具 — **您不需要明確提及 Episteme**。以下是確切的觸發模式和範例。

### 運作方式

```
您的自然語言輸入
    ↓ Claude 偵測關鍵字/模式
    ↓ Episteme 工具被自動呼叫
    ↓ 知識圖譜回傳已驗證的資料
    ↓（設計模式 · 程式碼壞味道 · 重構手法 · 工程法則）
    ↓ Claude 的回應基於證據
```

> **注意：** 這是基於提示的自動觸發，而非硬性掛鉤。若要保證呼叫，請直接使用 `/episteme` 技能。

### 程式碼結構問題

| 您說的話（範例） | Episteme 偵測到的內容 | 自動工具呼叫 |
|-----------------|---------------------|-------------|
| 「這個類別做了太多事」、「這個檔案超過 300 行」 | God Class、Large Class、單一職責 | `search_knowledge("god class large class single responsibility")` |
| 「這個函數太長了」、「這個方法行數太多」 | Long Method | `search_knowledge("long method extract method")` |
| 「程式碼太複雜了」、「難以理解」 | 複雜度、認知超載 | `search_knowledge("complexity smell cognitive overload")` |
| 「我到處複製貼上」、「有重複的邏輯」 | 重複程式碼、Clone | `search_knowledge("duplicated code clone smell")` |

### 耦合與依賴問題

| 您說的話（範例） | Episteme 偵測到的內容 | 自動工具呼叫 |
|-----------------|---------------------|-------------|
| 「業務邏輯直接呼叫資料庫」 | 耦合、持續性、Repository | `search_knowledge("coupling persistence repository data access layer")` |
| 「改了 X 就弄壞 Y」、「變動到處擴散」 | 脆弱耦合、變更傳播 | `search_knowledge("brittle coupling change propagation rigidity")` |
| 「新增類型需要到處修改」、「switch-case 一直在增長」 | 開放/封閉、Strategy、多型 | `search_knowledge("open closed principle strategy polymorphism")` |

### 測試與品質問題

| 您說的話（範例） | Episteme 偵測到的內容 | 自動工具呼叫 |
|-----------------|---------------------|-------------|
| 「這很難測試」、「無法為此撰寫單元測試」 | 可測試性、依賴注入 | `search_knowledge("testability dependency injection mockability")` |

### 效能與並行問題

| 您說的話（範例） | Episteme 偵測到的內容 | 自動工具呼叫 |
|-----------------|---------------------|-------------|
| 「API 很慢」、「回應時間太高」 | N+1 查詢、延遲載入、快取 | `search_knowledge("N+1 query lazy loading caching performance")` |
| 「這是執行緒安全的嗎？」、「有任何並行問題嗎？」 | 執行緒安全、競態條件 | `search_knowledge("thread safety race condition shared mutable state")` |

### 程式碼審查與分析

| 您說的話（範例） | 自動工具呼叫 |
|-----------------|-------------|
| 分享程式碼片段並要求審查 | `analyze_code(code)` → `suggest_refactorings(code)` |
| 「改善這段程式碼」、「幫我重構」 | `suggest_refactorings(code)` |

### 架構討論

| 您說的話（範例） | 自動工具呼叫 |
|-----------------|-------------|
| 「微服務 vs 單體」、「我們應該怎麼拆分？」 | `search_knowledge("monolith microservice decomposition bounded context")` |
| 「這個架構可以嗎？」、「審查這個設計」 | `search_knowledge("layered architecture coupling responsibility")` |

### 真實對話範例

**範例 1 — 偵測 God Object：**

```
您：「UserService 越來越失控了。它處理身份驗證、
     工作階段管理、電子郵件驗證和密碼重設。」

Claude（自動呼叫 search_knowledge）：
  → 偵測到：God Object / 單一職責違規
  → 回傳：SMELL-03（God Object）、LAW-001（SRP）、RF-018（Extract Class）

Claude：「UserService 顯示 God Object 特徵（SMELL-03）。
  它違反了單一職責原則（LAW-001）。

  建議的重構手法：
  1. Extract Class（RF-018）— 為工作階段邏輯建立 SessionManager
  2. Move Method（RF-023）— 將電子郵件驗證移至 EmailService
  3. Extract Interface（RF-042）— 定義 AuthService 介面

  從 #2 開始獲得快速勝利，然後 #1 進行結構性改善。」
```

**範例 2 — 效能調查：**

```
您：「產品列表 API 真的很慢。單獨載入沒問題
     但列表頁面需要 5 秒以上。」

Claude（自動呼叫 search_knowledge）：
  → 偵測到：N+1 查詢問題
  → 回傳：效能模式、延遲載入、快取策略

Claude：「這聽起來像是 N+1 查詢問題 — 每個產品觸發
  個別的資料庫呼叫，而非批次查詢。

  來自知識圖譜的解決方案：
  1. 批次載入/預先載入 — 在一次查詢中取得關聯資料
  2. Data Mapper（DP-019）— 將查詢邏輯與領域物件解耦
  3. 套用延遲載入（RF-050）— 僅在需要時載入關聯

  您希望我檢查程式碼中的 N+1 模式嗎？」
```

**範例 3 — 程式碼片段審查：**

```
您：[貼上 Python 程式碼]
    「可以幫我審查嗎？感覺哪裡不對。」

Claude（自動呼叫 analyze_code → suggest_refactorings）：
  → analyze_code 偵測到：Long Method（信心度 0.87）、Long Parameter List（0.92）
  → suggest_refactorings 回傳排序的建議

Claude：「發現 2 個程式碼壞味道：

  1. Long Method（SMELL-01）— 45 行，迴圈複雜度 12
     → Extract Method（RF-001），優先級 0.89

  2. Long Parameter List（SMELL-08）— 9 個參數
     → Introduce Parameter Object（RF-029），優先級 0.92

  從 RF-029（Parameter Object）開始 — 這是最高優先級
  且能讓後續的 Extract Method 更容易。」
```

---

## 後續步驟

1. **試用代理**：詢問 episteme-advisor「我應該使用 Singleton 嗎？」
2. **分析程式碼**：貼上一個函數並請 code-reviewer 檢查壞味道
3. **探索圖譜**：使用 episteme-researcher 尋找模式關聯
4. **自訂工作流程**：組合工具（analyze → suggest → search）

更多範例，請參見：
- [Alcove 整合](./alcove-integration.md) — 團隊知識 + Episteme
- [API 參考](./api.md) — REST 端點
