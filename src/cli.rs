//! CLI enum definitions for clap.

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze a source file for code smells
    Analyze {
        /// Path to the source file
        file: String,
        /// Programming language (auto-detected from extension if omitted)
        #[arg(long)]
        language: Option<String>,
        /// Output results as JSON
        #[arg(long)]
        json: bool,
        /// Minimum confidence threshold (0.0 - 1.0)
        #[arg(long, default_value_t = 0.0)]
        min_confidence: f64,
    },
    /// Analyze code smells and suggest refactorings
    Infer {
        /// Path to the source file
        file: String,
        /// Programming language (auto-detected from extension if omitted)
        #[arg(long)]
        language: Option<String>,
        /// Number of refactoring suggestions per smell
        #[arg(long, default_value_t = 3)]
        top_k: usize,
        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },
    /// Explore the knowledge graph by keyword search or interactive REPL
    Explore {
        /// Search query (omit for interactive REPL)
        #[arg(required = false)]
        query: Option<String>,
        /// Maximum number of results
        #[arg(long, default_value_t = 5)]
        limit: usize,
        /// Filter by entity type (pattern, refactoring, law, smell)
        #[arg(long)]
        entity_type: Option<String>,
        /// Launch interactive REPL
        #[arg(long)]
        interactive: bool,
    },
    /// Query the knowledge graph structure
    Graph {
        #[command(subcommand)]
        sub: GraphCommands,
    },
    /// Build the RAG index from raw data
    Build {
        /// Override data directory
        #[arg(long)]
        data_dir: Option<String>,
        /// Force GPU mode (Python CLI parity flag)
        #[arg(long)]
        gpu: bool,
        /// Force CPU mode (Python CLI parity flag)
        #[arg(long)]
        no_gpu: bool,
        /// Embedding batch size
        #[arg(long, default_value_t = 64)]
        batch_size: usize,
        /// Delete existing database before building
        #[arg(long)]
        rebuild: bool,
        /// Print statistics after building
        #[arg(long)]
        stats: bool,
    },
    /// Create release data archive (raw/meta/data + db)
    Dist {
        /// Output directory for generated archive
        #[arg(long, default_value_t = String::from("dist"))]
        out_dir: String,
        /// Skip embedding database file (~/.syntagma/db/syntagma.db)
        #[arg(long)]
        no_db: bool,
        /// Do not auto-build DB when missing
        #[arg(long)]
        skip_build: bool,
    },
    /// Start the REST API server
    Api {
        /// Bind address
        #[arg(long, default_value_t = String::from("0.0.0.0"))]
        host: String,
        /// Bind port
        #[arg(long, default_value_t = 8000)]
        port: u16,
    },
    /// Manage the MCP HTTP server daemon
    Service {
        #[command(subcommand)]
        sub: ServiceCommands,
    },
    /// Start the MCP server (stdio or HTTP)
    Mcp {
        /// Serve MCP over HTTP instead of stdio
        #[arg(long)]
        http: bool,
        /// Bind host for HTTP mode
        #[arg(long, default_value_t = String::from("127.0.0.1"))]
        host: String,
        /// Bind port for HTTP mode
        #[arg(long, default_value_t = 43175)]
        port: u16,
    },
    /// Telemetry consent management
    Telemetry {
        /// Action: on | off | status
        #[arg(default_value = "status")]
        action: String,
    },
    /// Print knowledge graph statistics
    Stats,
    /// AI-assisted workflow hooks (ground / sniff / audit)
    Hooks {
        #[command(subcommand)]
        sub: HooksCommands,
    },
    /// Start the web interface
    Web {
        /// Bind address
        #[arg(long, default_value_t = String::from("0.0.0.0"))]
        host: String,
        /// Bind port
        #[arg(long, default_value_t = 8080)]
        port: u16,
    },
    /// Install Syntagma into AI tools
    Install {
        /// Tools to install (claude, cursor, codex, gemini, opencode, cline, all)
        #[arg(required = false)]
        tools: Vec<String>,
        /// Install for all supported tools (non-interactive)
        #[arg(long)]
        all: bool,
        /// Preview without making changes
        #[arg(long)]
        dry_run: bool,
        /// Seed data from local source (dist/ archive first, then raw/meta/ fallback)
        #[arg(long)]
        local: bool,
    },
}

