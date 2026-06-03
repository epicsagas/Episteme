<p align="center">
<img src="../assets/icon.png" alt="Episteme" width="60%" />
</p>

<p align="center"><sub>Episteme (συν ταγμα) — यूनानी भाषा में "संगठित प्रणाली" या "विवेक" का अर्थ</sub></p>

<p align="center">एक ऑफलाइन-फर्स्ट, सिंगल-बाइनरी ज्ञान ग्राफ जो डिज़ाइन पैटर्न, रिफैक्टरिंग तकनीकों और सॉफ्टवेयर नियमों को semantic संबंधों के माध्यम से जोड़ता है।<br><b>AI एजेंट्स के लिए सर्वप्रथम बनाया गया</b> — सॉफ्टवेयर इंजीनियरिंग विशेषज्ञता को सीधे Claude Code, Cursor और अन्य MCP-संगत उपकरणों में एकीकृत करें।</p>

<p align="center">Rust में लिखा गया · सिंगल बाइनरी · पूरी तरह ऑफलाइन</p>

<p align="center">
    <a href="https://github.com/epicsagas/Episteme/actions"><img src="https://img.shields.io/github/actions/workflow/status/epicsagas/Episteme/ci.yml?branch=main&label=CI" alt="CI" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.95+-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/episteme"><img src="https://img.shields.io/badge/crates.io-v0.1.0-orange.svg" alt="crates.io" /></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

<p align="center">
  <a href="../../README.md">English</a> |
  <a href="../ja/">日本語</a> |
  <a href="../ko/">한국어</a> |
  <a href="../de/">Deutsch</a> |
  <a href="../fr/">Français</a> |
  <a href="../zh-CN/">简体中文</a> |
  <a href="../zh-TW/">繁體中文</a> |
  <a href="../pt/">Português</a> |
  <a href="../es/">Español</a> |
  हिन्दी
</p>

---

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/features.png">
  <img src="../assets/features.png" align="center" width="100%" alt="Episteme Features Overview" />
</picture>

---

## त्वरित शुरुआत

### Claude Code

```
/plugin marketplace add epicsagas/plugins
/plugin install episteme@epicsagas
```

प्लगइन हुक `epis` बाइनरी को स्वचालित रूप से इंस्टॉल करता है। **नया सेशन शुरू करने से पहले** टर्मिनल में एक बार यह कमांड चलाएँ:

```bash
epis install   # GitHub Releases से नॉलेज ग्राफ डेटा डाउनलोड करता है
```

`epis install` नॉलेज ग्राफ डेटाबेस को इनिशियलाइज़ करता है और पोर्ट 58302 पर HTTP API सर्वर शुरू करता है। इसके बाद नया Claude Code सेशन शुरू करें और तैयार हैं।

अपडेट करें: `/plugin update episteme@epicsagas`

### Codex CLI

```bash
codex plugin marketplace add epicsagas/plugins
```

प्लगइन हुक `epis` बाइनरी को स्वचालित रूप से इंस्टॉल करता है। **नया सेशन शुरू करने से पहले** टर्मिनल में एक बार यह कमांड चलाएँ:

```bash
epis install   # GitHub Releases से नॉलेज ग्राफ डेटा डाउनलोड करता है
```

`epis install` नॉलेज ग्राफ डेटाबेस को इनिशियलाइज़ करता है और पोर्ट 58302 पर HTTP API सर्वर शुरू करता है। इसके बाद नया सेशन शुरू करें और तुरंत उपलब्ध है।

अपडेट करें: `codex plugin update episteme@epicsagas`

### अन्य टूल्स

```bash
epis install cursor       # Cursor IDE
epis install opencode     # OpenCode
epis install cline        # Cline
epis install --all        # सभी समर्थित टूल्स
```

### मैनुअल इंस्टॉल

