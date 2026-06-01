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
    #[command(name = "search", alias = "explore")]
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
        /// Skip embedding database file (~/.episteme/db/episteme.db)
        #[arg(long)]
        no_db: bool,
        /// Do not auto-build DB when missing
        #[arg(long)]
        skip_build: bool,
    },
    /// Start the REST API server (foreground when no subcommand given)
    Api {
        /// REST API server lifecycle management
        #[command(subcommand)]
        sub: Option<ServiceLifecycle>,
        /// Bind address (for foreground mode)
        #[arg(long, default_value_t = String::from("0.0.0.0"))]
        host: String,
        /// Bind port (for foreground mode)
        #[arg(long, default_value_t = 8000)]
        port: u16,
    },
    /// (deprecated) Use 'mcp start/stop/restart/status/enable/disable' instead
    #[command(name = "service", alias = "mcp-service")]
    Service {
        #[command(subcommand)]
        sub: ServiceCommands,
    },
    /// Start the MCP server (stdio mode when no subcommand given)
    Mcp {
        /// MCP server lifecycle management
        #[command(subcommand)]
        sub: Option<ServiceLifecycle>,
        /// (deprecated) Use 'mcp start' or 'mcp serve' instead
        #[arg(long, hide = true)]
        http: bool,
        /// Internal: bind host for background transport
        #[arg(long, default_value_t = String::from("127.0.0.1"), hide = true)]
        host: String,
        /// Internal: bind port for background transport
        #[arg(long, default_value_t = 43175, hide = true)]
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
    /// Manage user insights (tacit knowledge)
    Insight {
        #[command(subcommand)]
        sub: InsightCommands,
    },
    /// Install Episteme into AI tools
    Install {
        /// Tools to install (cursor, opencode, cline, all)
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
pub enum ServiceLifecycle {
    /// Start daemon in background
    Start {
        #[arg(long)]
        host: Option<String>,
        #[arg(long)]
        port: Option<u16>,
    },
    /// Stop running daemon
    Stop,
    /// Restart daemon
    Restart {
        #[arg(long)]
        host: Option<String>,
        #[arg(long)]
        port: Option<u16>,
    },
    /// Show daemon status
    Status,
    /// Register as OS login service (macOS: launchd, Linux: systemd)
    Enable {
        /// Start immediately after enabling
        #[arg(long)]
        now: bool,
    },
    /// Unregister OS login service
    Disable {
        /// Stop immediately before disabling
        #[arg(long)]
        now: bool,
    },
    /// Internal use: run foreground server for background daemon
    #[command(hide = true)]
    Serve {
        #[arg(long)]
        host: String,
        #[arg(long)]
        port: u16,
    },
}

#[derive(Subcommand)]
pub enum ServiceCommands {
    /// (deprecated) Use 'mcp start' instead
    Serve {
        /// Bind address
        #[arg(long, default_value_t = String::from("127.0.0.1"))]
        host: String,
        /// Bind port
        #[arg(long, default_value_t = 43175)]
        port: u16,
    },
    /// (deprecated) Use 'mcp start' instead
    Start {
        /// Bind address
        #[arg(long, default_value_t = String::from("127.0.0.1"))]
        host: String,
        /// Bind port
        #[arg(long, default_value_t = 43175)]
        port: u16,
    },
    /// (deprecated) Use 'mcp stop' instead
    Stop,
    /// (deprecated) Use 'mcp restart' instead
    Restart {
        /// Bind address
        #[arg(long, default_value_t = String::from("127.0.0.1"))]
        host: String,
        /// Bind port
        #[arg(long, default_value_t = 43175)]
        port: u16,
    },
    /// (deprecated) Use 'mcp status' instead
    Status,
    /// (deprecated) Use 'mcp enable' instead
    LaunchdInstall {
        /// Bind address
        #[arg(long, default_value_t = String::from("127.0.0.1"))]
        host: String,
        /// Bind port
        #[arg(long, default_value_t = 43175)]
        port: u16,
    },
    /// (deprecated) Use 'mcp disable' instead
    LaunchdUninstall,
    /// (deprecated) Use 'mcp status' instead
    LaunchdStatus,
    /// (deprecated) Use 'mcp enable' instead
    Enable {
        /// Start immediately after enabling
        #[arg(long)]
        now: bool,
    },
    /// (deprecated) Use 'mcp disable' instead
    Disable {
        /// Stop immediately before disabling
        #[arg(long)]
        now: bool,
    },
}

#[derive(Subcommand)]
pub enum InsightCommands {
    /// Add a new insight to the user knowledge graph
    Add {
        /// Title of the insight
        title: String,
        /// Content / body of the insight
        content: String,
        /// Comma-separated tags
        #[arg(long)]
        tags: Option<String>,
        /// Comma-separated entity IDs to link (e.g. DP-005,SMELL-01)
        #[arg(long)]
        link: Option<String>,
    },
    /// List all user insights
    List {
        /// Maximum number of results
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Search user insights by keyword
    Search {
        /// Search query
        query: String,
        /// Maximum number of results
        #[arg(long, default_value_t = 10)]
        limit: usize,
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
