# MCP एकीकरण मार्गदर्शिका

> Episteme के ज्ञान ग्राफ को Claude Code, Cursor और अन्य MCP-संगत AI टूल्स में एकीकृत करें

## Rust MCP HTTP मोड (वर्तमान)
स्टैंडअलोन HTTP ट्रांसपोर्ट सीधे उपयोग करें:

```bash
# HTTP पर MCP प्रारंभ करें
episteme mcp --http --host 127.0.0.1 --port 43175
```

प्रमाणीकरण व्यवहार:
- यदि `EPISTEME_API_KEYS` कॉन्फ़िगर किया गया है, तो अनुरोधों में यह शामिल होना चाहिए:
```http
Authorization: Bearer <api-key>
```
- यदि कोई कुंजियाँ कॉन्फ़िगर नहीं हैं, तो प्रमाणीकरण छोड़ दिया जाता है (विकास मोड)।
- `GET /health` हेल्थ चेक के लिए हमेशा सार्वजनिक है।

नोट:
- `epis service` बैकग्राउंड में इसी MCP HTTP मोड का प्रबंधन करता है (`start|stop|status|enable|disable`)।
- पुराने `--proxy` उदाहरण अप्रचलित हैं; `mcp --http`/`service` सीधे उपयोग करें।

## MCP क्या है?

