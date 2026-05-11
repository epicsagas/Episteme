# Episteme REST API दस्तावेज़ीकरण

**संस्करण:** 0.1.0
**बेस URL:** `http://localhost:8000`

---

## त्वरित आरंभ

```bash
# सर्वर प्रारंभ करें
epis api

# या कस्टम होस्ट/पोर्ट के साथ
epis api --host 0.0.0.0 --port 8000

# Docker
docker-compose up -d
```

---

## प्रमाणीकरण

`/`, `/health`, `/live`, `/ready` को छोड़कर सभी एंडपॉइंट्स के लिए API कुंजी प्रमाणीकरण आवश्यक है।

### API कुंजी प्रमाणीकरण

**हेडर:** `X-API-Key: <your-api-key>`

**मोड:**

1. **उत्पादन मोड** — `EPISTEME_API_KEYS` पर्यावरण चर सेट करें
   - कॉमा-सेपरेटेड मान्य API कुंजियों की सूची
   - सभी संरक्षित एंडपॉइंट्स के लिए मान्य कुंजी आवश्यक
   - यदि अनुपस्थित/अमान्य है तो 401 Unauthorized लौटाता है

2. **विकास मोड** — `EPISTEME_API_KEYS` को खाली छोड़ें या अनसेट करें
   - कोई प्रमाणीकरण आवश्यक नहीं

### API कुंजियाँ उत्पन्न करना

```bash
openssl rand -base64 32
```

### अनुरोध उदाहरण

```bash
# प्रमाणीकरण के साथ (उत्पादन)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"code": "def long_method(): pass", "min_confidence": 0.5}'

# प्रमाणीकरण के बिना (विकास मोड)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"code": "def long_method(): pass"}'
```

---

## रेट लिमिटिंग

सभी एंडपॉइंट्स प्रति IP पते रेट-लिमिटेड हैं जिसमें TTL-आधारित बकेट निष्कासन है।

| एंडपॉइंट | रेट लिमिट | कारण |
|----------|------------|------|
| `/analyze` | 20/मिनट | CPU-गहन |
| `/refactor` | 20/मिनट | CPU-गहन |
| `/search` | 50/मिनट | एम्बेडिंग गणना |
| `/stats`, `/graph/*` | 100/मिनट | मानक |
| `/`, `/health` | असीमित | सार्वजनिक |

सीमा पार होने पर, `Retry-After` हेडर के साथ 429 लौटाता है।

---

## एंडपॉइंट

### हेल्थ और जानकारी

#### `GET /`

सेवा जानकारी।

**प्रतिक्रिया:**
```json
{
  "name": "episteme",
  "version": "0.1.0",
  "description": "Software engineering knowledge graph",
  "endpoints": ["analyze", "search", "graph", "refactor", "stats"]
}
```

#### `GET /health`

घटक स्थिति के साथ हेल्थ चेक।

**प्रतिक्रिया:**
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

लाइवनेस प्रोब: `{"status": "alive"}`

#### `GET /ready`

रेडीनेस प्रोब: `{"status": "ready"}` (यदि तैयार नहीं है तो 503)

#### `GET /stats`

ग्राफ सांख्यिकी।

**प्रतिक्रिया:**
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

### कोड विश्लेषण

#### समर्थित कोड स्मेल (16 डिटेक्टर)

| ID | नाम | भाषाएँ |
|----|------|---------|
| SMELL-01 | Long Method | सभी |
| SMELL-02 | Long Parameter List | सभी |
| SMELL-03 | Primitive Obsession | Python |
| SMELL-04 | Large Class | सभी |
| SMELL-05 | Data Clumps | सभी (स्टब) |
| SMELL-06 | Switch Statements | सभी |
| SMELL-07 | Data Class | सभी |
| SMELL-09 | Shotgun Surgery | सभी (स्टब) |
| SMELL-10 | Divergent Change | सभी |
| SMELL-11 | Lazy Class | सभी |
| SMELL-12 | Speculative Generality | सभी |
| SMELL-13 | Duplicate Code | सभी (आंशिक) |
| SMELL-14 | Middle Man | सभी |
| SMELL-18 | Feature Envy | सभी |
| SMELL-20 | Message Chains | सभी |
| SMELL-21 | God Object | सभी |

