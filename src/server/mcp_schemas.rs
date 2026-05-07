use serde_json::{Value, json};

pub const PROTOCOL_VERSION: &str = "2024-11-05";
pub const SERVER_NAME: &str = "syntagma";
pub const SERVER_VERSION: &str = "0.1.0";
pub const MAX_CODE_BYTES: usize = 500_000;

pub fn server_info() -> Value {
    json!({
        "name": SERVER_NAME,
        "version": SERVER_VERSION,
    })
}

pub fn capabilities() -> Value {
    json!({
        "tools": { "listChanged": false },
        "resources": { "subscribe": false, "listChanged": false },
    })
}

pub fn server_instructions() -> String {
    "\
Syntagma is a software engineering knowledge graph. Use it to AUTOMATICALLY ground your answers \
in proven patterns, laws, and refactoring techniques — even when the user doesn't know those terms.

CRITICAL: Auto-trigger on PROBLEM DESCRIPTIONS, not just named concepts. Users describe symptoms, \
not diagnoses. You must translate their informal problem into knowledge graph queries.

Auto-trigger rules — call Syntagma tools WITHOUT waiting for the user to mention them:

## Code problems (auto-translate complaint -> knowledge graph query)

User says / does -> What to call
\"this class does too much\" / file > 300 lines -> search_knowledge(\"god class large class single responsibility\")
\"this function is too long\" -> search_knowledge(\"long method extract method\")
\"code is too complex\" / hard to follow -> search_knowledge(\"complexity smell cognitive overload\")
\"calling DB directly in business logic\" -> search_knowledge(\"coupling persistence repository data access layer\")
\"hard to test\" / can't write unit tests -> search_knowledge(\"testability dependency injection mockability\")
\"copy-pasted this\" / duplicated logic -> search_knowledge(\"duplicated code clone smell\")
\"changing X breaks Y\" / change ripple -> search_knowledge(\"brittle coupling change propagation rigidity\")
\"adding a new type means touching everywhere\" / growing switch -> search_knowledge(\"open closed principle strategy polymorphism\")
\"is this thread-safe?\" / concurrency concerns -> search_knowledge(\"thread safety race condition shared mutable state\")
\"API is slow\" / performance issues -> search_knowledge(\"N+1 query lazy loading caching performance\")
User shares code for review -> analyze_code(code) then suggest_refactorings(code)
User wants to refactor or improve code -> suggest_refactorings(code)

## Architecture discussions (auto-trigger on trade-off questions)

\"microservices vs monolith\" / how to split -> search_knowledge(\"monolith microservice decomposition bounded context\")
\"is this architecture okay?\" / architecture review -> search_knowledge(\"layered architecture coupling cohesion separation concerns\")
\"where should this go?\" / code placement -> search_knowledge(\"responsibility assignment package structure\")
Team/org structure affects code -> search_knowledge(\"Conway law organizational structure architecture\")

## Concept exploration (when user names something or asks follow-up)

Entity ID mentioned (DP-xxx, LAW-xxx, RF-xxx, SMELL-xxx) -> get_entity(id)
\"how does X relate to Y\" -> find_path or get_neighbors
\"tell me more\" about a previous result -> get_entity for full details, get_neighbors for connected concepts

## Always

- Translate informal language into technical queries. User says \"it's a tangled mess\", you search \"coupling tangled dependency\".
- After search_knowledge, present the named concept and explain it in the user's own language.
- Cite entity IDs (DP-005, LAW-003) in responses."
        .to_owned()
}

const DESC_SEARCH: &str = "Hybrid search over the Syntagma knowledge base. Use when the user asks about design patterns, engineering laws, code smells, refactorings, or any software engineering concept. Examples: \"what is Strategy pattern\", \"laws about coupling\", \"find smells related to God Class\", \"relevant principles for microservices\".";
const DESC_GET_ENTITY: &str = "Get detailed information about a knowledge graph entity by ID. Use when an entity ID is mentioned (DP-xxx, LAW-xxx, RF-xxx, SMELL-xxx) or the user asks \"tell me more about\" a previously mentioned concept.";
const DESC_GET_NEIGHBORS: &str = "Get entities related to a given entity in the knowledge graph. Use to explore what patterns complement a law, what refactorings solve a smell, or how concepts connect. Follow up after search_knowledge or get_entity.";
const DESC_FIND_PATH: &str = "Find the shortest path between two entities in the knowledge graph. Use when the user asks how two concepts relate or connect, e.g. \"how does Strategy relate to Open/Closed?\", \"what connects God Class and Single Responsibility?\".";
const DESC_ANALYZE_CODE: &str = "Detect code smells in source code. Use when the user shares code and asks for review, feedback, problems, or improvements. Supports Python, Java, Go, Rust, TypeScript, C++, C#, Kotlin, PHP, and Ruby.";
const DESC_SUGGEST_REFACTORINGS: &str = "Get ranked refactoring suggestions for source code. Uses composite scoring (severity, effort, principle alignment, usage) to prioritize suggestions. Supports Python, Java, Go, Rust, TypeScript, C++, C#, Kotlin, PHP, and Ruby.";

pub fn tool_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": "search_knowledge",
            "description": DESC_SEARCH,
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural-language search query.",
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum results to return.",
                        "minimum": 1,
                        "maximum": 20,
                        "default": 5,
                    },
                    "entity_type": {
                        "type": "string",
                        "description": "Restrict results to this entity type.",
                        "enum": ["pattern", "refactoring", "law", "smell"],
                    },
                },
                "required": ["query"],
            },
        }),
        json!({
            "name": "get_entity",
            "description": DESC_GET_ENTITY,
            "inputSchema": {
                "type": "object",
                "properties": {
                    "entity_id": {
                        "type": "string",
                        "description": "Entity ID, e.g. DP-005, SMELL-01, RF-001.",
                    },
                    "detail_level": {
                        "type": "string",
                        "description": "How much detail to include.",
                        "enum": ["minimal", "summary", "detailed", "full"],
                        "default": "summary",
                    },
                },
                "required": ["entity_id"],
            },
        }),
        json!({
            "name": "get_neighbors",
            "description": DESC_GET_NEIGHBORS,
            "inputSchema": {
                "type": "object",
                "properties": {
                    "entity_id": {
                        "type": "string",
                        "description": "Source entity ID.",
                    },
                    "relation_type": {
                        "type": "string",
                        "description": "Filter by relation type.",
                        "enum": [
                            "solves",
                            "solved_by",
                            "enforces",
                            "violates",
                            "related_to",
                        ],
                    },
                },
                "required": ["entity_id"],
            },
        }),
        json!({
            "name": "find_path",
            "description": DESC_FIND_PATH,
            "inputSchema": {
                "type": "object",
                "properties": {
                    "from_id": {
                        "type": "string",
                        "description": "Starting entity ID.",
                    },
                    "to_id": {
                        "type": "string",
                        "description": "Target entity ID.",
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum traversal depth.",
                        "minimum": 1,
                        "maximum": 10,
                        "default": 5,
                    },
                },
                "required": ["from_id", "to_id"],
            },
        }),
        json!({
            "name": "analyze_code",
            "description": DESC_ANALYZE_CODE,
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Source code to analyze.",
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language (python, java, go, rust, typescript, cpp, csharp, kotlin, php, ruby).",
                        "default": "python",
                    },
                },
                "required": ["code"],
            },
        }),
        json!({
            "name": "suggest_refactorings",
            "description": DESC_SUGGEST_REFACTORINGS,
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Source code to analyze.",
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language (python, java, go, rust, typescript, cpp, csharp, kotlin, php, ruby).",
                        "default": "python",
                    },
                    "top_k": {
                        "type": "integer",
                        "description": "Number of suggestions per detected smell.",
                        "minimum": 1,
                        "maximum": 10,
                        "default": 3,
                    },
                },
                "required": ["code"],
            },
        }),
    ]
}

pub fn resource_schemas() -> Vec<Value> {
    vec![
        json!({
            "uri": "syntagma://stats",
            "name": "Knowledge Graph Statistics",
            "description": "Aggregate statistics about the knowledge graph.",
            "mimeType": "application/json",
        }),
        json!({
            "uri": "syntagma://categories",
            "name": "Categories and Entity Types",
            "description": "Category and entity type listing.",
            "mimeType": "application/json",
        }),
        json!({
            "uri": "syntagma://contradictions",
            "name": "Contradictions",
            "description": "Entities with conflicting relations.",
            "mimeType": "application/json",
        }),
    ]
}
