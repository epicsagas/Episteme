# Alcove पारिस्थितिकी तंत्र — आर्किटेक्चर और क्षमता विश्लेषण

> Episteme की अंतर्निहित ज्ञान परत (TK-*) और Alcove दस्तावेज़ीकरण पारिस्थितिकी तंत्र की विस्तृत तुलना, जिसमें भंडारण मॉडल, खोज क्षमताएँ, जीवनचक्र प्रबंधन और उपयोग-मामले मार्गदर्शन शामिल हैं।

---

## 1. आर्किटेक्चर अवलोकन

### Episteme अंतर्निहित ज्ञान (TK-*)

| पहलू | विवरण |
|--------|--------|
| **भंडारण** | SQLite एकल फ़ाइल (`~/.episteme/user_knowledge.db`) |
| **स्कीमा** | 5 तालिकाएँ: `user_entities`, `user_relations`, `user_embeddings`, `user_entities_fts` (FTS5 वर्चुअल), `insight_seq` |
| **इकाई** | एक अंतर्दृष्टि = एक `UserEntity` पंक्ति (TK-xxx ID) |
| **ग्राफ** | रनटाइम पर `CompositeGraph` के माध्यम से कैननिकल ग्राफ के साथ विलय — क्रॉस-लेयर पथ ट्रैवर्सल सक्षम (TK-001 → DP-005 → SMELL-01) |
| **समवर्तीता** | MCP + CLI समकालिक पहुँच के लिए `Mutex<Connection>` + WAL मोड |

### Alcove दस्तावेज़ीकरण प्रणाली

| पहलू | विवरण |
|--------|--------|
| **भंडारण** | फ़ाइल सिस्टम पर Markdown फ़ाइलें + Tantivy BM25 इंडेक्स + sqlite-vec एम्बेडिंग |
| **संरचना** | 3-स्तरीय वर्गीकरण: कोर (7), सप्लिमेंटरी (19), सार्वजनिक (15) फ़ाइलें प्रति प्रोजेक्ट |
| **इकाई** | एक संरचित Markdown फ़ाइल (PRD, ARCHITECTURE, DECISIONS, आदि) |
| **ग्राफ** | wikilink + फ़ाइल-पाथ आधारित ढीले कनेक्शन |
| **समवर्तीता** | फ़ाइल-आधारित लॉक (`.index_lock`) प्रति docs root, प्रति-वॉल्ट इंडेक्स आइसोलेशन |
| **वॉल्ट** | Obsidian PARA फ़ोल्डर्स के लिए 3 सिमलिंक: areas (8 दस्तावेज़), resources (71), zettelkasten (17) |

---

## 2. भंडारण मॉडल तुलना

### Episteme TK-* स्कीमा

```sql
-- कोर तालिका
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,           -- TK-001, TK-002, ...
    title TEXT,                    -- स्वतः: पहली पंक्ति, अधिकतम 80 अक्षर
    content TEXT,                  -- मुक्त पाठ (अधिकतम लंबाई नहीं)
    author TEXT DEFAULT 'user',
    confidence REAL DEFAULT 0.5,   -- प्रत्येक पुष्ट लिंक +0.05, अधिकतम 1.0
    evidence_count INTEGER DEFAULT 0,
    last_validated TEXT DEFAULT '',
    tags TEXT DEFAULT '[]',        -- JSON ऐरे
    relations TEXT DEFAULT '{}',   -- JSON HashMap<relation_type, Vec<entity_id>>
    link_provenance TEXT DEFAULT '{}',
    created_at TEXT,
    updated_at TEXT
);

-- सामान्यीकृत संबंध (derives_from, applies_to, supersedes)
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT,
    relation_type TEXT,
    to_id TEXT,
    UNIQUE(from_id, relation_type, to_id)
);

-- FTS5 फ़ुल-टेक्स्ट खोज
CREATE VIRTUAL TABLE user_entities_fts USING fts5(title, content, tags, content=user_entities);
```

### Alcove फ़ाइल संरचना