#### `POST /analyze`

कोड स्मेल का पता लगाएँ।

**अनुरोध:**
```json
{
  "code": "def long_method():\n    ...",
  "language": "python",
  "min_confidence": 0.5
}
```

**प्रतिक्रिया:**
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

पहचाने गए स्मेल के लिए रैंक किए गए रिफैक्टरिंग सुझाव प्राप्त करें।

**अनुरोध:**
```json
{
  "code": "def long_method():\n    ...",
  "top_k": 3,
  "min_confidence": 0.5
}
```

**प्रतिक्रिया:**
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

### खोज

#### `GET /search`

क्वेरी पैरामीटर के माध्यम से खोजें: `/search?q=strategy+pattern&top_k=5`

#### `POST /search`

ज्ञान आधार में सिमेंटिक खोज।

**अनुरोध:**
```json
{
  "query": "How to fix Long Method?",
  "top_k": 5,
  "entity_type": "refactoring"
}
```

**प्रतिक्रिया:**
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

### ज्ञान ग्राफ

#### `GET /graph/{id}`

ID द्वारा एंटिटी विवरण प्राप्त करें।

**उदाहरण:** `GET /graph/DP-005`

#### `GET /graph/{id}/neighbors`

एंटिटी के पड़ोसी प्राप्त करें: `/graph/SMELL-01/neighbors?relation_type=solved_by`

#### `POST /graph/neighbors`

पड़ोसी प्राप्त करें (POST)।

**अनुरोध:**
```json
{
  "entity_id": "SMELL-01",
  "relation_type": "solved_by"
}
```

#### `GET /graph/path`

सबसे छोटा पथ: `/graph/path?from_id=SMELL-01&to_id=LAW-042-S&max_depth=5`

#### `POST /graph/subgraph`

सबग्राफ निकालें।

**अनुरोध:**
```json
{
  "entity_id": "DP-005",
  "depth": 2
}
```

#### `GET /graph/contradictions`

विरोधाभासी संबंधों वाली एंटिटीज़ खोजें।

#### `POST /graph/infer-transitive`

ट्रांज़िटिव प्रवर्तन संबंधों का अनुमान लगाएँ।

---

### मॉनिटरिंग

#### `GET /metrics`

Prometheus-प्रारूप मेट्रिक्स सहित:
- `http_requests_total` — विधि, एंडपॉइंट, स्थिति द्वारा
- `episteme_smells_detected_total` — smell_id द्वारा
- `episteme_searches_total` — entity_type द्वारा
- `episteme_analysis_duration_seconds` — हिस्टोग्राम

---

## प्रदर्शन

| एंडपॉइंट | औसत विलंबता | टिप्पणी |
|----------|-------------|---------|
| `/analyze` | ~5ms | Regex + AST पार्सिंग (OnceLock कैश किया गया) |
| `/refactor` | ~10ms | ग्राफ ट्रैवर्सल शामिल |
| `/search` | ~20ms | FTS5 + cosine similarity |
| `/graph/neighbors` | ~1ms | इन-मेमोरी ग्राफ |
| `/graph/path` | ~5ms | BFS अधिकतम गहराई 5 |

---

## त्रुटि हैंडलिंग

| स्थिति कोड | अर्थ |
|-------------|------|
| 200 | सफल |
| 400 | अमान्य अनुरोध |
| 401 | API कुंजी अनुपस्थित/अमान्य |
| 404 | एंटिटी नहीं मिली |
| 429 | रेट लिमिट पार हुई |
| 500 | आंतरिक त्रुटि |

---

## पर्यावरण चर

```bash
# सर्वर
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEYS=key1,key2

# डेटा
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# लॉगिंग
RUST_LOG=info
```

---

## लाइसेंस

APACHE-2.0 लाइसेंस
