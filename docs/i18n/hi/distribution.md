# वितरण पैकेजिंग (Rust CLI)

यह मार्गदर्शिका बताती है कि Rust CLI का उपयोग करके अन्य उपयोगकर्ताओं के लिए रिलीज़ डेटा आर्काइव कैसे बनाएँ।

## कमांड

```bash
episteme dist
```

## `episteme dist` में क्या शामिल है
- `raw/`
- `meta/`
- `data/` (यदि मौजूद है)
- `db/episteme.db` (एम्बेडिंग DB)

आउटपुट आर्काइव:
- `dist/episteme-data-<version>.tar.gz`

## स्वचालित बिल्ड व्यवहार
- यदि `~/.episteme/db/episteme.db` अनुपस्थित है, तो `episteme dist` स्वचालित रूप से पहले `epis build` चलाता है।
- बनाया गया DB प्रोजेक्ट-स्थानीय `db/` निर्देशिका में भी कॉपी किया जाता है ताकि आर्काइव में शामिल किया जा सके।
- `epis install --local` आर्काइव (या स्रोत ट्री फ़ॉलबैक) से डेटा सीड करता है और स्वचालित रूप से RAG इंडेक्स `~/.episteme/` में बनाता है।

## विकल्प
- `--out-dir <DIR>`: आउटपुट निर्देशिका (डिफ़ॉल्ट: `dist`)
- `--no-db`: DB समावेशन छोड़ें
- `--skip-build`: यदि DB अनुपस्थित है तो स्वचालित रूप से बिल्ड न करें

उदाहरण:

```bash
# dist/ में डिफ़ॉल्ट पैकेजिंग
episteme dist

# कस्टम आउटपुट निर्देशिका
episteme dist --out-dir release

# केवल मेटाडेटा पैकेज करें (DB के बिना)
episteme dist --no-db

# सख्त मोड: यदि DB अनुपस्थित है तो विफल करें
episteme dist --skip-build
```

## सत्यापन
आर्काइव जनरेट करने के बाद, संरचना सत्यापित करें:

```bash
tar -tzf dist/episteme-data-*.tar.gz | head -n 30
```

आपको निम्नलिखित पथों के अंतर्गत प्रविष्टियाँ दिखनी चाहिए:
- `episteme-data-<version>/raw/...`
- `episteme-data-<version>/meta/...`
- `episteme-data-<version>/db/episteme.db` (`--no-db` का उपयोग न करने पर)