```
~/.alcove/
  config.toml                    # वैश्विक कॉन्फ़िग (docs_root, core/team/public फ़ाइल सूचियाँ, एम्बेडिंग मॉडल)
  docs -> सिमलिंक               # → Obsidian/SecondBrain/99-Archives/projects
  vaults/
    areas -> सिमलिंक             # → Obsidian/02-Areas (8 दस्तावेज़)
    resources -> सिमलिंक         # → Obsidian/03-Resources (71 दस्तावेज़)
    zettelkasten -> सिमलिंक      # → Obsidian/10-Zettelkasten (17 दस्तावेज़)
  models/                        # कैश किए गए ONNX एम्बेडिंग मॉडल
  logs/

<docs_root>/<project>/
  .alcove/
    index/                       # Tantivy BM25 इंडेक्स फ़ाइलें
    index_meta.json              # फ़ाइल फ़िंगरप्रिंट्स (mtime + size)
    vectors.db                   # sqlite-vec एम्बेडिंग
  PRD.md                         # उत्पाद आवश्यकताएँ
  ARCHITECTURE.md                # सिस्टम डिज़ाइन
  PROGRESS.md                    # माइलस्टोन और स्थिति
  DECISIONS.md                   # आर्किटेक्चर निर्णय रिकॉर्ड
  CONVENTIONS.md                 # कोडिंग मानक
  SECRETS_MAP.md                 # पर्यावरण चर और रहस्य
  DEBT.md                        # तकनीकी ऋण रजिस्टर
```

---

## 3. ज्ञान विशेषता

| आयाम | Episteme TK-* | Alcove |
|-----------|---------------|--------|
| **प्रकार** | क्षणिक अंतर्दृष्टि, सीखे गए पाठ, टीम निर्णय | संरचित प्रोजेक्ट दस्तावेश (आवश्यकताएँ, आर्किटेक्चर, निर्णय) |
| **परिवर्तनीयता** | परिवर्तनीय (SQLite CRUD) | परिवर्तनीय (फ़ाइल संपादन + इंडेक्स पुनर्निर्माण) |
| **स्रोत** | उपयोगकर्ता-योगदान मुक्त पाठ | उपयोगकर्ता-लिखित + टेम्पलेट से एजेंट-जनित |
| **प्राधिकरण** | व्यक्तिगत/टीम अवलोकन | टीम अधिदेश / सांगठनिक नीति |
| **ग्रैन्युलैरिटी** | परमाणु (प्रति प्रविष्टि एक अंतर्दृष्टि) | खंडित (प्रति DECISIONS.md कई ADR) |
| **लिंकिंग** | कैननिकल एंटिटीज़ को स्वतः-पहचान (कीवर्ड स्कोरिंग) | मैनुअल wikilinks + markdown लिंक |
| **वर्ज़निंग** | कोई नहीं (केवल SQLite) | Git-आधारित (फ़ाइल = सत्य का स्रोत) |

### अंतर्दृष्टि जीवनचक्र (Episteme TK-*)

```
add_insight(text, tags?, project?, linked_entities?)
  │
  ├── TK-xxx ID जनरेट करें (परमाणु अनुक्रम)
  ├── detect_canonical_links() — कीवर्ड मिलान → शीर्ष 5 कैननिकल एंटिटीज़
  │     स्कोर >= 0.5 → स्वतः लिंक (derives_from)
  │     स्कोर < 0.5 → सुझावित लिंक
  ├── FTS5 डुप्लिकेट पहचान → DuplicateCandidate[]
  ├── SQLite + इन-मेमोरी कैश में बनाए रखें
  └── लौटाएँ: { id, auto_links, suggested_links, duplicates, confidence }

confirm_links(id, accepted[], rejected[])
  │
  ├── derives_from/applies_to संबंध जोड़ें
  ├── link_provenance स्रोत को "manual" में अपग्रेड करें
  ├── विश्वास बढ़ाएँ (प्रति लिंक +0.05, अधिकतम 1.0)
  └── अपडेट बनाए रखें

search_insights(query, limit?)
  │
  └── FTS5 MATCH क्वेरी → रैंक किए गए परिणाम
```

