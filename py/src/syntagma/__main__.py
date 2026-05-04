#!/usr/bin/env python3
"""
Syntagma CLI Entry Point

Main command-line interface for Syntagma operations.
Run with: python -m syntagma [command]
"""

import argparse
import sys


def main():
    """Main CLI entry point with subcommands."""
    parser = argparse.ArgumentParser(
        prog="syntagma",
        description="Syntagma — Knowledge graph for software engineering: patterns, laws, smells, refactorings.",
        epilog=(
            "Commands:\n"
            "  install   Onboard Syntagma into your AI tool (downloads data automatically)\n"
            "  build     Generate the vector database from seeded data\n"
            "  service   Manage the MCP HTTP daemon (start/stop/enable)\n"
            "  analyze   Detect code smells in source files\n"
            "  infer     Suggest ranked refactorings for detected smells\n"
            "  explore   Interactively browse the knowledge graph\n"
            "  api       Start the REST API server\n"
            "  mcp       Start the MCP server\n"
            "  telemetry Manage telemetry consent  (on|off|status)\n\n"
            "Quickstart:\n"
            "  syntagma install claude   # install agents + download data\n"
            "  syntagma build            # generate vector DB\n"
            "  syntagma service start    # start MCP proxy in background\n\n"
            "Use 'syntagma <command> --help' for details."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    subparsers = parser.add_subparsers(dest="command", metavar="COMMAND")

    # Analyze command
    analyze_parser = subparsers.add_parser(
        "analyze",
        help="Detect code smells in source files",
        description=(
            "Analyze source files for code smells using AST-based detection.\n"
            "Supports Python, Java, TypeScript, Go, Rust, C++, C#, PHP, Ruby, Kotlin."
        ),
        epilog=(
            "Examples:\n"
            "  syntagma analyze my_code.py\n"
            "  syntagma analyze src/ --json\n"
            "  syntagma analyze app.java --language java"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    analyze_parser.add_argument("file", help="Python file or directory to analyze")
    analyze_parser.add_argument("--json", action="store_true", help="Output JSON format")
    analyze_parser.add_argument(
        "--min-confidence",
        type=float,
        default=0.5,
        help="Minimum confidence threshold (0.0-1.0)",
    )
    analyze_parser.add_argument("--language", help="Language hint (e.g. java, typescript)")

    # Infer command
    infer_parser = subparsers.add_parser(
        "infer",
        help="Suggest refactorings for detected code smells",
        description=(
            "Map detected code smells to ranked refactoring suggestions\n"
            "using the Syntagma knowledge graph."
        ),
        epilog=("Examples:\n  syntagma infer my_code.py\n  syntagma infer src/ --top-k 5"),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    infer_parser.add_argument("file", help="Python file or directory to analyze")
    infer_parser.add_argument("--top-k", type=int, default=3, help="Top K suggestions per smell")
    infer_parser.add_argument("--json", action="store_true", help="Output JSON format")

    # Explore command
    subparsers.add_parser(
        "explore",
        help="Interactively explore the knowledge graph",
        description=(
            "Navigate the knowledge graph: browse entities, traverse relationships,\n"
            "find shortest paths, and explore subgraphs.\n\n"
            "Entity types: pattern (DP-xxx), refactoring (RF-xxx),\n"
            "              law (LAW-xxx), smell (SMELL-xxx)"
        ),
        epilog=(
            "Examples:\n  syntagma explore\n  > goto DP-005\n  > neighbors\n  > path LAW-042-S"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    # API command
    subparsers.add_parser(
        "api",
        help="Start the REST API server",
        description=(
            "Start the Syntagma REST API server (FastAPI + Uvicorn).\n"
            "Provides 15+ endpoints for search, analysis, and graph traversal.\n\n"
            "Docs available at http://localhost:8000/docs after startup."
        ),
        epilog=("Examples:\n  syntagma api\n  UVICORN_PORT=8080 syntagma api"),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    # MCP command
    subparsers.add_parser(
        "mcp",
        help="Start the MCP server for AI tool integration",
        description=(
            "Start the Syntagma MCP (Model Context Protocol) server.\n"
            "Exposes 6 tools to AI agents: search_knowledge, get_entity,\n"
            "get_neighbors, find_path, analyze_code, suggest_refactorings.\n\n"
            "Default mode: stdio (for Claude Code, Cursor, Codex, etc.)\n"
            "HTTP mode:    syntagma mcp --http  (port 43175)"
        ),
        epilog=(
            "Examples:\n"
            "  syntagma mcp               # stdio mode\n"
            "  syntagma mcp --http        # HTTP mode on port 43175\n"
            "  syntagma mcp --http --port 9000"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    # Service command
    service_parser = subparsers.add_parser(
        "service",
        help="Manage the MCP HTTP proxy as a background daemon",
        description=(
            "Manage the Syntagma MCP HTTP proxy as a background daemon.\n\n"
            "  serve    Run in the foreground (Ctrl+C to stop)\n"
            "  start    Launch in the background (PID file)\n"
            "  stop     Stop the running daemon\n"
            "  restart  Stop then start\n"
            "  status   Show running state and PID\n"
            "  enable   Register as a login item (macOS LaunchAgent)\n"
            "  disable  Remove the login item"
        ),
        epilog=(
            "Examples:\n"
            "  syntagma service serve           # foreground\n"
            "  syntagma service start           # background\n"
            "  syntagma service status\n"
            "  syntagma service enable          # register login item only\n"
            "  syntagma service enable --now    # register + start immediately\n"
            "  syntagma service disable         # unregister only\n"
            "  syntagma service disable --now   # stop + unregister"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    service_parser.add_argument(
        "subcommand",
        choices=["serve", "start", "stop", "restart", "status", "enable", "disable"],
        metavar="COMMAND",
        help="serve | start | stop | restart | status | enable | disable",
    )
    service_parser.add_argument(
        "--now",
        action="store_true",
        default=False,
        help="For enable: also start now. For disable: also stop now.",
    )

    # Build command
    build_parser = subparsers.add_parser(
        "build",
        help="Generate the vector database from seeded knowledge data",
        description=(
            "Generate the Syntagma vector database from seeded knowledge data.\n\n"
            "Reads markdown files from ~/.syntagma/raw/ (seeded by `syntagma install`),\n"
            "chunks them, generates sentence-transformer embeddings, and stores everything\n"
            "in ~/.syntagma/db/syntagma.db for semantic search and MCP tools.\n\n"
            "GPU acceleration (CUDA / Apple MPS) is used automatically when available."
        ),
        epilog=(
            "Examples:\n"
            "  syntagma build               # auto-detect GPU\n"
            "  syntagma build --no-gpu      # force CPU\n"
            "  syntagma build --rebuild     # drop and rebuild from scratch\n"
            "  syntagma build --stats       # show database statistics"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    build_parser.add_argument("--gpu", action="store_true", help="Force GPU mode")
    build_parser.add_argument("--no-gpu", action="store_true", help="Force CPU mode")
    build_parser.add_argument("--batch-size", type=int, default=64, help="Embedding batch size")
    build_parser.add_argument(
        "--rebuild", action="store_true", help="Drop and rebuild from scratch"
    )
    build_parser.add_argument("--stats", action="store_true", help="Show database statistics")

    # Install command
    install_parser = subparsers.add_parser(
        "install",
        help="Install agents and MCP config into your AI tool",
        description=(
            "Onboard Syntagma into your AI coding tool.\n"
            "Copies agents from registry/ and registers the MCP server.\n\n"
            "Supported tools: claude, cursor, codex, gemini, opencode, all\n\n"
            "  claude   — copies agents to ~/.claude/agents/, registers in ~/.claude.json\n"
            "  cursor   — creates .cursor/mcp.json in current project\n"
            "  codex    — creates/appends AGENTS.md in current project\n"
            "  gemini   — registers in ~/.gemini/mcp.json\n"
            "  opencode — creates .opencode/mcp.json in current project"
        ),
        epilog=(
            "Examples:\n"
            "  syntagma install                   # interactive mode\n"
            "  syntagma install all               # install for all tools\n"
            "  syntagma install claude            # Claude Code only\n"
            "  syntagma install cursor codex      # multiple tools\n"
            "  syntagma install claude --dry-run  # preview without writing files"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    install_parser.add_argument(
        "tools",
        nargs="*",
        metavar="TOOL",
        help="AI tools to install for (claude, cursor, codex, gemini, opencode, all)",
    )
    install_parser.add_argument(
        "--dry-run",
        action="store_true",
        default=False,
        help="Show what would be installed without writing files",
    )
    install_parser.add_argument(
        "--offline",
        metavar="PATH",
        default=None,
        help="Use a local syntagma-data-*.tar.gz instead of downloading from GitHub Releases",
    )
    install_parser.add_argument(
        "--local",
        action="store_true",
        default=False,
        help="Seed from source tree (meta/ + raw/) without downloading — for dev/build use",
    )

    # Web command
    web_parser = subparsers.add_parser(
        "web",
        help="Start the graph visualization web UI",
        description=(
            "Start the interactive knowledge graph web viewer.\n"
            "Opens a browser-based UI at http://localhost:8001\n\n"
            "Features:\n"
            "  - Full graph view (all entities and relationships)\n"
            "  - Entity subgraph (click a node to explore)\n"
            "  - Shortest path between any two entities\n"
            "  - Keyword search"
        ),
        epilog=(
            "Examples:\n"
            "  syntagma web\n"
            "  syntagma web --port 9001\n"
            "  syntagma web --host 0.0.0.0 --port 8001"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    web_parser.add_argument("--host", default="127.0.0.1", help="Bind host (default: 127.0.0.1)")
    web_parser.add_argument("--port", type=int, default=8001, help="Bind port (default: 8001)")

    # Register telemetry subcommand
    telemetry_parser = subparsers.add_parser(
        "telemetry",
        help="Manage telemetry consent",
        description="Enable or disable anonymous usage telemetry.",
        epilog=(
            "Examples:\n"
            "  syntagma telemetry          # show current status\n"
            "  syntagma telemetry on       # enable\n"
            "  syntagma telemetry off      # disable"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    telemetry_parser.add_argument(
        "action",
        nargs="?",
        default="status",
        choices=["on", "off", "status"],
        help="on | off | status (default)",
    )

    args = parser.parse_args()

    if args.command is None:
        parser.print_help()
        sys.exit(0)

    # install and telemetry manage consent — skip auto-enable
    if args.command == "install":
        from syntagma.cli.install import main as install_main

        sys.exit(install_main(sys.argv[2:]))
    if args.command == "telemetry":
        from syntagma.telemetry import run_cli

        sys.exit(run_cli(args.action))

    # All other commands: auto-enable telemetry on first run (opt-out model)
    import time as _time

    from syntagma.telemetry import (
        Command,
        FailureClass,
        ensure_consent_or_set_default,
        track_command_completed,
        track_command_failed,
        track_command_invoked,
        track_session_started,
    )

    ensure_consent_or_set_default()
    track_session_started()

    _CMD_MAP = {
        "build": Command.Build,
        "analyze": Command.Analyze,
        "explore": Command.Explore,
        "infer": Command.Infer,
        "api": Command.Api,
        "mcp": Command.Mcp,
        "service": Command.Service,
        "web": Command.Api,
    }
    _cmd_enum = _CMD_MAP.get(args.command)
    if _cmd_enum is not None:
        track_command_invoked(_cmd_enum)

    _t0 = _time.monotonic()

    def _run_command() -> int:
        if args.command == "build":
            from syntagma.cli.build import main as build_main

            return build_main(sys.argv[2:])
        elif args.command == "analyze":
            from syntagma.cli.analyze import main as analyze_main

            return int(analyze_main(sys.argv[2:]) or 0)
        elif args.command == "explore":
            from syntagma.cli.explore import main as explore_main

            return int(explore_main() or 0)
        elif args.command == "infer":
            from syntagma.cli.infer import main as infer_main

            return int(infer_main(sys.argv[2:]) or 0)
        elif args.command == "api":
            from syntagma.api.main import start

            return int(start() or 0)
        elif args.command == "mcp":
            from syntagma.mcp.server import main as mcp_main

            mcp_main()
            return 0
        elif args.command == "service":
            from syntagma.cli.service import main as service_main

            service_argv = [args.subcommand]
            if args.now:
                service_argv.append("--now")
            return service_main(service_argv)
        elif args.command == "web":
            from syntagma.web.graph_viewer import start

            start(host=args.host, port=args.port)
            return 0
        else:
            parser.print_help()
            return 1

    try:
        rc = _run_command()
    except Exception:
        if _cmd_enum is not None:
            track_command_failed(_cmd_enum, FailureClass.Unknown)
        raise

    if _cmd_enum is not None:
        _duration_ms = int((_time.monotonic() - _t0) * 1000)
        if rc == 0:
            track_command_completed(_cmd_enum, _duration_ms)
        else:
            track_command_failed(_cmd_enum, FailureClass.Unknown)

    sys.exit(rc)


if __name__ == "__main__":
    main()