| विधि | कमांड |
|------|--------|
| **Homebrew** | `brew install epicsagas/tap/episteme` |
| **Shell स्क्रिप्ट** | `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/epicsagas/Episteme/releases/latest/download/episteme-installer.sh \| sh` |
| **cargo** | `cargo binstall episteme` ⚡ या `cargo install episteme` |
| **Docker** | [विकल्प 3](#विकल्प-3-docker-rust-की-आवश्यकता-नहीं) देखें |

### सत्यापित करें

```bash
epis --version
epis stats
```

या Claude Code / Codex CLI के अंदर से:

```
/episteme verify
```

### 30 सेकंड में आज़माएँ

**विकल्प A — CLI:** इसे अपने प्रोजेक्ट की किसी भी फ़ाइल पर चलाएँ।

```bash
epis analyze src/domain/engine.rs
```

```
✓ 2 smells detected in src/domain/engine.rs

  SMELL-07 (Large Class) — RefactoringRanker, 743 lines
  → RF-018 Extract Class          priority 0.89  effort: medium
  → RF-001 Extract Method         priority 0.76  effort: small
  → Violates: LAW-001 Single Responsibility Principle

  SMELL-01 (Long Method) — rank_refactorings(), 58 lines
  → RF-001 Extract Method         priority 0.92  effort: small
  → Violates: LAW-001 SRP, LAW-004 DRY
```

**विकल्प B — Claude Code:** अपने प्रोजेक्ट की कोई भी फ़ाइल खोलें और स्वाभाविक रूप से पूछें।

```
Find code smells in this project and suggest refactorings.
```

Episteme स्वचालित रूप से सक्रिय होता है — किसी विशेष सिंटैक्स की आवश्यकता नहीं है। यह आपके विवरण को ज्ञान ग्राफ से मैप करता है और रैंक किए गए, उद्धृत करने योग्य परिणाम लौटाता है।

---

## Episteme क्यों?

LLM पहले से ही जानते हैं कि Strategy पैटर्न क्या है। वे SOLID सिद्धांतों को दोहरा सकते हैं, GoF पैटर्न्स की सूची बना सकते हैं, और कोड स्मेल्स को समझा सकते हैं। तो यह प्रोजेक्ट क्यों मौजूद है?

**अंतर ज्ञान नहीं है — यह संरचित, जुड़ी हुई तर्कशक्ति है।**

जब आप एक LLM से पूछते हैं "मैं God Object को कैसे ठीक करूँ?", तो यह एक उचित उत्तर देता है। लेकिन वह उत्तर वार्तालापों के बीच बदलता है, इसमें ट्रेसेबिलिटी की कमी होती है, और यह समस्या को उसके मूल कारणों या दीर्घकालिक प्रभावों से नहीं जोड़ता है। Episteme पृथक तथ्यों को एक ट्रैवर्सेबल ग्राफ में बदल देता है जहाँ हर सिफारिश आधारित, उद्धृत करने योग्य, और व्यापक डिज़ाइन परिदृश्य से जुड़ी हुई है।

### केवल LLM को अच्छी तरह प्रॉम्प्ट करने से यह कैसे भिन्न है?

| | अच्छी तरह से तैयार किया गया LLM प्रॉम्प्ट | Episteme + LLM |
|---|---|---|
| सक्रिय पहचान | केवल तभी जब उपयोगकर्ता सही प्रश्न पूछे | समस्या विवरणों पर स्वचालित रूप से सक्रिय होता है |
| टोकन दक्षता | लंबे स्पष्टीकरण + कई फॉलो-अप राउंड | एक टूल कॉल संरचित परिणाम लौटाता है |
| संबंध ट्रैवर्सल | अधिकतम एक हॉप, अक्सर काल्पनिक | बहु-हॉप ग्राफ ट्रैवर्सल, सत्यापित |
| क्रॉस-रेफरेंसिंग | मैन्युअल, त्रुटि-प्रवण | 201 semantic संबंधों के माध्यम से स्वचालित |
| संगतता | वार्तालापों के बीच भिन्न | हर बार समान संरचित उत्तर |
| उद्धरणीयता | "मुझे लगता है कि आपको Extract Class उपयोग करना चाहिए" | "Extract Class (RF-018), प्राथमिकता 0.89" |
| ऑफलाइन / एयर-गैप्ड | सर्वोत्तम परिणामों के लिए इंटरनेट आवश्यक | पूरी तरह स्थानीय, सिंगल बाइनरी |

### यह कब उपयोगी है?

<details>
<summary><b>1. जब आपका AI एजेंट समस्याओं की सक्रिय पहचान करे, पूछे जाने का इंतज़ार न करे</b></summary>

MCP एकीकरण समस्या विवरणों पर स्वचालित रूप से सक्रिय होता है। जब एक उपयोगकर्ता कहता है "यह क्लास बहुत कुछ करती है", तो एजेंट को God Object के बारे में पूछने की ज़रूरत नहीं है — Episteme शिकायत को `SMELL-03` से मैप करता है, रैंक किए गए रिफैक्टरिंग्स प्रस्तुत करता है, और उल्लंघन को मूल सिद्धांतों तक ट्रेस करता है। यह एक अस्पष्ट शिकायत को एक संरचित सुधार योजना में बदल देता है।
</details>

<details>
<summary><b>2. जब आप टोकन खपत कम करना चाहते हैं — स्पष्टीकरण पर नहीं खर्च करना चाहते</b></summary>

Episteme के बिना, एक LLM "मैं God Object को कैसे ठीक करूँ?" का उत्तर स्मेल को समझाकर, रिफैक्टरिंग्स की सूची बनाकर, SOLID सिद्धांतों का वर्णन करके, और प्रत्येक विकल्प से गुज़रकर देता है — प्रति उत्तर सैकड़ों टोकन। Episteme के साथ, एक MCP टूल कॉल `SMELL-03 → RF-018 (0.89) → LAW-001` लौटाता है। समान विशेषज्ञता टोकन बजट के एक अंश में।
</details>

<details>
<summary><b>3. जब आपको कोड विश्लेषण सुधार से जुड़ा हुआ चाहिए — केवल पहचान नहीं</b></summary>

SonarQube जैसे उपकरण स्मेल्स की पहचान करते हैं। LLM पैटर्न्स सुझा सकते हैं। Episteme दोनों करता है और उन्हें जोड़ता है: Long Method की पहचान करें → यह जो नियम तोड़ता है उन्हें ट्रेस करें → उन रिफैक्टरिंग्स को रैंक करें जो इसे हल करते हैं → दिखाएँ कि कौन से पैटर्न्स उन रिफैक्टरिंग्स को लागू करते हैं।
</details>

<details>
<summary><b>4. जब पृथक पैटर्न ज्ञान पर्याप्त नहीं है — आपको संबंधों की आवश्यकता है</b></summary>

Extract Method क्या करता है, यह जानना आधारभूत बात है। यह जानना कि यह Long Method (SMELL-01) को *हल करता है*, जो Single Responsibility (LAW-001) का *उल्लंघन करता है*, जिसे Facade Pattern (DP-012) *लागू करता है* — यह एक तर्क श्रृंखला है जिसे एक LLM स्वतंत्र रूप से विश्वसनीय ढंग से नहीं बना सकता। Episteme के 201 semantic संबंध AI एजेंट्स को इन पथों को नियतात्मक रूप से ट्रैवर्स करने देते हैं।
</details>

<details>
<summary><b>5. जब आप आर्किटेक्चर निर्णय ले रहे हैं और राय नहीं, साक्ष्य चाहिए</b></summary>

"क्या मुझे माइक्रोसर्विसेज़ का उपयोग करना चाहिए?" — Episteme प्रश्न को Conway's Law (LAW-017), SRP (LAW-001), और Strangler Fig पैटर्न (DP-026) से जोड़ता है, फिर दिखाता है कि वे कैसे संबंधित हैं। निर्णय इंजीनियरिंग नियमों तक ट्रेसेबल हो जाते हैं, ब्लॉग पोस्ट तक नहीं।
</details>

<details>
<summary><b>6. जब आपको संगत, उद्धृत करने योग्य इंजीनियरिंग सलाह चाहिए — काल्पनिक सिफारिशें नहीं</b></summary>

हर निष्कर्ष स्पष्ट एंटिटी IDs (`DP-005`, `RF-001`, `LAW-021`) को संदर्भित करता है। सिफारिशें प्राथमिकता स्कोर और प्रयास अनुमानों के साथ आती हैं। समान क्वेरी हमेशा समान संरचित उत्तर लौटाती है।
</details>

<details>
<summary><b>7. जब आप एक एयर-गैप्ड या प्रतिबंधित नेटवर्क में काम कर रहे हैं</b></summary>

Episteme पूरी तरह से ऑफलाइन चलता है: सिंगल बाइनरी, स्थानीय SQLite डेटाबेस, fastembed (ONNX Runtime) के माध्यम से स्थानीय एम्बेडिंग्स। कोई टेलीमेट्री नहीं, कोई फ़ोन-होम नहीं, कोई बाहरी API कॉल नहीं। आपका कोड और विश्लेषण परिणाम कभी भी आपकी मशीन से बाहर नहीं जाते।
</details>

---

## विशेषताएँ

| | विशेषता | यह क्यों महत्वपूर्ण है |
|--|---------|----------------------|
| 🧠 | **22 GoF डिज़ाइन पैटर्न** | वास्तविक उदाहरणों के साथ पूर्ण कैटलॉग |
| 🔧 | **66 रिफैक्टरिंग तकनीकें** | कोड नमूनों के साथ Fowler का कैटलॉग |
| ⚖️ | **56 सॉफ्टवेयर नियम और सिद्धांत** | SOLID, Conway का नियम, CAP प्रमेय आदि |
| 👃 | **17 कोड स्मेल प्रकार** | Long Method, God Object, Feature Envy आदि ¹ |
| 🔗 | **201 अर्थपूर्ण संबंध** | "हल करता है", "लागू करता है", "उल्लंघन करता है", "संबंधित है" |
| 🤖 | **9 MCP उपकरण + 4 एजेंट** | उच्च-निष्ठता AI एजेंट इंटरैक्शन, एजेंट-बीच हैंडऑफ |
| 🌐 | **HTTP API सर्वर** | पोर्ट 58302 पर REST API, इंस्टॉल पर स्वचालित रूप से शुरू |
| 🌍 | **10 भाषा समर्थन** | Python (AST), Java, TypeScript, Go, Rust, C++, C#, PHP, Ruby, Kotlin |
| 📊 | **नियतात्मक विश्लेषण** | AST-आधारित Python + रेगेक्स बहु-भाषा, हर बार एक ही परिणाम |
| 🏷️ | **उद्धरणीय ज्ञान** | प्रत्येक खोज स्पष्ट एंटिटी ID (`RF-001`, `LAW-021`) से जुड़ी है |
| 🌐 | **REST API (17 एंडपॉइंट)** | प्रमाणीकरण, दर सीमा, हेल्थ प्रोब, Prometheus मेट्रिक्स |
| 📦 | **एकल बाइनरी** | कोई रनटाइम नहीं, क्रॉस-प्लेटफॉर्म (macOS, Linux, Windows) |
| 🔌 | **स्थानीय एम्बेडिंग्स** | fastembed (ONNX Runtime), शून्य-कॉन्फ़िग अर्थपूर्ण खोज |
| 🐳 | **Docker समर्थन** | हेल्थ चेक के साथ मल्टी-स्टेज बिल्ड |

> ¹ Duplicate Code (SMELL-13) और Shotgun Surgery (SMELL-09) को बहु-फ़ाइल संदर्भ की आवश्यकता होती है और एकल-फ़ाइल मोड में छोड़ दिया जाता है।

---

## इंस्टॉलेशन

### विकल्प 1: cargo-binstall (अनुशंसित)

```bash
cargo binstall episteme    # प्री-बिल्ट बाइनरी डाउनलोड करता है — संकलन की आवश्यकता नहीं
epis install cursor        # डेटा सीड करता है + API सर्वर शुरू करता है + एजेंट्स इंस्टॉल करता है
```

यदि cargo-binstall नहीं है: `cargo install cargo-binstall`

> `epis install` के बाद, HTTP API सर्वर स्वचालित रूप से पोर्ट 58302 पर शुरू होता है। MCP अभी भी उपलब्ध है -- मैन्युअल सेटअप के लिए `registry/mcp.json` देखें।

### विकल्प 2: सोर्स से

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme && cargo build --release
```

फिर अपने प्लेटफ़ॉर्म के लिए बाइनरी चलाएँ:

| प्लेटफ़ॉर्म | कमांड |
|----------|---------|
| **macOS / Linux** | `./target/release/epis install --local cursor` |
| **Windows** | `.\target\release\episteme.exe install --local cursor` |

### विकल्प 3: Docker (Rust की आवश्यकता नहीं)

```bash
docker-compose up -d
```

अपने MCP कॉन्फ़िग फ़ाइल में जोड़ें:

| टूल | कॉन्फ़िग फ़ाइल पथ |
|------|-----------------|
| Claude Code | `~/.claude.json` |
| Cursor | `.cursor/mcp.json` |
| VS Code (Copilot) | `.vscode/mcp.json` |

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

### विकल्प 4: प्री-बिल्ट बाइनरीज़ (Rust की आवश्यकता नहीं)

[GitHub Releases](https://github.com/epicsagas/Episteme/releases) से अपने प्लेटफ़ॉर्म के लिए नवीनतम बाइनरी डाउनलोड करें:

| प्लेटफ़ॉर्म | फ़ाइल |
|----------|------|
| **macOS** (Apple Silicon) | `episteme-aarch64-apple-darwin.tar.xz` |
| **Linux** (x86_64) | `episteme-x86_64-unknown-linux-gnu.tar.xz` |

```bash
# macOS / Linux
tar xzf episteme-*.tar.gz
sudo mv episteme /usr/local/bin/

# Windows — ZIP निकालें और episteme.exe को अपने PATH में जोड़ें
```

फिर इंस्टॉल करें:
```bash
epis install cursor
```

### सत्यापित करें

```bash
epis --version
epis stats
epis explore "strategy pattern"    # ज्ञान ग्राफ का अन्वेषण करें
```

या Claude Code / Codex CLI के अंदर से:

```
/episteme verify
```

---

## HTTP API एंडपॉइंट

> Episteme पोर्ट 58302 पर हमेशा चलने वाले HTTP API सर्वर के रूप में काम करता है। Skills और एजेंट MCP टूल्स के बजाय `curl http://localhost:58302/...` का उपयोग करते हैं। MCP मैन्युअल सेटअप के लिए अभी भी उपलब्ध है -- `registry/mcp.json` देखें।

### API एंडपॉइंट

#### नॉलेज ग्राफ

| विधि | एंडपॉइंट | उद्देश्य |
|------|----------|---------|
| **GET** | `/health` | स्वास्थ्य जाँच |
| **GET** | `/search?q=...` | नॉलेज ग्राफ खोजें |
| **GET** | `/graph/{id}` | ID द्वारा एंटिटी प्राप्त करें |
| **GET** | `/graph/{id}/neighbors` | संबंधित एंटिटीज प्राप्त करें |
| **POST** | `/graph/path` | दो एंटिटीज के बीच पाथ खोजें |

#### कोड विश्लेषण

| विधि | एंडपॉइंट | उद्देश्य |
|------|----------|---------|
| **POST** | `/analyze` | कोड स्मेल का पता लगाएं |
| **POST** | `/refactor` | रिफैक्टरिंग सुझाव |

#### अंतर्निहित ज्ञान

| विधि | एंडपॉइंट | उद्देश्य |
|------|----------|---------|
| **POST** | `/insights` | टीम इनसाइट जोड़ें |

### 9 MCP टूल्स (लेगसी)

#### कैननिकल नॉलेज (6 टूल्स)

| टूल | उद्देश्य | उपयोग उदाहरण |
|------|---------|-------------|
| **`search_knowledge`** | सभी एंटिटीज में सिमेंटिक सर्च | "रीट्री लॉजिक के लिए पैटर्न खोजें" |
| **`get_entity`** | ID द्वारा विशिष्ट एंटिटी का विवरण प्राप्त करें | "Strategy Pattern (DP-023) समझाएं" |
| **`get_neighbors`** | संबंधित एंटिटीज का अन्वेषण करें | "Long Method को कौन से रिफैक्टरिंग हल करते हैं?" |
| **`find_path`** | दो एंटिटीज के बीच कनेक्शन खोजें | "SRP और Extract Class कैसे संबंधित हैं?" |
| **`analyze_code`** | regex/AST विश्लेषण द्वारा कोड स्मेल का पता लगाएं | "इस पेमेंट वैलिडेशन कोड की समीक्षा करें" |
| **`suggest_refactorings`** | रैंक किए गए रिफैक्टरिंग सुझाव | "मुझे इस क्लास में क्या रिफैक्टर करना चाहिए?" |

#### अंतर्निहित ज्ञान (3 टूल्स)

| टूल | उद्देश्य | उपयोग उदाहरण |
|------|---------|-------------|
| **`add_insight`** | टीम निर्णय, सीखी गई सीखें रिकॉर्ड करें | "पोलिंग के बजाय इवेंट-चालित चुनने का कारण" |
| **`search_insights`** | पिछला टीम ज्ञान खोजें | "हमने ऑथ मिडलवेयर के बारे में क्या तय किया?" |
| **`confirm_links`** | स्वतः पता लगाए गए कैननिकल एंटिटी लिंक को मान्य करें | TK-001 का SMELL-03 से संबंध होने की पुष्टि करें |

Episteme अंतर्निहित ज्ञान को एक अलग डेटाबेस (`~/.episteme/user_knowledge.db`) में संग्रहीत करता है और रनटाइम पर एक कंपोजिट लेयर के माध्यम से कैननिकल ग्राफ के साथ विलय करता है। टीम अंतर्दृष्टि स्वचालित रूप से पैटर्न, नियम और स्मेल से जुड़ जाती है — अनुभव को नेविगेट करने योग्य ज्ञान में बदलना।

पूर्ण डिज़ाइन के लिए [अंतर्निहित ज्ञान आर्किटेक्चर](./tacit-knowledge.md) देखें।

### 4 विशेषज्ञ एजेंट्स (जुड़ा हुआ नेटवर्क)

एजेंट्स एक साथ मिलकर काम करते हैं — प्रत्येक विश्लेषण **अगले चरण** विकल्पों के साथ समाप्त होता है जो अन्य एजेंट्स को हस्तांतरित करते हैं।

| एजेंट | कब उपयोग करें | मुख्य क्षमता | हस्तांतरित करता है |
|-------|-------------|----------------|--------------|
| **`code-reviewer`** | कोड स्मेल्स, SOLID उल्लंघन | कॉज़ेशन विश्लेषण (मूल कारण → दीर्घकालिक लक्षण) | advisor, architecture-analyst, refactoring-expert |
| **`episteme-advisor`** | इंजीनियरिंग निर्णय, ट्रेड-ऑफ़ | बहु-एंटिटी ट्रेड-ऑफ़ श्रृंखलाएँ कार्य योजनाओं के साथ | code-reviewer, architecture-analyst, researcher |
| **`episteme-researcher`** | ज्ञान ग्राफ अन्वेषण | पैटर्न्स, नियमों, स्मेल्स के बीच संबंध मानचित्र | advisor, code-reviewer |
| **`architecture-analyst`** | नियमों के विरुद्ध आर्किटेक्चर मूल्यांकन | जोखिम-भारित मूल्यांकन के साथ अनुपालन स्कोरिंग | advisor, code-reviewer, researcher |

**वर्कफ़्लो उदाहरण**: `code-reviewer` God Object का पता लगाता है → कॉज़ेशन को 3 दीर्घकालिक स्मेल्स तक ट्रेस करता है → "RF-018 लागू करें" (→ refactoring-expert) या "मूल कारण का गहन विश्लेषण" (→ episteme-advisor) या "आर्किटेक्चर जाँच" (→ architecture-analyst) प्रस्तुत करता है।

[पूर्ण MCP एकीकरण गाइड](./mcp-integration-guide.md)

---

## CLI उपयोग

```bash
# स्मेल्स के लिए कोड विश्लेषण करें
epis analyze my_code.py --language python --json
episteme infer my_code.py

# ज्ञान ग्राफ का अन्वेषण करें
epis explore "strategy pattern"
epis graph path DP-005 RF-001   # उदाहरण: Factory Method → Extract Method

# RAG इंडेक्स बनाएँ
epis build

# सर्वर प्रारंभ करें
epis api              # :58302 पर REST API
episteme mcp --http       # :43175 पर MCP सर्वर (लेगसी)
episteme web --port 8080  # Web UI (इंटरैक्टिव ग्राफ एक्सप्लोरर)

# वितरण पैकेजिंग
episteme dist --out-dir release/
```

---

## दस्तावेज़ीकरण

| दस्तावेज़ | विवरण |
|----------|-------------|
| [त्वरित शुरुआत](./QUICKSTART.md) | चरण-दर-चरण सेटअप, पहला रन, समस्या निवारण |
| [MCP एकीकरण गाइड](./mcp-integration-guide.md) | टूल संदर्भ, एजेंट उदाहरण, वार्तालाप प्रवाह |
| [अंतर्निहित ज्ञान आर्किटेक्चर](./tacit-knowledge.md) | दो-डेटाबेस डिज़ाइन, अंतर्दृष्टि जीवनचक्र, स्कीमा |
| [Alcove पारिस्थितिकी तंत्र तुलना](./alcove-ecosystem.md) | स्टोरेज मॉडल, खोज क्षमताएँ, उपयोग-केस मैट्रिक्स |
| [Alcove एकीकरण गाइड](./alcove-integration.md) | डुअल-कॉन्टेक्स्ट वर्कफ़्लो, सेटअप, सर्वोत्तम अभ्यास |
| [API संदर्भ](./api.md) | REST एंडपॉइंट्स, प्रमाणीकरण, उदाहरण |
| [वितरण](./distribution.md) | रिलीज़ पैकेजिंग और परिनियोजन |
| [विकास और योगदान](./DEVELOPMENT.md) | आर्किटेक्चर, योगदान कैसे करें |
| [बदलाव सूची](./CHANGELOG.md) | रिलीज़ इतिहास और संस्करण नोट्स |

---

## कॉन्फ़िगरेशन

### पर्यावरण चर

```bash
# डेटा स्थान
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# API सर्वर
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=58302
EPISTEME_API_KEY=your-secret-key

# MCP सर्वर
EPISTEME_MCP_HOST=127.0.0.1
EPISTEME_MCP_PORT=43175
```

---

## समस्या निवारण

**इंस्टॉल के बाद `episteme` कमांड नहीं मिल रहा है**

| प्लेटफ़ॉर्म | समाधान |
|----------|-----|
| **macOS / Linux** | `export PATH="$HOME/.cargo/bin:$PATH"` — स्थायी बनाने के लिए `~/.bashrc` या `~/.zshrc` में जोड़ें |
| **Windows** | `%USERPROFILE%\.cargo\bin` को अपने सिस्टम PATH में जोड़ें, या एक नया टर्मिनल खोलें |

**MCP टूल्स Claude Code / Cursor में नहीं दिख रहे हैं**

`epis install` के बाद HTTP API सर्वर स्वचालित रूप से पोर्ट 58302 पर शुरू होता है। Skills `curl http://localhost:58302/...` का उपयोग करके Episteme के साथ इंटरैक्ट करते हैं। MCP मैन्युअल सेटअप के लिए अभी भी उपलब्ध है -- `registry/mcp.json` देखें।

**पोर्ट पहले से उपयोग में है**
```bash
epis api --port 58303   # एक भिन्न पोर्ट का उपयोग करें
```

**पहली बार शुरू करने में धीमापन**

Episteme पहली बार चलने पर एक स्थानीय एम्बेडिंग इंडेक्स बनाता है। इसमें 30–60 सेकंड लगते हैं और यह एक बार की लागत है। बाद के शुरू होने तुरंत होते हैं।

**`cargo install` के दौरान संकलन त्रुटियाँ**

सुनिश्चित करें कि Rust 1.95+ इंस्टॉल है:
```bash
rustup update stable
rustup show   # सक्रिय टूलचेन की पुष्टि करें
```

> अधिक सहायता: [QUICKSTART.md समस्या निवारण अनुभाग](../../QUICKSTART.md#troubleshooting) · [एक इश्यू खोलें](https://github.com/epicsagas/Episteme/issues)

---

## रोडमैप

**जारी किया गया**
- [x] `epis install` — GitHub Releases से एक कमांड में डेटा सेटअप
- [x] Homebrew tap (`epicsagas/tap/episteme`) — macOS Apple Silicon + Linux x86_64
- [x] Claude Code & Codex CLI प्लगइन मार्केटप्लेस सपोर्ट
- [x] README अनुवाद — 9 भाषाएँ (ko, ja, zh-CN, zh-TW, de, fr, es, pt, hi)

**योजनाबद्ध**
- [ ] **क्रॉस-प्लेटफ़ॉर्म बिल्ड** — Intel macOS, Windows, Linux ARM64 के समर्थन के लिए `fastembed` → `candle` (Pure Rust) में माइग्रेशन ([#32](https://github.com/epicsagas/Episteme/issues/32))
- [ ] **कस्टम एंटिटीज़** — टीम-विशिष्ट पैटर्न्स/स्मेल्स जोड़ें
- [ ] **बहुभाषी मेटाडेटा** — CJK भाषाओं में एंटिटी शीर्षक और सारांश
- [ ] **इंटरैक्टिव ट्यूटोरियल** — MCP टूल्स के लिए इन-ऐप गाइडेड टूर
- [ ] **टीम मेट्रिक्स** — संगठन में पैटर्न उपयोग का समेकन

---

## योगदान

योगदान का स्वागत है! आर्किटेक्चर अवलोकन और योगदान गाइड के लिए [DEVELOPMENT.md](./DEVELOPMENT.md) देखें।

```bash
# टेस्ट चलाएँ
cargo test

# लिंट
cargo clippy -- -D warnings

# फ़ॉर्मेट
cargo fmt
```

प्रश्न हैं? [एक चर्चा खोलें](https://github.com/epicsagas/Episteme/discussions) या [एक इश्यू दर्ज करें](https://github.com/epicsagas/Episteme/issues)।

---

## लाइसेंस

Apache 2.0 — विवरण के लिए [LICENSE](../../LICENSE) देखें।