### दस्तावेश जीवनचक्र (Alcove)

```
init_project(project_name, project_path?)
  │
  ├── टेम्पलेट से 7 कोर दस्तावेश बनाएँ (PRD, ARCHITECTURE, ...)
  ├── वैकल्पिक रूप से सार्वजनिक दस्तावेश बनाएँ (README, CHANGELOG, ...)
  └── खोज इंडेक्स पुनर्निर्माण करें

validate_docs()
  │
  ├── आवश्यक फ़ाइल अस्तित्व जाँचें
  ├── टेम्पलेट प्लेसहोल्डर जाँचें (TODO, FIXME)
  ├── आवश्यक अनुभाग शीर्षलेख जाँचें
  ├── न्यूनतम सूची आइटम गणना जाँचें
  └── लौटाएँ: प्रति फ़ाइल pass/warn/fail

lint_project()
  │
  ├── टूटे हुए [[wikilinks]] और markdown लिंक का पता लगाएँ
  ├── अनाथ फ़ाइलें खोजें (किसी दस्तावेश से लिंक नहीं)
  ├── पुराने मार्कर खोजें (WIP, TODO, FIXME, DRAFT, DEPRECATED)
  └── पुराने वर्ष संदर्भ खोजें (2+ वर्ष पुराने)

audit_project()
  │
  ├── ग़ुम आवश्यक दस्तावेशों के लिए निजी doc-repo स्कैन करें
  ├── उजागर आंतरिक दस्तावेशों के लिए सार्वजनिक प्रोजेक्ट repo स्कैन करें
  ├── फ़ाइलों को स्तरों में वर्गीकृत करें
  └── लौटाएँ: suggested_actions[]
```

---

## 4. खोज क्षमताएँ

| क्षमता | Episteme TK-* | Alcove |
|-----------|---------------|--------|
| **इंजन** | FTS5 (कीवर्ड मिलान) | Tantivy BM25 + sqlite-vec cosine similarity |
| **फ़्यूज़न** | कोई नहीं | RRF (Reciprocal Rank Fusion, k=60) |
| **CJK** | कोई विशेष समर्थन नहीं | NgramTokenizer (न्यूनतम=2, अधिकतम=3) |
| **चंकिंग** | लागू नहीं (एक पंक्ति = एक अंतर्दृष्टि) | 200-500 अक्षर चंक |
| **वृद्धिशील** | लागू नहीं (एकल तालिका) | mtime + size फ़िंगरप्रिंट तुलना |
| **वेक्टर खोज** | स्कीमा मौजूद (`user_embeddings`) लेकिन **कनेक्ट नहीं** | पूर्णतः परिचालित (MultilingualE5Small, 384d) |
| **दायरा** | एकल डेटाबेस | प्रति-प्रोजेक्ट या वैश्विक (क्रॉस-प्रोजेक्ट) |
| **फ़ॉलबैक** | कोई नहीं | इंडेक्स न होने पर grep सबस्ट्रिंग मिलान |

---

## 5. सुविधा पूर्णता

| सुविधा | Episteme TK-* | Alcove |
|---------|---------------|--------|
| बनाएँ | `add_insight` | `init_project`, फ़ाइल संपादन |
| पढ़ें | `search_insights` (केवल खोज, ID द्वारा प्राप्ति नहीं) | `get_doc_file`, `search_project_docs` |
| अपडेट करें | MCP के माध्यम से उजागर नहीं | प्रत्यक्ष फ़ाइल संपादन + `rebuild_index` |
| हटाएँ | MCP के माध्यम से उजागर नहीं | फ़ाइल हटाएँ + `rebuild_index` |
| सत्यापन | कोई नहीं | `validate_docs`, `lint_project` |
| ऑडिट | कोई नहीं | `audit_project` (सार्वजनिक/निजी पृथक्करण) |
| बैकअप | कोई नहीं | `backup_vault` (Git कमिट स्नैपशॉट) |
| आयात | कोई नहीं | `promote_document` (Obsidian → doc-repo) |
| नीति | कोई नहीं | `policy.toml` प्रवर्तन स्तरों के साथ |
| टेम्पलेट | कोई नहीं | 7 कोर + 19 सप्लिमेंटरी + 15 सार्वजनिक |

