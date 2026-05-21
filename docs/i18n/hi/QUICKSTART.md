# Episteme — त्वरित आरंभ मार्गदर्शिका

2 मिनट से भी कम समय में Episteme के साथ आरंभ करें।

---

## आवश्यकताएँ

- **Rust 1.95+** (संस्करण 2024 आवश्यक) — [rustup के माध्यम से स्थापित करें](https://rustup.rs)
- इंटरनेट कनेक्शन (प्रारंभिक डेटा डाउनलोड के लिए)

---

## विकल्प 1: AI टूल एकीकरण (अनुशंसित)

**यह विकल्प उपयुक्त है:** Claude Code, Cursor, Codex, Gemini उपयोगकर्ताओं के लिए

```bash
# 1. Episteme स्थापित करें
cargo install --git https://github.com/epicsagas/Episteme

# 2. अपने AI टूल में स्थापित करें (डेटा डाउनलोड करता है, MCP कॉन्फ़िगर करता है, एजेंट कॉपी करता है)
epis install claude      # Claude Code
epis install cursor      # Cursor
epis install codex       # OpenAI Codex
epis install gemini      # Antigravity
epis install all         # सभी टूल एक साथ
```

> यदि `epis install claude` डेटा डाउनलोड करने में विफल होता है, तो नीचे दिए गए स्रोत स्थापना विकल्प का उपयोग करें।

**बस इतना ही।** अपना AI टूल पुनः आरंभ करें और Episteme सक्रिय है।

---

## विकल्प 2: Docker (Rust आवश्यक नहीं)

```bash
docker-compose up -d

# पहुँच
# API:       http://localhost:8000
# Health:    http://localhost:8000/health
```

Docker के माध्यम से MCP एकीकरण के लिए, अपने MCP कॉन्फ़िग में जोड़ें:
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

## विकल्प 3: स्रोत से

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme

# बिल्ड करें
cargo build --release

# डेटा सीड करें और वेक्टर DB बनाएं (बिल्ड स्वचालित रूप से चलता है)
./target/release/epis install --local
```

---

## ग्राफ विज़ुअलाइज़ेशन

Episteme में एक संवादात्मक D3-force ग्राफ व्यूअर शामिल है:

```bash
episteme web               # डिफ़ॉल्ट: http://localhost:8080
episteme web --port 9001   # कस्टम पोर्ट
episteme web --host 0.0.0.0 --port 8080  # नेटवर्क पर एक्सपोज़ करें
```

---

## सामान्य कमांड

```bash
# स्मेल के लिए कोड विश्लेषण करें
epis analyze my_code.py --language python
epis analyze my_code.py --json

# रिफैक्टरिंग सुझाव प्राप्त करें
episteme infer my_code.py --top-k 5

# ज्ञान ग्राफ का अन्वेषण करें
epis explore "strategy pattern"
epis graph path DP-005 RF-001

# सर्वर प्रारंभ करें
epis api              # REST API :8000 पर
episteme mcp --http       # MCP सर्वर :43175 पर
episteme web --port 8080  # Web UI

# बैकग्राउंड MCP डेमन (HTTP प्रॉक्सी)
epis service start
epis service status
epis service stop

# रिलीज़ आर्काइव बनाएं
episteme dist --out-dir release
```

---

## समस्या निवारण

### "Database not found"
```bash
epis install claude   # डेटा आर्काइव पुनः डाउनलोड करें
# या
epis install --local
```

### "Port already in use"
```bash
episteme web --port 9001
epis api --port 9000
```

---

## अगले चरण

- **[README](../../README.md)** — संपूर्ण सुविधा अवलोकन और आर्किटेक्चर
- **[MCP एकीकरण मार्गदर्शिका](./mcp-integration-guide.md)** — टूल संदर्भ और एजेंट उदाहरण
- **[API संदर्भ](./api.md)** — REST एंडपॉइंट
