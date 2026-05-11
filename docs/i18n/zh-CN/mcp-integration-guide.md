# MCP集成指南

> 将Episteme的知识图谱集成到Claude Code、Cursor和其他兼容MCP的AI工具中

## Rust MCP HTTP模式（当前）
直接使用独立的HTTP传输:

```bash
# 通过HTTP启动MCP
episteme mcp --http --host 127.0.0.1 --port 43175
```

认证行为:
- 如果配置了`EPISTEME_API_KEYS`，请求必须包含:
```http
Authorization: Bearer <api-key>
```
- 如果未配置密钥，则跳过认证（开发模式）。
- `GET /health`始终公开，用于健康检查。

注意:
- `epis service`在后台管理相同的MCP HTTP模式（`start|stop|status|enable|disable`）。
- 旧的`--proxy`示例已弃用；请直接使用`mcp --http`/`service`。

## 什么是MCP？

[模型上下文协议（MCP）](https://modelcontextprotocol.io)是一种开放标准，允许AI助手访问外部工具和数据源。Episteme提供6个MCP工具，让AI代理直接访问软件工程知识。

---

## 快速入门（Claude Code）

### 1. 安装Episteme

```bash
# 安装（需要Rust 1.95及以上版本）
cargo install --git https://github.com/epicsagas/Episteme

# 将代理和MCP服务器安装到Claude Code
# （自动填充数据并配置MCP）
epis install claude
```

> 如果数据下载失败，请使用源码安装: `git clone` → `cargo build --release` → `epis install --local`

### 2. 验证安装

检查`~/.claude/claude_desktop_config.json`:

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

### 3. 开始使用

重启Claude Code。你现在可以访问6个Episteme工具:

```
用户: "修复God Object异味的最佳方法是什么？"

Claude（使用search_knowledge工具）:
  → 搜索"God Object"的重构方案
  → 返回: RF-018 (Extract Class)、RF-023 (Move Method)

Claude: "God Object反模式（SMELL-03）违反了单一职责原则（LAW-001）。
最佳重构方案:

1. Extract Class（RF-018） - 将相关方法/字段移到新类
2. Move Method（RF-023） - 将方法重新定位到合适的类

两者都强制执行SOLID原则并提高可测试性。"
```

---

## MCP工具参考

### 1. `search_knowledge`

**用途**: 跨所有实体（模式、法则、重构、异味）的语义搜索

**参数:**
```typescript
{
  query: string          // 自然语言查询
  top_k?: number         // 返回结果数（默认: 5）
  filter_type?: string   // "pattern"、"law"、"refactoring"、"smell"
}
```

**返回:**
```typescript
{
  results: [{
    entity_id: string     // 例如: "DP-023"
    title: string
    entity_type: string
    similarity: number    // 0.0-1.0
    summary: string
  }]
}
```

**对话示例:**
```
用户: "如何让我的代码更具可测试性？"

Claude调用: search_knowledge({
  query: "improve testability",
  top_k: 3
})

返回:
- LAW-001: 单一职责原则
- DP-018: 依赖注入
- RF-042: 提取接口

Claude: "提高可测试性的三个关键方法:
1. 应用SRP（LAW-001） - 一个类，一个变更原因
2. 使用依赖注入（DP-023） - 注入依赖
3. 提取接口（RF-042） - 模拟外部依赖"
```

---

### 2. `get_entity`

**用途**: 按ID获取特定实体的完整详情

**参数:**
```typescript
{
  entity_id: string   // 例如: "DP-023"、"RF-001"、"SMELL-01"
}
```

**返回:**
```typescript
{
  entity_id: string
  title: string
  type: string
  description: string
  implementation: string    // 代码示例
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

**对话示例:**
```
用户: "详细解释一下策略模式"

Claude调用: get_entity({ entity_id: "DP-023" })

返回:
- 策略模式的完整描述
- 使用场景（多种算法、运行时切换）
- 实现（Context、Strategy接口、ConcreteStrategies）
- 优点: 开闭原则、运行时灵活性
- 权衡: 增加类数量
- 相关: LAW-002（开闭原则）、RF-034（用多态替换条件语句）

Claude: "策略模式（DP-023）将算法封装在独立的类中...
[提供带代码示例的详细解释]

相关概念:
- 强制执行开闭原则（LAW-002）
- 通常通过用多态替换条件语句（RF-034）应用"
```

---

### 3. `get_neighbors`

**用途**: 探索与给定实体相关的实体

**参数:**
```typescript
{
  entity_id: string
  relation_type?: string   // "solves"、"enforces"、"violates"、"relates_to"
  max_depth?: number       // 默认: 1
}
```

**返回:**
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

**对话示例:**
```
用户: "哪些重构可以解决Long Method异味？"

Claude调用: get_neighbors({
  entity_id: "SMELL-01",
  relation_type: "solved_by"
})

返回:
- RF-001: Extract Method
- RF-034: Decompose Conditional
- RF-056: Replace Temp with Query

Claude: "Long Method（SMELL-01）可以通过以下方式解决:
1. Extract Method（RF-001） - 拆分为更小的方法
2. Decompose Conditional（RF-034） - 简化复杂条件
3. Replace Temp with Query（RF-056） - 消除临时变量"
```

---

### 4. `find_path`

**用途**: 查找两个实体之间的连接（例如，法则如何与模式相关）

**参数:**
```typescript
{
  from_id: string
  to_id: string
  max_depth?: number   // 默认: 5
}
```

**返回:**
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

**对话示例:**
```
用户: "单一职责原则与Extract Class重构有什么关系？"

Claude调用: find_path({
  from_id: "LAW-001",
  to_id: "RF-018"
})

返回路径:
LAW-001 (SRP) --enforces--> DP-007 (Facade) --achieved_via--> RF-018 (Extract Class)

Claude: "SRP（LAW-001）通过2跳连接到Extract Class（RF-018）:
1. SRP强制执行外观模式（DP-007）
2. 外观模式通常通过Extract Class（RF-018）实现

洞察: 当一个类有多个职责时，
提取类（RF-018）有助于满足SRP（LAW-001）。"
```

---

### 5. `analyze_code`

**用途**: 通过AST分析检测代码异味

**参数:**
```typescript
{
  code: string
  language: string   // "python"、"java"、"typescript"等
  min_confidence?: number   // 默认: 0.5
}
```

**返回:**
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

**对话示例:**
```
用户: "审查这段支付验证代码"
[粘贴代码]

Claude调用: analyze_code({
  code: "...",
  language: "python"
})

返回:
- SMELL-01: Long Method（置信度 0.87，LOC=45）
- SMELL-08: Long Parameter List（置信度 0.92，params=9）

Claude: "发现了2个代码异味:

1. Long Method（SMELL-01，置信度 87%）
   - 45行代码（阈值: 20）
   - 高圈复杂度（12）
   - 建议: Extract Method（RF-001）

2. Long Parameter List（SMELL-08，置信度 92%）
   - 9个参数（阈值: 4）
   - 建议: Introduce Parameter Object（RF-029）"
```

---

### 6. `suggest_refactorings`

**用途**: 获取已检测异味的排名重构建议

**参数:**
```typescript
{
  code: string
  language: string
  top_k?: number   // 建议数量（默认: 3）
  min_confidence?: number
}
```

**返回:**
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

**对话示例:**
```
用户: "这段代码应该重构哪里？"
[粘贴大型类]

Claude调用: suggest_refactorings({
  code: "...",
  language: "java",
  top_k: 3
})

返回God Object异味的排名建议:
1. RF-018: Extract Class（优先级 0.89，严重性 High，工作量 Medium）
2. RF-023: Move Method（优先级 0.76，严重性 Medium，工作量 Low）
3. RF-042: Extract Interface（优先级 0.65，严重性 Low，工作量 Low）

Claude: "这段代码显示了God Object（SMELL-03）的特征。

重构优先级:

1. Extract Class（RF-018） - 优先级 0.89
   - 严重性: High
   - 工作量: Medium
   - 将支付相关方法移到PaymentProcessor类
   - 强制执行: SRP（LAW-001）、SoC（LAW-012）

2. Move Method（RF-023） - 优先级 0.76
   - 严重性: Medium
   - 工作量: Low
   - 将验证方法重新定位到Validator类
   - 提高内聚性的快速改进

先从#2（Move Method）开始立即改善，
然后解决#1（Extract Class）进行长期设计。"
```

---

## AI代理

Episteme包含4个专用代理，它们作为互联系统协同工作。每个代理使用知识图谱工具进行分析，并提供可连接到其他代理的交互式后续选项。

### 代理网络

```
code-reviewer ←→ episteme-advisor
      ↕                ↕
architecture-analyst ←→ episteme-researcher
      ↕
refactoring-expert（外部）
```

每个代理在报告末尾都会提供**下一步** — 连接到其他代理或工具的交互式选项。这创建了从检测到修复的端到端工作流。

---

### 1. `code-reviewer`

**使用场景**: 审查代码的异味、SOLID违规和重构优先级

**独特之处**: 不仅限于检测 — 追踪异味之间的因果关系链（哪些是根本原因，哪些是下游症状），针对语言生态惯例（Rust、Go等）验证发现，并根据上下文校准严重性。

**使用工具**: `analyze_code` → `suggest_refactorings` → `get_entity` → `get_neighbors` → `find_path`

**提示示例:**
```
"审查这个认证模块"
[粘贴代码]
```

**代理响应:**
```
# Episteme代码审查

## 摘要
AuthenticationModule有一个作为根本原因的God Object异味，
导致3个下游问题。修复根本原因将产生级联改善。

## 异味检测
| 异味 | 位置 | 严重性 | 置信度 | 相关原则 |
|-------|----------|----------|------------|-------------------|
| God Object (SMELL-03) | auth.rs:1-420 | Critical | confirmed | LAW-001 (SRP) |
| Long Method (SMELL-01) | auth.rs:88-142 | High | confirmed | LAW-001 (SRP) |
| Divergent Change (SMELL-09) | auth.rs:200-380 | Medium | confirmed | LAW-012 (SoC) |
| Feature Envy (SMELL-15) | auth.rs:290-333 | Medium | confirmed | LAW-003 (LSP) |

## 因果分析
根本原因:
- SMELL-03 → 导致 → SMELL-01, SMELL-09
  （修复God Object将产生级联改善）

独立的:
- SMELL-15 — 独立存在，无下游影响

## 排名重构
1. **Extract Class** (RF-018) — 优先级: Critical
   - 解决: SMELL-03（根本原因）
   - 级联效果: 同时解决SMELL-01、SMELL-09
   - 执行: 提取SessionManager、EmailVerifier类

## 下一步
1. 重构 — 应用RF-018（Extract Class） → 使用refactoring-expert代理
2. 深入分析 — 探索SMELL-03的根本原因链 → 使用episteme-advisor
3. 架构检查 — 评估结构性影响 → 使用architecture-analyst
4. 了解更多 — 获取SMELL-03的完整说明 → get_entity(detail_level="full")
```

**验证**: 在确认异味之前，代理会与语言惯用法（例如: Rust中大结构体是正常的）、领域上下文（测试文件、生成代码）和实际影响（表面问题还是阻碍开发）进行交叉检查。

---

### 2. `episteme-advisor`

**使用场景**: 进行工程决策（模式选择、架构权衡、根本原因深入分析）

**使用工具**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**提示示例:**
```
"我的ConfigManager类应该用Singleton吗？"
```

**代理响应:**
```
# 咨询: ConfigManager的Singleton vs DI

## 适用实体
### 1. 单例模式 (DP-006) - 主要
- **说明**: 确保类只有一个实例
- **对你的意义**: 全局配置访问但存在隐式耦合
- **建议**: 避免 — 违反可测试性

### 2. 依赖注入 (DP-018) - 替代
- **说明**: 通过构造函数注入依赖
- **对你的意义**: 显式依赖，易于模拟
- **建议**: ConfigManager推荐使用此方案

## 权衡
- Singleton（DP-006）提供便利但违反DI（DP-018）
- DI增加样板代码但实现LAW-005（DIP）合规
- 解决方案: 使用带有配置提供者接口的DI

## 行动计划
1. 立即: 通过构造函数注入ConfigManager
2. 短期: 提取ConfigProvider接口
3. 长期: 考虑环境特定的配置策略

## 下一步
1. 实现 — 应用DI模式 → 使用code-reviewer验证
2. 探索替代方案 — 比较方案 → 使用episteme-researcher
3. 架构影响 — 评估下游影响 → 使用architecture-analyst
```

---

### 3. `episteme-researcher`

**使用场景**: 探索知识图谱中的关系、寻找替代方案

**使用工具**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**提示示例:**
```
"观察者模式强制执行哪些法则？"
```

**代理响应:**
```
# 研究报告: 观察者模式强制执行的法则

## 主要实体
1. **观察者模式** (DP-012) - 行为型
   - 定义: 在对象之间定义一对多的依赖关系
   - 当一个对象的变更自动影响其他对象时

## 连接图
DP-012 --enforces--> LAW-002 (开闭原则)
DP-012 --enforces--> LAW-005 (依赖倒置原则)
DP-012 --enforces--> LAW-001 (单一职责原则)

## 矛盾与权衡
- 观察者（DP-012）实现松耦合但可能导致意外的
  更新链 — 与集中化的中介者（DP-014）形成对比

## 下一步
1. 获取建议 — 将发现转化为指导 → 使用episteme-advisor
2. 检查代码 — 在代码库中检测观察者 → 使用code-reviewer
3. 评估架构 — 评估结构适用性 → 使用architecture-analyst
```

---

### 4. `architecture-analyst`

**使用场景**: 根据法则、模式和结构风险评估系统架构

**使用工具**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**提示示例:**
```
"这个微服务架构是否遵循康威定律？"
[描述架构]
```

**代理响应:**
```
# 架构分析: 微服务

## 法则与原则合规性
| 原则 | 状态 | 证据 | 影响 |
|-----------|--------|----------|--------|
| 康威定律 (LAW-042) | 违反 | 配送跨2个团队 | 协调开销 |
| SRP (LAW-001) | 有风险 | Analytics依赖所有服务 | 紧耦合 |
| 限界上下文 (LAW-031) | 违反 | 无明确领域边界 | 数据共享混乱 |

## 关键矛盾
- 康威定律（LAW-042）要求团队↔服务对齐
  但Shipping服务跨越Commerce + Platform团队
- 追踪路径: LAW-042 → related_to → LAW-001 → enforced_by → DP-026 (Strangler Fig)

## 架构建议
1. **Critical**: 将Shipping移至Commerce团队 — LAW-042预测协调失败
2. **High**: 为Analytics引入事件总线 — 通过异步事件解耦
3. **Medium**: 定义限界上下文 — 将服务边界与领域对齐

## 合规评分
- 总体: 5/10 | 结构: 4/10 | 可扩展性: 6/10 | 可维护性: 5/10

## 下一步
1. 获取建议 — 解决关键矛盾 → 使用episteme-advisor
2. 检查代码 — 检测结构性异味 → 使用code-reviewer
3. 研究替代方案 — 寻找更好的模式 → 使用episteme-researcher
```

---

## 工作流链

代理和工具连接成端到端管道。每个链产生一个报告，随后提供交互式后续选项。

### 链1: 代码审查管道
```
analyze_code → suggest_refactorings → get_neighbors("solved_by")
  → find_path(smell_A, smell_B) → 带因果关系图的报告
  → 用户选择: 应用修复 / 深入分析 / 架构检查 / 了解更多
```

### 链2: 架构审查管道
```
search_knowledge → get_entity → get_neighbors("enforces")
  → get_neighbors("violates") → find_path → 合规报告
  → 用户选择: 重构计划 / 咨询 / 研究替代方案
```

### 链3: 问题诊断管道
```
search_knowledge(症状) → get_entity → get_neighbors("solved_by")
  → 根本原因报告 → 用户选择: 应用修复 / 咨询 / 验证
```

### 链4: 学习管道
```
search_knowledge(主题) → get_entity → get_neighbors("related_to")
  → 概念图 → 用户选择: 代码示例 / 应用到代码 / 比较
```

### 跨工具链规则

每个工具调用自然引向下一个:

| 调用后... | 始终跟进... |
|-------------------|--------------------------|
| `analyze_code` | 对检测到的异味执行`suggest_refactorings` |
| `suggest_refactorings` | 使用`get_neighbors(smell_id, "solved_by")`查找替代方案 |
| `search_knowledge` | 对前1-2个结果执行`get_entity` |
| `get_entity`（异味） | 使用`get_neighbors(id, "violates")`查找受影响的原则 |
| `get_entity`（模式） | 使用`get_neighbors(id, "enforces")`查找强制执行的法则 |
| 检测到多个异味 | 使用`find_path(smell_A, smell_B)`进行因果关系映射 |

---

## 其他工具的安装

### Cursor

```bash
epis install cursor
```

添加MCP配置到`~/.cursor/mcp.json`:
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

在项目根目录生成带有代理定义的`AGENTS.md`。

### 自定义MCP集成

如果你的工具支持MCP，手动配置:

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

## 作为后台服务运行

为了更好的性能，将Episteme MCP作为持久的HTTP代理运行:

```bash
# 启动后台服务
epis service start

# 检查状态
epis service status
# 输出: Running on http://localhost:43175 (PID 12345)

# 启用开机自启（macOS）
epis service enable

# 停止服务
epis service stop
```

更新MCP配置以使用HTTP代理:

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

日志: `~/.episteme/logs/mcp.out.log`

---

## 故障排除

### 工具未在Claude中显示

1. 检查配置文件是否存在: `cat ~/.claude/claude_desktop_config.json`
2. 验证episteme在PATH中: `which episteme`
3. 直接测试MCP: `episteme mcp`
4. 检查日志: `tail -f ~/.episteme/logs/mcp.err.log`

### "Database not found"错误

```bash
# 重建知识数据库
epis build --rebuild
```

### 搜索响应缓慢

```bash
# 使用GPU加速
epis build --gpu

# 或作为后台服务运行（更快的预热）
epis service start
```

### 代理未使用工具

确保代理具有工具调用能力。在Claude Code中:
```
用户: "使用Episteme查找重试逻辑的模式"
              ^^^^ 明确提及工具使用
```

---

## 高级: 自定义知识集成

将Episteme（通用知识）与Alcove（团队知识）结合:

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

参见[Alcove集成指南](./alcove-integration.md)了解双源模式。

---

## API替代方案

如果你的AI工具不支持MCP，请使用REST API:

```bash
# 启动API服务器
docker-compose up -d

# 从任何工具使用
curl http://localhost:8000/search?q=strategy+pattern
```

参见[API文档](./api.md)了解端点信息。

---

## 自动触发（Claude Code）

当你用自然语言描述问题时，Claude Code会自动检测意图并调用适当的Episteme工具 — **你无需明确提及Episteme**。以下是精确的触发模式和示例。

### 工作原理

```
你的自然语言输入
    ↓ Claude检测关键词/模式
    ↓ 自动调用Episteme工具
    ↓ 知识图谱返回验证数据
    ↓ (设计模式 · 代码异味 · 重构技术 · 工程法则)
    ↓ Claude的回答基于证据
```

> **注意:** 这是基于提示的自动触发，而非硬钩子。要保证调用，请直接使用`/episteme`技能。

### 代码结构问题

| 你说的话（示例） | Episteme检测到的内容 | 自动工具调用 |
|-------------------------|-----------------------|---------------------|
| "这个类做的事情太多了"、"这个文件超过300行" | God Class、Large Class、单一职责 | `search_knowledge("god class large class single responsibility")` |
| "这个函数太长了"、"这个方法行数太多" | Long Method | `search_knowledge("long method extract method")` |
| "代码太复杂了"、"很难理解" | 复杂性、认知过载 | `search_knowledge("complexity smell cognitive overload")` |
| "我到处复制粘贴了"、"有重复的逻辑" | 重复代码、克隆 | `search_knowledge("duplicated code clone smell")` |

### 耦合与依赖问题

| 你说的话（示例） | Episteme检测到的内容 | 自动工具调用 |
|-------------------------|-----------------------|---------------------|
| "业务逻辑直接调用数据库" | 耦合、持久化、仓储 | `search_knowledge("coupling persistence repository data access layer")` |
| "修改X会破坏Y"、"变更到处波及" | 脆弱耦合、变更传播 | `search_knowledge("brittle coupling change propagation rigidity")` |
| "添加新类型意味着到处都要改"、"switch-case不断增长" | 开闭原则、策略、多态 | `search_knowledge("open closed principle strategy polymorphism")` |

### 测试与质量问题

| 你说的话（示例） | Episteme检测到的内容 | 自动工具调用 |
|-------------------------|-----------------------|---------------------|
| "这很难测试"、"无法为这个写单元测试" | 可测试性、依赖注入 | `search_knowledge("testability dependency injection mockability")` |

### 性能与并发问题

| 你说的话（示例） | Episteme检测到的内容 | 自动工具调用 |
|-------------------------|-----------------------|---------------------|
| "API很慢"、"响应时间太高" | N+1查询、懒加载、缓存 | `search_knowledge("N+1 query lazy loading caching performance")` |
| "这是线程安全的吗？"、"有并发问题吗？" | 线程安全、竞态条件 | `search_knowledge("thread safety race condition shared mutable state")` |

### 代码审查与分析

| 你说的话（示例） | 自动工具调用 |
|-------------------------|---------------------|
| 分享代码片段并请求审查 | `analyze_code(code)` → `suggest_refactorings(code)` |
| "改进这段代码"、"帮我重构这个" | `suggest_refactorings(code)` |

### 架构讨论

| 你说的话（示例） | 自动工具调用 |
|-------------------------|---------------------|
| "微服务还是单体"、"我们应该怎么拆分？" | `search_knowledge("monolith microservice decomposition bounded context")` |
| "这个架构可以吗？"、"审查这个设计" | `search_knowledge("layered architecture coupling responsibility")` |

### 真实对话示例

**示例1 — 检测God Object:**

```
你: "UserService越来越失控了。它处理认证、
     会话管理、邮件验证和密码重置。"

Claude（自动调用search_knowledge）:
  → 检测到: God Object / 单一职责违规
  → 返回: SMELL-03 (God Object), LAW-001 (SRP), RF-018 (Extract Class)

Claude: "UserService显示了God Object特征（SMELL-03）。
  它违反了单一职责原则（LAW-001）。

  推荐的重构:
  1. Extract Class（RF-018） — 为会话逻辑创建SessionManager
  2. Move Method（RF-023） — 将邮件验证移到EmailService
  3. Extract Interface（RF-042） — 定义AuthService接口

  先从#2开始快速改善，然后#1进行结构性改进。"
```

**示例2 — 性能调查:**

```
你: "产品列表API真的很慢。单独加载没问题
     但列表页面要5秒以上。"

Claude（自动调用search_knowledge）:
  → 检测到: N+1查询问题
  → 返回: 性能模式、懒加载、缓存策略

Claude: "这听起来像是N+1查询问题 — 每个产品触发
  单独的数据库调用而不是批量查询。

  知识图谱中的解决方案:
  1. 批量加载 / 预加载 — 一次查询获取相关数据
  2. 数据映射器（DP-019） — 将查询逻辑与领域对象解耦
  3. 应用懒加载（RF-050） — 仅按需加载关系

  需要我检查代码中的N+1模式吗？"
```

**示例3 — 带代码片段的审查:**

```
你: [粘贴Python代码]
    "你能审查一下这个吗？感觉有点不对劲。"

Claude（自动调用analyze_code → suggest_refactorings）:
  → analyze_code检测到: Long Method（置信度 0.87）、Long Parameter List（0.92）
  → suggest_refactorings返回排名建议

Claude: "发现了2个代码异味:

  1. Long Method（SMELL-01） — 45行，圈复杂度 12
     → Extract Method（RF-001），优先级 0.89

  2. Long Parameter List（SMELL-08） — 9个参数
     → Introduce Parameter Object（RF-029），优先级 0.92

  先从RF-029（Parameter Object）开始 — 它的优先级最高，
  而且会使后续的Extract Method更容易。"
```

---

## 下一步

1. **试用代理**: 向episteme-advisor询问"我应该用Singleton吗？"
2. **分析代码**: 粘贴一个函数并让code-reviewer检查异味
3. **探索图谱**: 使用episteme-researcher查找模式关系
4. **自定义工作流**: 组合工具（analyze → suggest → search）

更多示例请参见:
- [Alcove集成](./alcove-integration.md) — 团队知识 + Episteme
- [监控设置](../../monitoring/README.md) — 追踪模式使用情况
- [API参考](./api.md) — REST端点