#[derive(Subcommand)]
pub enum GraphCommands {
    /// Look up a single entity by ID
    Entity {
        /// Entity ID (e.g. DP-005, SMELL-01, RF-001, LAW-001)
        id: String,
    },
    /// Get neighbors of an entity
    Neighbors {
        /// Entity ID
        id: String,
        /// Filter by relation type
        #[arg(long)]
        relation_type: Option<String>,
    },
    /// Find the shortest path between two entities
    Path {
        /// Starting entity ID
        from: String,
        /// Target entity ID
        to: String,
        /// Maximum traversal depth
        #[arg(long, default_value_t = 5)]
        max_depth: usize,
    },
    /// Find entities with contradictory relations
    Contradictions,
}

#[derive(Subcommand)]
pub enum ServiceCommands {
    /// Run MCP HTTP server in foreground
    Serve {
        /// Bind address
        #[arg(long, default_value_t = String::from("127.0.0.1"))]
        host: String,
        /// Bind port
        #[arg(long, default_value_t = 43175)]
        port: u16,
    },
    /// Start MCP HTTP server in background
    Start {
        /// Bind address
        #[arg(long, default_value_t = String::from("127.0.0.1"))]
        host: String,
        /// Bind port
        #[arg(long, default_value_t = 43175)]
        port: u16,
    },
    /// Stop running MCP HTTP server
    Stop,
    /// Restart MCP HTTP server
    Restart {
        /// Bind address
        #[arg(long, default_value_t = String::from("127.0.0.1"))]
        host: String,
        /// Bind port
        #[arg(long, default_value_t = 43175)]
        port: u16,
    },
    /// Show server status
    Status,
    /// Install macOS launchd LaunchAgent
    LaunchdInstall {
        /// Bind address
        #[arg(long, default_value_t = String::from("127.0.0.1"))]
        host: String,
        /// Bind port
        #[arg(long, default_value_t = 43175)]
        port: u16,
    },
    /// Remove macOS launchd LaunchAgent
    LaunchdUninstall,
    /// Show macOS launchd LaunchAgent status
    LaunchdStatus,
    /// Enable launchd login item (Python parity)
    Enable {
        /// Start immediately after enabling
        #[arg(long)]
        now: bool,
    },
    /// Disable launchd login item (Python parity)
    Disable {
        /// Stop immediately before disabling
        #[arg(long)]
        now: bool,
    },
}

#[derive(Subcommand)]
pub enum HooksCommands {
    /// Search knowledge graph for patterns relevant to a prompt
    Ground {
        /// The prompt to search against
        prompt: Option<String>,
        /// Maximum number of results
        #[arg(long, default_value_t = 3)]
        limit: usize,
        /// Output as JSON instead of XML comments
        #[arg(long)]
        json: bool,
    },
    /// Detect code smells in staged or specific files
    Sniff {
        /// Files to analyze
        #[arg(required = false)]
        files: Vec<String>,
        /// Analyze git-staged files
        #[arg(long)]
        staged: bool,
        /// Minimum confidence threshold
        #[arg(long, default_value_t = 0.6)]
        min_confidence: f64,
        /// Output as JSON instead of XML comments
        #[arg(long)]
        json: bool,
        /// Show verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Final quality audit
    Audit {
        /// File to audit (reads from stdin if omitted)
        #[arg(long)]
        file: Option<String>,
        /// Output as JSON instead of XML comments
        #[arg(long)]
        json: bool,
    },
}