---

## 6. Alcove वॉल्ट प्रणाली

तीन वॉल्ट, Obsidian PARA संरचना से सिमलिंक किए गए:

| वॉल्ट | लक्ष्य | दस्तावेश | उद्देश्य |
|-------|--------|-----------|---------|
| `areas` | `02-Areas` | 8 | डोमेन क्षेत्र: MCP एजेंट, DevOps, Rust, LLM/RAG, ओपन सोर्स |
| `resources` | `03-Resources` | 71 | संदर्भ: AWS, सॉफ़्टवेयर इंजीनियरिंग नियम, तकनीकी दस्तावेश |
| `zettelkasten` | `10-Zettelkasten` | 17 | परमाणु नोट्स: AI आर्किटेक्चर, BM25, ज्ञान ग्राफ, Rust पैटर्न |

प्रत्येक वॉल्ट में स्वतंत्र:
- BM25 इंडेक्स (Tantivy)
- वेक्टर डेटाबेस (sqlite-vec)
- फ़ाइल फ़िंगरप्रिंट ट्रैकिंग (`index_meta.json`)
- कैश आइसोलेशन (अलग `OnceLock<Mutex<HashMap>>`)

---

## 7. Alcove कॉन्फ़िगरेशन प्रणाली

### वैश्विक: `~/.alcove/config.toml`

```toml
docs_root = "/path/to/Obsidian/SecondBrain/99-Archives/projects"

[core]
files = ["PRD.md", "ARCHITECTURE.md", "PROGRESS.md", "DECISIONS.md",
         "CONVENTIONS.md", "SECRETS_MAP.md", "DEBT.md"]

[team]
files = ["ENV_SETUP.md", "ONBOARDING.md", "DATA_MODEL.md", "SCHEMA.md",
         "DEPLOYMENT.md", "RUNBOOK.md", "PLAYBOOK.md", "MONITORING.md", ...]  # 19 फ़ाइलें

[public]
files = ["README.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md", ...]  # 15 फ़ाइलें

[embedding]
model = "MultilingualE5Small"
auto_download = true
enabled = true
```

### प्रति-प्रोजेक्ट: `alcove.toml`

वैश्विक डिफ़ॉल्ट को ओवरराइड करता है: `diagram_format`, `core_files`, `team_files`, `public_files`।

### नीति: `policy.toml`

परिभाषित करता है:
- `enforce` स्तर: `strict` | `warn` | `off`
- आवश्यक दस्तावेश अनुभाग शीर्षलेखों और न्यूनतम आइटम गणनाओं के साथ
- नामकरण परंपराएँ (`UPPER_SNAKE`, `lower_snake`, `kebab`, `free`)
- प्राथमिकता: प्रोजेक्ट > टीम > अंतर्निहित डिफ़ॉल्ट

---

## 8. उपयोग-मामला निर्णय मैट्रिक्स

