# Episteme विकास मार्गदर्शिका

**प्रोजेक्ट:** Episteme v0.1.0
**भाषा:** Rust (संस्करण 2024)
**अंतिम अपडेट:** 2026-05-03

---

## वर्तमान स्थिति

| घटक | स्थिति | विवरण |
|------|--------|--------|
| **ज्ञान आधार** | पूर्ण | 22 पैटर्न, 66 रिफैक्टरिंग, 56 नियम, 23 स्मेल, 201 संबंध |
| **कोड स्मेल पहचान** | उत्पादन | 16 डिटेक्टर फ़ंक्शन, 10 भाषाएँ |
| **REST API** | उत्पादन | 17 एंडपॉइंट (axum), रेट लिमिटिंग, प्रमाणीकरण |
| **MCP सर्वर** | उत्पादन | 6 टूल्स, stdio + HTTP ट्रांसपोर्ट |
| **RAG पाइपलाइन** | उत्पादन | SQLite + FTS5 + fastembed (ONNX) |
| **ग्राफ विज़ुअलाइज़ेशन** | उत्पादन | D3-force के साथ संवादात्मक Web UI |

---

## आर्किटेक्चर

षटकोणीय (पोर्ट्स और एडाप्टर्स) आर्किटेक्चर:

```
src/
├── commands/          # CLI उपकमांड हैंडलर (clap)
│   ├── analysis.rs    # analyze, infer
│   ├── build.rs       # build (RAG पाइपलाइन)
│   ├── explore.rs     # explore (खोज/REPL)
│   ├── graph.rs       # ग्राफ क्वेरीज़
│   ├── install.rs     # स्थापना विज़ार्ड (TUI)
│   ├── service.rs     # MCP HTTP डेमन प्रबंधन
│   └── other.rs       # api, mcp, web, telemetry, hooks
├── adapters/          # इंफ्रास्ट्रक्चर परत
│   ├── regex_parsers.rs   # GenericParser (10 भाषाएँ, OnceLock regex कैश)
│   ├── python_ast_parser.rs  # Python AST (rustpython-parser)
│   ├── search_engines.rs  # FTS5 कीवर्ड + cosine similarity
│   ├── service.rs         # MCP HTTP डेमन
│   ├── sqlite_db.rs       # SQLite कनेक्शन पूल
│   ├── cache.rs           # Redis कैशिंग (वैकल्पिक)
│   └── ...
├── domain/            # व्यावसायिक तर्क (कोई बाहरी निर्भरता नहीं)
│   ├── graph.rs       # KnowledgeGraph (BFS, subgraph, contradictions, Jaccard)
│   ├── detectors.rs   # TieredAccum के साथ 16 स्मेल डिटेक्टर
│   ├── engine.rs      # RefactoringInferenceEngine + RefactoringRanker
│   ├── summarizer.rs  # Detail-level प्रतिक्रिया अनुकूलन
│   └── types.rs       # EntityType, RelationType, कोर प्रकार
├── server/            # HTTP परत (axum)
│   ├── api_routes.rs  # 17 REST एंडपॉइंट
│   ├── mcp_handler.rs # MCP पतला फ़ैकेड
│   ├── mcp_search.rs  # खोज सेवा
│   ├── mcp_graph.rs   # ग्राफ सेवा
│   └── mcp_analysis.rs # कोड विश्लेषण सेवा
└── ports/             # Traits (षटकोणीय सीमाएँ)
    ├── parser.rs      # CodeParser trait
    ├── search.rs      # SearchEngine trait
    ├── graph.rs       # GraphStore trait
    └── embeddings.rs  # EmbeddingProvider trait
```

---

## तकनीकी स्टैक

| घटक | तकनीक | उद्देश्य |
|------|--------|----------|
| **भाषा** | Rust (संस्करण 2024) | सुरक्षा, प्रदर्शन, एकल-बाइनरी |
| **Web फ़्रेमवर्क** | axum | REST API + MCP HTTP ट्रांसपोर्ट |
| **डेटाबेस** | rusqlite (बंडल किया गया SQLite) | ज्ञान ग्राफ + वेक्टर स्टोर |
| **खोज** | FTS5 + cosine similarity | कीवर्ड + सिमेंटिक हाइब्रिड खोज |
| **एम्बेडिंग** | fastembed (ONNX Runtime) | स्थानीय, zero-config एम्बेडिंग जनरेशन |
| **CLI** | clap (derive) | 15 उपकमांड |
| **Python AST** | rustpython-parser | AST-आधारित Python स्मेल पहचान |
| **अन्य भाषाएँ** | regex (OnceLock कैश किया गया) | GenericParser फ़्रेमवर्क |

---