[मॉडल कॉन्टेक्स्ट प्रोटोकॉल (MCP)](https://modelcontextprotocol.io) एक खुला मानक है जो AI सहायकों को बाहरी टूल्स और डेटा स्रोतों तक पहुँचने की अनुमति देता है। Episteme 6 MCP टूल्स प्रदान करता है जो AI एजेंटों को सॉफ़्टवेयर इंजीनियरिंग ज्ञान तक सीधी पहुँच देते हैं।

---

## त्वरित आरंभ (Claude Code)

### 1. Episteme स्थापित करें

```bash
# स्थापना (Rust 1.95+ आवश्यक)
cargo install --git https://github.com/epicsagas/Episteme

# एजेंट और MCP सर्वर Claude Code में स्थापित करें
# (डेटा डाउनलोड करता है और MCP स्वचालित रूप से कॉन्फ़िगर करता है)
epis install claude
```

> यदि डेटा डाउनलोड विफल होता है, तो स्रोत स्थापना उपयोग करें: `git clone` → `cargo build --release` → `epis install --local`

### 2. स्थापना सत्यापित करें

`~/.claude/claude_desktop_config.json` जाँचें:

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

### 3. उपयोग शुरू करें

Claude Code पुनः आरंभ करें। अब आपके पास 6 Episteme टूल्स तक पहुँच है:

```
उपयोगकर्ता: "God Object स्मेल को ठीक करने का सबसे अच्छा तरीका क्या है?"

Claude (search_knowledge टूल का उपयोग करते हुए):
  → "God Object" रिफैक्टरिंग खोजता है
  → लौटाता है: RF-018 (Extract Class), RF-023 (Move Method)

Claude: "God Object एंटी-पैटर्न (SMELL-03) Single Responsibility Principle
(LAW-001) का उल्लंघन करता है। सर्वोत्तम रिफैक्टरिंग:

1. Extract Class (RF-018) - संबंधित विधियों/फ़ील्ड्स को नई क्लास में ले जाएँ
2. Move Method (RF-023) - विधियों को उपयुक्त क्लास में पुनर्स्थापित करें

दोनों SOLID सिद्धांतों को लागू करते हैं और टेस्टेबिलिटी में सुधार करते हैं।"
```

---

## MCP टूल्स संदर्भ

### 1. `search_knowledge`

**उद्देश्य**: सभी एंटिटीज़ (पैटर्न, नियम, रिफैक्टरिंग, स्मेल) में सिमेंटिक खोज

**पैरामीटर**:
```typescript
{
  query: string          // प्राकृतिक भाषा क्वेरी
  top_k?: number         // लौटाने के लिए परिणाम (डिफ़ॉल्ट: 5)
  filter_type?: string   // "pattern", "law", "refactoring", "smell"
}
```

**लौटाता है**:
```typescript
{
  results: [{
    entity_id: string     // उदा. "DP-023"
    title: string
    entity_type: string
    similarity: number    // 0.0-1.0
    summary: string
  }]
}
```

---

### 2. `get_entity`

**उद्देश्य**: ID द्वारा विशिष्ट एंटिटी का पूर्ण विवरण प्राप्त करें

**पैरामीटर**:
```typescript
{
  entity_id: string   // उदा. "DP-023", "RF-001", "SMELL-01"
}
```

**लौटाता है**:
```typescript
{
  entity_id: string
  title: string
  type: string
  description: string
  implementation: string    // कोड उदाहरण
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

---

### 3. `get_neighbors`

**उद्देश्य**: किसी एंटिटी से जुड़ी एंटिटीज़ का अन्वेषण करें

**पैरामीटर**:
```typescript
{
  entity_id: string
  relation_type?: string   // "solves", "enforces", "violates", "relates_to"
  max_depth?: number       // डिफ़ॉल्ट: 1
}
```

**लौटाता है**:
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

---

### 4. `find_path`

**उद्देश्य**: दो एंटिटीज़ के बीच कनेक्शन खोजें (उदा. एक नियम पैटर्न से कैसे संबंधित है)

**पैरामीटर**:
```typescript
{
  from_id: string
  to_id: string
  max_depth?: number   // डिफ़ॉल्ट: 5
}
```

**लौटाता है**:
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

---

### 5. `analyze_code`

**उद्देश्य**: AST विश्लेषण के माध्यम से कोड स्मेल पहचानें

**पैरामीटर**:
```typescript
{
  code: string
  language: string   // "python", "java", "typescript" आदि
  min_confidence?: number   // डिफ़ॉल्ट: 0.5
}
```

**लौटाता है**:
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

---

### 6. `suggest_refactorings`

**उद्देश्य**: पहचाने गए स्मेल के लिए रैंक किए गए रिफैक्टरिंग सुझाव प्राप्त करें

**पैरामीटर**:
```typescript
{
  code: string
  language: string
  top_k?: number   // सुझावों की संख्या (डिफ़ॉल्ट: 3)
  min_confidence?: number
}
```

**लौटाता है**:
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

---

## AI एजेंट

Episteme में 4 विशेषज्ञ एजेंट शामिल हैं जो एक कनेक्टेड सिस्टम के रूप में एक साथ काम करते हैं। प्रत्येक एजेंट विश्लेषण के लिए ज्ञान ग्राफ टूल्स का उपयोग करता है और इंटरैक्टिव फ़ॉलो-अप विकल्प प्रस्तुत करता है जो अन्य एजेंटों को हस्तांतरित किए जा सकते हैं।

### एजेंट नेटवर्क

```
code-reviewer ←→ episteme-advisor
      ↕                ↕
architecture-analyst ←→ episteme-researcher
      ↕
refactoring-expert (बाहरी)
```

प्रत्येक एजेंट अपनी रिपोर्ट **अगले चरणों** के साथ समाप्त करता है — इंटरैक्टिव विकल्प जो अन्य एजेंटों या टूल्स से जुड़ते हैं। यह पहचान से सुधार तक end-to-end वर्कफ़्लो बनाता है।

---

### 1. `code-reviewer`

**उपयोग करें जब**: स्मेल, SOLID उल्लंघन और रिफैक्टरिंग प्राथमिकताओं के लिए कोड की समीक्षा करनी हो

**अलग क्या करता है**: केवल पहचान से आगे बढ़ता है — स्मेल के बीच कारण-श्रृंखलाओं को ट्रैक करता है (कौन से स्मेल मूल कारण बनाम डाउनस्ट्रीम लक्षण हैं), भाषा इकोसिस्टम परंपराओं (Rust, Go, आदि) के विरुद्ध निष्कर्षों को मान्य करता है, और संदर्भ के अनुसार गंभीरता को कैलिब्रेट करता है।

**उपयोग किए गए टूल्स**: `analyze_code` → `suggest_refactorings` → `get_entity` → `get_neighbors` → `find_path`

---

### 2. `episteme-advisor`

**उपयोग करें जब**: इंजीनियरिंग निर्णय लेने हों (पैटर्न चयन, आर्किटेक्चर ट्रेड-ऑफ़, मूल-कारण गहन विश्लेषण)

**उपयोग किए गए टूल्स**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

---

### 3. `episteme-researcher`

**उपयोग करें जब**: ज्ञान ग्राफ में संबंधों का अन्वेषण करना हो, विकल्प खोजने हों

**उपयोग किए गए टूल्स**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

---

### 4. `architecture-analyst`

**उपयोग करें जब**: नियमों, पैटर्न और संरचनात्मक जोखिमों के विरुद्ध सिस्टम आर्किटेक्चर का मूल्यांकन करना हो

**उपयोग किए गए टूल्स**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

---

## वर्कफ़्लो श्रृंखलाएँ

एजेंट और टूल्स end-to-end पाइपलाइनों में जुड़ते हैं। प्रत्येक श्रृंखला एक रिपोर्ट उत्पन्न करती है जिसके बाद इंटरैक्टिव फ़ॉलो-अप विकल्प होते हैं।

### श्रृंखला 1: कोड समीक्षा पाइपलाइन
```
analyze_code → suggest_refactorings → get_neighbors("solved_by")
  → find_path(smell_A, smell_B) → कारण ग्राफ के साथ रिपोर्ट
  → उपयोगकर्ता चुनता है: फ़िक्स लागू करें / गहराई / आर्किटेक्चर चेक / और जानें
```

### श्रृंखला 2: आर्किटेक्चर समीक्षा पाइपलाइन
```
search_knowledge → get_entity → get_neighbors("enforces")
  → get_neighbors("violates") → find_path → अनुपालन रिपोर्ट
  → उपयोगकर्ता चुनता है: रिफैक्टरिंग योजना / सलाह / विकल्प खोजें
```

### श्रृंखला 3: समस्या निदान पाइपलाइन
```
search_knowledge(लक्षण) → get_entity → get_neighbors("solved_by")
  → मूल कारण रिपोर्ट → उपयोगकर्ता चुनता है: फ़िक्स लागू करें / सलाह / सत्यापित करें
```

### श्रृंखला 4: शिक्षण पाइपलाइन
```
search_knowledge(विषय) → get_entity → get_neighbors("related_to")
  → अवधारणा मानचित्र → उपयोगकर्ता चुनता है: कोड उदाहरण / कोड पर लागू करें / तुलना करें
```

### क्रॉस-टूल श्रृंखला नियम

हर टूल कॉल स्वाभाविक रूप से अगले की ओर ले जाता है:

| ...कॉल करने के बाद | हमेशा इसके साथ फ़ॉलो करें |
|---------------------|---------------------|
| `analyze_code` | पहचाने गए स्मेल पर `suggest_refactorings` |
| `suggest_refactorings` | विकल्पों के लिए `get_neighbors(smell_id, "solved_by")` |
| `search_knowledge` | शीर्ष 1-2 परिणामों पर `get_entity` |
| `get_entity` (स्मेल) | प्रभावित सिद्धांतों के लिए `get_neighbors(id, "violates")` |
| `get_entity` (पैटर्न) | लागू नियमों के लिए `get_neighbors(id, "enforces")` |
| कई स्मेल पहचाने गए | कारण मैपिंग के लिए `find_path(smell_A, smell_B)` |

---

## अन्य टूल्स के लिए स्थापना

### Cursor

```bash
epis install cursor
```

MCP कॉन्फ़िग `~/.cursor/mcp.json` में जोड़ता है:
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

### Codex (OpenAI)

```bash
epis install codex
```

प्रोजेक्ट रूट में एजेंट परिभाषाओं के साथ `AGENTS.md` जनरेट करता है।

### कस्टम MCP एकीकरण

यदि आपका टूल MCP का समर्थन करता है, तो मैन्युअल रूप से कॉन्फ़िगर करें:

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

## बैकग्राउंड सेवा के रूप में चलाएँ

बेहतर प्रदर्शन के लिए, Episteme MCP को persistent HTTP प्रॉक्सी के रूप में चलाएँ:

```bash
# बैकग्राउंड सेवा प्रारंभ करें
epis service start

# स्थिति जाँचें
epis service status
# आउटपुट: Running on http://localhost:43175 (PID 12345)

# बूट पर स्वतः-प्रारंभ सक्षम करें (macOS)
epis service enable

# सेवा रोकें
epis service stop
```

HTTP प्रॉक्सी का उपयोग करने के लिए MCP कॉन्फ़िग अपडेट करें:

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

लॉग: `~/.episteme/logs/mcp.out.log`

---

## समस्या निवारण

### टूल्स Claude में दिखाई नहीं दे रहे

1. कॉन्फ़िग फ़ाइल मौजूद है जाँचें: `cat ~/.claude/claude_desktop_config.json`
2. episteme PATH में है सत्यापित करें: `which episteme`
3. MCP सीधे टेस्ट करें: `episteme mcp`
4. लॉग जाँचें: `tail -f ~/.episteme/logs/mcp.err.log`

### "Database not found" त्रुटि

```bash
# ज्ञान डेटाबेस पुनर्निर्मित करें
epis build --rebuild
```

### धीमी खोज प्रतिक्रियाएँ

```bash
# GPU त्वरण उपयोग करें
epis build --gpu

# या बैकग्राउंड सेवा के रूप में चलाएँ (तेज़ वार्मअप)
epis service start
```

### एजेंट टूल्स उपयोग नहीं कर रहा

सुनिश्चित करें कि एजेंट में टूल-कॉलिंग क्षमता है। Claude Code में:
```
उपयोगकर्ता: "रीट्री लॉजिक के लिए पैटर्न खोजने के लिए Episteme उपयोग करें"
      ^^^^ टूल उपयोग स्पष्ट रूप से बताएँ
```

---

## उन्नत: कस्टम ज्ञान एकीकरण

Episteme (सामान्य ज्ञान) को Alcove (टीम ज्ञान) के साथ संयोजित करें:

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

दोहरे-स्रोत पैटर्न के लिए [Alcove एकीकरण मार्गदर्शिका](./alcove-integration.md) देखें।

---

## API विकल्प

यदि आपका AI टूल MCP का समर्थन नहीं करता है, तो REST API उपयोग करें:

```bash
# API सर्वर प्रारंभ करें
docker-compose up -d

# किसी भी टूल से उपयोग करें
curl http://localhost:8000/search?q=strategy+pattern
```

एंडपॉइंट के लिए [API दस्तावेश़ीकरण](./api.md) देखें।

---

## स्वचालित ट्रिगरिंग (Claude Code)

जब आप किसी समस्या का वर्णन प्राकृतिक भाषा में करते हैं, तो Claude Code स्वचालित रूप से आशय का पता लगाता है और उपयुक्त Episteme टूल कॉल करता है — **आपको Episteme को स्पष्ट रूप से बताने की आवश्यकता नहीं है**। नीचे सटीक ट्रिगर पैटर्न और उदाहरण हैं।

### यह कैसे काम करता है

```
आपका प्राकृतिक भाषा इनपुट
    ↓ Claude कीवर्ड/पैटर्न पहचानता है
    ↓ Episteme टूल स्वचालित रूप से कॉल होता है
    ↓ ज्ञान ग्राफ सत्यापित डेटा लौटाता है
    ↓ (डिज़ाइन पैटर्न · कोड स्मेल · रिफैक्टरिंग तकनीकें · इंजीनियरिंग नियम)
    ↓ Claude की प्रतिक्रिया साक्ष्य-आधारित है
```

> **नोट:** यह प्रॉम्प्ट-आधारित स्वतः-ट्रिगरिंग है, हार्ड हुक नहीं। कॉल की गारंटी के लिए, `/episteme` स्किल सीधे उपयोग करें।

### कोड संरचना समस्याएँ

| आप क्या कहते हैं (उदाहरण) | Episteme क्या पहचानता है | स्वचालित टूल कॉल |
|--------------------------|---------------------|--------------------------|
| "यह क्लास बहुत कुछ करती है", "यह फ़ाइल 300 लाइनों से अधिक है" | God Class, Large Class, Single Responsibility | `search_knowledge("god class large class single responsibility")` |
| "यह फ़ंक्शन बहुत लंबा है", "इस विधि में बहुत अधिक लाइनें हैं" | Long Method | `search_knowledge("long method extract method")` |
| "कोड बहुत जटिल है", "समझना कठिन है" | जटिलता, Cognitive Overload | `search_knowledge("complexity smell cognitive overload")` |
| "मैंने यह हर जगह कॉपी-पेस्ट किया", "डुप्लिकेट लॉजिक है" | Duplicate Code, Clone | `search_knowledge("duplicated code clone smell")` |

### युग्मन और निर्भरता समस्याएँ

| आप क्या कहते हैं (उदाहरण) | Episteme क्या पहचानता है | स्वचालित टूल कॉल |
|--------------------------|---------------------|--------------------------|
| "बिज़नेस लॉजिक सीधे DB कॉल करता है" | युग्मन, Persistence, Repository | `search_knowledge("coupling persistence repository data access layer")` |
| "X बदलने से Y टूटता है", "परिवर्तन हर जगह फैलते हैं" | Brittle Coupling, Change Propagation | `search_knowledge("brittle coupling change propagation rigidity")` |
| "नया प्रकार जोड़ने का मतलब हर जगह छूना", "switch-case लगातार बढ़ रहा है" | Open/Closed, Strategy, Polymorphism | `search_knowledge("open closed principle strategy polymorphism")` |

### टेस्टिंग और गुणवत्ता समस्याएँ

| आप क्या कहते हैं (उदाहरण) | Episteme क्या पहचानता है | स्वचालित टूल कॉल |
|--------------------------|---------------------|--------------------------|
| "यह टेस्ट करना कठिन है", "इसके लिए यूनिट टेस्ट नहीं लिख सकते" | टेस्टेबिलिटी, Dependency Injection | `search_knowledge("testability dependency injection mockability")` |

### प्रदर्शन और समवर्तीता समस्याएँ

| आप क्या कहते हैं (उदाहरण) | Episteme क्या पहचानता है | स्वचालित टूल कॉल |
|--------------------------|---------------------|--------------------------|
| "API धीमी है", "प्रतिक्रिया समय बहुत अधिक है" | N+1 Query, Lazy Loading, Caching | `search_knowledge("N+1 query lazy loading caching performance")` |
| "क्या यह Thread-सुरक्षित है?", "कोई समवर्तीता समस्याएँ?" | Thread Safety, Race Condition | `search_knowledge("thread safety race condition shared mutable state")` |

### कोड समीक्षा और विश्लेषण

| आप क्या कहते हैं (उदाहरण) | स्वचालित टूल कॉल |
|--------------------------|--------------------------|
| कोड स्निपेट साझा करें और समीक्षा का अनुरोध करें | `analyze_code(code)` → `suggest_refactorings(code)` |
| "इस कोड में सुधार करें", "मेरे लिए रिफैक्टर करें" | `suggest_refactorings(code)` |

### आर्किटेक्चर चर्चाएँ

| आप क्या कहते हैं (उदाहरण) | स्वचालित टूल कॉल |
|--------------------------|--------------------------|
| "माइक्रोसर्विसेज़ बनाम मोनोलिथ", "हमें इसे कैसे विभाजित करना चाहिए?" | `search_knowledge("monolith microservice decomposition bounded context")` |
| "क्या यह आर्किटेक्चर ठीक है?", "इस डिज़ाइन की समीक्षा करें" | `search_knowledge("layered architecture coupling responsibility")` |

---

## अगले चरण

1. **एजेंट आज़माएँ**: episteme-advisor से पूछें "क्या मुझे Singleton उपयोग करना चाहिए?"
2. **कोड विश्लेषण करें**: एक फ़ंक्शन पेस्ट करें और code-reviewer से स्मेल जाँचने को कहें
3. **ग्राफ का अन्वेषण करें**: episteme-researcher उपयोग करके पैटर्न संबंध खोजें
4. **कस्टम वर्कफ़्लो**: टूल्स संयोजित करें (analyze → suggest → search)

अधिक उदाहरणों के लिए देखें:
- [Alcove एकीकरण](./alcove-integration.md) — टीम ज्ञान + Episteme
- [API संदर्भ](./api.md) — REST एंडपॉइंट