| स्थिति | अनुशंसित टूल | तर्क |
|-----------|-----------------|------------|
| "प्रोडक्शन घटना से सीखा गया पाठ रिकॉर्ड करें" | **Episteme TK-*** | भविष्य के क्रॉस-रेफ़रेंस के लिए प्रासंगिक स्मेल/नियमों को स्वतः-लिंक करता है |
| "एक नए प्रोजेक्ट के लिए दस्तावेशीकरण शुरू करें" | **Alcove** `init_project` | 7 कोर टेम्पलेट स्वचालित रूप से जनरेट होते हैं |
| "जाँचें कि क्या कोई दस्तावेश पुराना है" | **Alcove** `lint_project` | WIP/TODO/DEPRECATED/पुरानी तारीखों को स्वचालित रूप से पहचानता है |
| "जानें कि टीम ने auth middleware के बारे में क्या तय किया" | **Alcove** `search_project_docs` | BM25 + वेक्टर के साथ संरचित DECISIONS.md खोजता है |
| "किसी मॉड्यूल में कोड स्मेल पहचानें" | **Episteme** `analyze_code` | पैटर्न/regex-आधारित स्मेल पहचान |
| "सुनिश्चित करें कि PRD में सभी आवश्यक अनुभाग हैं" | **Alcove** `validate_docs` | नीति-आधारित अनुभाग और आइटम गणना सत्यापन |
| "एक अंतर्दृष्टि को Strategy पैटर्न से लिंक करें" | **Episteme** `confirm_links` | कैननिकल एंटिटी के लिए `derives_from` किनारा बनाता है |
| "एजेंट पहुँच के लिए Obsidian नोट्स आयात करें" | **Alcove** `promote_document` | स्वतः प्रोजेक्ट पहचान के साथ doc-repo में आयात करता है |
| "SRP और Extract Class के बीच संबंध खोजें" | **Episteme** `find_path` | एंटिटी प्रकारों में मल्टी-हॉप ग्राफ ट्रैवर्सल |
| "प्रोजेक्ट दस्तावेशीकरण स्थिति बैकअप करें" | **Alcove** `backup_vault` | टाइमस्टैंप के साथ Git कमिट स्नैपशॉट |
| "सार्वजनिक repo में उजागर आंतरिक दस्तावेशों के लिए ऑडिट करें" | **Alcove** `audit_project` | निजी और सार्वजनिक दोनों स्थानों को स्कैन करता है |
| "कोड के लिए रैंक किए गए रिफैक्टरिंग सुझाव प्राप्त करें" | **Episteme** `suggest_refactorings` | कंपोजिट स्कोरिंग: गंभीरता x प्रयास x सिद्धांत संरेखण |

---

## 9. पूरक भूमिकाएँ

```
Episteme TK-*                     Alcove
"यहाँ कौन सा सार्वभौमिक सिद्धांत    "हमारी टीम ने इसके
 लागू होता है?"                     बारे में क्या तय किया?"

 क्षणिक अंतर्दृष्टि ←────────────→ संरचित निर्णय रिकॉर्ड
 कीवर्ड स्वतः-लिंकिंग               टेम्पलेट-आधारित स्कैफ़ोल्डिंग
 क्रॉस-लेयर ग्राफ ट्रैवर्सल        क्रॉस-प्रोजेक्ट दस्तावेश खोज
 कोड विश्लेषण → स्मेल पहचान        दस्तावेश विश्लेषण → पुरानापन पहचान
```

**जब दोनों सक्रिय हों**: Episteme सार्वभौमिक "क्यों" प्रदान करता है (नियम, पैटर्न), Alcove प्रोजेक्ट-विशिष्ट "हमने क्या तय किया" (ADRs, परंपराएँ) प्रदान करता है। एजेंटों को दोनों स्रोतों का हवाला देना चाहिए, जब टीम नियम सामान्य मार्गदर्शन से टकराते हैं तो Alcove को प्राथमिकता देनी चाहिए।

---

## 10. स्केल और प्रदर्शन

| मीट्रिक | Episteme TK-* | Alcove |
|--------|---------------|--------|
| **डिज़ाइन क्षमता** | सैकड़ों अंतर्दृष्टि | ~10,000 फ़ाइलें |
| **खोज विलंबता** | FTS5 तत्काल (इन-मेमोरी) | BM25 अवलोकन < 500ms |
| **Token दक्षता** | प्रति परिणाम एक अंतर्दृष्टि | टॉप-5 चंक ~1.5k tokens (grep के लिए ~8k) |
| **इंडेक्स पुनर्निर्माण** | आवश्यक नहीं (FTS5 ट्रिगर) | वृद्धिशील: केवल बदली हुई फ़ाइलें |
| **मॉडल आकार** | लागू नहीं (कनेक्ट नहीं) | 15MB (ArcticEmbedXS) से 2.3GB (BGE-M3) |

---

*यह भी देखें: [Alcove एकीकरण मार्गदर्शिका](./alcove-integration.md) उपयोग पैटर्न और वर्कफ़्लो उदाहरणों के लिए।*