## कोड स्मेल डिटेक्टर (16)

| ID | स्मेल | पहचान |
|----|-------|--------|
| SMELL-01 | Long Method | LOC थ्रेसहोल्ड |
| SMELL-02 | Long Parameter List | पैरामीटर गणना |
| SMELL-03 | Primitive Obsession | आदिम पैरामीटर अनुपात |
| SMELL-04 | Large Class | विधि + फ़ील्ड गणना |
| SMELL-05 | Data Clumps | दोहराए गए पैरामीटर समूह (स्टब) |
| SMELL-06 | Switch Statements | switch/match गणना |
| SMELL-07 | Data Class | विधियाँ बनाम फ़ील्ड अनुपात |
| SMELL-08 | Temporary Field | सशर्त फ़ील्ड उपयोग (स्टब) |
| SMELL-09 | Shotgun Surgery | परिवर्तन युग्मन (स्टब) |
| SMELL-10 | Divergent Change | विधि सामंजस्य मेट्रिक्स |
| SMELL-11 | Lazy Class | निम्न LOC + विधि गणना |
| SMELL-12 | Speculative Generality | ठोस के बिना सार |
| SMELL-13 | Duplicate Code | हैश-आधारित समानता (आंशिक) |
| SMELL-14 | Middle Man | प्रतिनिधिमंडल अनुपात |
| SMELL-15 | Parallel Inheritance Hierarchies | पदानुक्रम प्रतिबिंब (स्टब) |
| SMELL-16 | Comments | टिप्पणी-से-कोड अनुपात (स्टब) |
| SMELL-17 | Dead Code | अप्राप्य/अप्रयुक्त पहचान (स्टब) |
| SMELL-18 | Feature Envy | बाहरी कॉल अनुपात |
| SMELL-19 | Inappropriate Intimacy | क्रॉस-क्लास निजी पहुँच (स्टब) |
| SMELL-20 | Message Chains | कॉल श्रृंखला गहराई |
| SMELL-21 | God Object | समग्र: LOC + विधियाँ + युग्पन |
| SMELL-22 | Refused Bequest | ओवरराइड-टू-नथिंग अनुपात (स्टब) |
| SMELL-23 | Alternative Classes with Different Interfaces | इंटरफ़ेस विचलन (स्टब) |

---

## विकास सेटअप

```bash
# क्लोन और बिल्ड करें (Rust 1.95+ आवश्यक)
git clone https://github.com/epicsagas/Episteme.git
cd Episteme
cargo build

# टेस्ट चलाएँ
cargo test

# लिंट
cargo clippy -- -D warnings

# स्थानीय रूप से स्थापित करें (डेटा सीड करता है और DB स्वचालित रूप से बनाता है)
cargo install --path .
epis install --local
```

---

## API एंडपॉइंट (17)

| विधि | पथ | विवरण |
|--------|------|-------------|
| GET | `/` | सेवा जानकारी |
| GET | `/health` | हेल्थ चेक |
| GET | `/live` | लाइवनेस प्रोब |
| GET | `/ready` | रेडीनेस प्रोब |
| GET | `/stats` | ग्राफ सांख्यिकी |
| POST | `/analyze` | कोड स्मेल पहचान |
| POST | `/refactor` | रिफैक्टरिंग सुझाव |
| GET | `/search` | ज्ञान खोज |
| POST | `/search` | ज्ञान खोज (POST) |
| GET | `/graph/{id}` | एंटिटी प्राप्त करें |
| GET | `/graph/{id}/neighbors` | पड़ोसी प्राप्त करें |
| POST | `/graph/neighbors` | पड़ोसी प्राप्त करें (POST) |
| POST | `/graph/subgraph` | सबग्राफ निकालें |
| GET | `/graph/path` | सबसे छोटा पथ |
| GET | `/graph/contradictions` | विरोधाभास खोजें |
| POST | `/graph/infer-transitive` | ट्रांज़िटिव संबंध अनुमान लगाएँ |
| GET | `/metrics` | Prometheus मेट्रिक्स |

---

## भविष्य का रोडमैप

- **IDE प्लगइन** — VSCode, IntelliJ नेटिव एकीकरण
- **कस्टम एंटिटीज़** — टीम-विशिष्ट पैटर्न/स्मेल जोड़ें
- **टीम मेट्रिक्स** — संगठन भर में पैटर्न उपयोग को समेकित करें
- **बहुभाषी दस्तावेज़** — कोरियाई, जापानी, चीनी में ज्ञान आधार
- **संवादात्मक ट्यूटोरियल** — MCP टूल्स के लिए इन-ऐप निर्देशित टूर

---

*अंतिम अपडेट: 2026-05-03*
