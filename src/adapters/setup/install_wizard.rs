//! Full-screen install picker: arrow keys / j-k, Space toggles, A toggles all, Enter confirms.

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::{Attribute, Print, ResetColor, SetAttribute, SetForegroundColor};
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{execute, queue};
use std::io::{self, IsTerminal, Write};

use crate::adapters::installer::Transport;

const DEFAULT_MCP_PORT: u16 = 43175;

/// Prompt the user to choose HTTP or stdio transport for MCP integration.
///
/// In a non-TTY environment (CI, pipes), silently returns HTTP + default port.
pub fn select_transport() -> io::Result<Transport> {
    if !io::stdin().is_terminal() {
        return Ok(Transport::Http {
            port: DEFAULT_MCP_PORT,
            token: None,
        });
    }
    run_transport_tui()
}

fn run_transport_tui() -> io::Result<Transport> {
    let options: &[(&str, &str)] = &[
        (
            "HTTP",
            "recommended — persistent server, instant tool calls",
        ),
        ("stdio", "spawns a new process per tool call"),
    ];
    let mut cursor = 0usize;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        Hide,
        MoveTo(0, 0),
        Clear(ClearType::All)
    )?;

    struct RawGuard;
    impl Drop for RawGuard {
        fn drop(&mut self) {
            let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
            let _ = disable_raw_mode();
        }
    }
    let _guard = RawGuard;

    let transport = loop {
        draw_transport(&mut stdout, options, cursor)?;
        stdout.flush()?;

        let ev = event::read()?;
        let Event::Key(key) = ev else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                cursor = cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                cursor = (cursor + 1).min(options.len() - 1);
            }
            KeyCode::Enter => break cursor,
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => break 0,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break 0,
            _ => {}
        }
    };

    drop(_guard);

    if transport == 0 {
        // HTTP selected — prompt for port in same style
        let port = prompt_port_tui(DEFAULT_MCP_PORT)?;
        Ok(Transport::Http { port, token: None })
    } else {
        Ok(Transport::Stdio)
    }
}

fn draw_transport(w: &mut impl Write, options: &[(&str, &str)], cursor: usize) -> io::Result<()> {
    queue!(w, MoveTo(0, 0), Clear(ClearType::All))?;
    tui_header(w, "MCP transport for Claude Code")?;
    queue!(w, Print("\r\n"))?;

    for (i, (name, desc)) in options.iter().enumerate() {
        let row_hi = i == cursor;
        let mark = if row_hi { "[•]" } else { "[ ]" };
        let prefix = if row_hi { " › " } else { "   " };

        if row_hi {
            queue!(
                w,
                SetForegroundColor(HI),
                SetAttribute(Attribute::Bold),
                Print(prefix),
                Print(mark),
                Print("  "),
                Print(format!("{name:<8}")),
                ResetColor,
                SetForegroundColor(DIM),
                Print("  "),
                Print(truncate_desc(desc, 48)),
                ResetColor,
                Print("\r\n"),
            )?;
        } else {
            queue!(
                w,
                Print(prefix),
                SetForegroundColor(DIM),
                Print(mark),
                ResetColor,
                Print("  "),
                Print(format!("{name:<8}")),
                SetForegroundColor(DIM),
                Print("  "),
                Print(truncate_desc(desc, 48)),
                ResetColor,
                Print("\r\n"),
            )?;
        }
    }

    queue!(
        w,
        Print("\r\n"),
        SetForegroundColor(DIM),
        Print(" ────────────────────────────────────────────────────────────────────────\r\n"),
        Print("  ↑/↓ Move   Enter Confirm   Esc/Q Quit\r\n"),
        ResetColor,
    )?;
    Ok(())
}

fn prompt_port_tui(default: u16) -> io::Result<u16> {
    prompt_numeric_tui("MCP HTTP port", "port", default)
}

const ACCENT: crossterm::style::Color = crossterm::style::Color::Cyan;
const DIM: crossterm::style::Color = crossterm::style::Color::DarkGrey;
const HI: crossterm::style::Color = crossterm::style::Color::Yellow;

fn truncate_desc(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        return s.to_string();
    }
    let take = max_chars.saturating_sub(1);
    s.chars().take(take).chain(std::iter::once('…')).collect()
}

// ---------------------------------------------------------------------------
// Server config TUI (host + bearer token)
// ---------------------------------------------------------------------------

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub token: Option<String>,
}

/// Interactive server configuration screen.
///
/// - Asks for bind address (127.0.0.1 or 0.0.0.0)
/// - Token generation defaults to Yes for 0.0.0.0 and No for 127.0.0.1
///
/// Non-TTY returns defaults (127.0.0.1, current port, no token).
pub fn configure_server_tui(
    server_label: &str,
    current_host: &str,
    current_port: u16,
    current_token: &str,
) -> io::Result<ServerConfig> {
    if !io::stdin().is_terminal() {
        return Ok(ServerConfig {
            host: current_host.to_owned(),
            port: current_port,
            token: None,
        });
    }

    // Step 1: Host selection
    let host = run_host_select_tui()?;

    // Step 2: Token decision. Default follows the bind address:
    // remote bind defaults to Yes, localhost defaults to No.
    let is_public = !crate::server::mcp_auth::is_localhost(&host);
    let generate = run_yes_no_tui(
        "Server auth",
        &format!("Generate a bearer token for {server_label} access?"),
        is_public,
    )?;
    let token = if generate {
        let t = crate::server::mcp_auth::generate_token();
        show_token_tui(&t)?;
        Some(t)
    } else if !current_token.is_empty() {
        // Keep existing token if present
        Some(current_token.to_owned())
    } else {
        None
    };

    // Step 3: Port (reuse existing)
    let port = prompt_numeric_tui::<u16>(server_label, "port", current_port)?;

    Ok(ServerConfig { host, port, token })
}

fn run_host_select_tui() -> io::Result<String> {
    let options: &[(&str, &str, &str)] = &[
        (
            "127.0.0.1",
            "Local only (recommended)",
            "Only connections from this machine",
        ),
        (
            "0.0.0.0",
            "Remote accessible",
            "Allows connections from other machines (token required)",
        ),
    ];
    let mut cursor = 0usize;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        Hide,
        MoveTo(0, 0),
        Clear(ClearType::All)
    )?;

    struct RawGuard;
    impl Drop for RawGuard {
        fn drop(&mut self) {
            let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
            let _ = disable_raw_mode();
        }
    }
    let _guard = RawGuard;

    loop {
        queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        tui_header(&mut stdout, "API bind address")?;
        queue!(stdout, Print("\r\n"))?;

        for (i, (addr, label, desc)) in options.iter().enumerate() {
            let row_hi = i == cursor;
            let mark = if row_hi { "[•]" } else { "[ ]" };
            let prefix = if row_hi { " › " } else { "   " };

            if row_hi {
                queue!(
                    stdout,
                    SetForegroundColor(HI),
                    SetAttribute(Attribute::Bold),
                    Print(prefix),
                    Print(mark),
                    Print("  "),
                    Print(format!("{addr:<12}")),
                    ResetColor,
                    SetForegroundColor(DIM),
                    Print("  "),
                    Print(truncate_desc(label, 28)),
                    ResetColor,
                    Print("\r\n"),
                )?;
                queue!(
                    stdout,
                    SetForegroundColor(DIM),
                    Print("                 "),
                    Print(truncate_desc(desc, 50)),
                    Print("\r\n"),
                    ResetColor,
                )?;
            } else {
                queue!(
                    stdout,
                    Print(prefix),
                    SetForegroundColor(DIM),
                    Print(mark),
                    ResetColor,
                    Print("  "),
                    Print(format!("{addr:<12}")),
                    SetForegroundColor(DIM),
                    Print("  "),
                    Print(truncate_desc(label, 28)),
                    ResetColor,
                    Print("\r\n"),
                )?;
            }
        }

        queue!(
            stdout,
            Print("\r\n"),
            SetForegroundColor(DIM),
            Print(" ────────────────────────────────────────────────────────────────────────\r\n"),
            Print("  ↑/↓ Move   Enter Confirm\r\n"),
            ResetColor,
        )?;
        stdout.flush()?;

        let ev = event::read()?;
        let Event::Key(key) = ev else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                cursor = cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                cursor = (cursor + 1).min(options.len() - 1);
            }
            KeyCode::Enter => {
                let selected = options[cursor].0;
                drop(_guard);
                return Ok(selected.to_owned());
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                drop(_guard);
                return Ok("127.0.0.1".to_owned());
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                drop(_guard);
                return Ok("127.0.0.1".to_owned());
            }
            _ => {}
        }
    }
}

/// Display a generated token to the user and wait for Enter.
fn show_token_tui(token: &str) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        Hide,
        MoveTo(0, 0),
        Clear(ClearType::All)
    )?;

    struct RawGuard;
    impl Drop for RawGuard {
        fn drop(&mut self) {
            let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
            let _ = disable_raw_mode();
        }
    }
    let _guard = RawGuard;

    let title = "Token generated";

    loop {
        queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        tui_header(&mut stdout, title)?;
        queue!(
            stdout,
            Print("\r\n"),
            SetForegroundColor(HI),
            Print("  Bearer token:\r\n"),
            ResetColor,
            Print("\r\n"),
            SetForegroundColor(ACCENT),
            SetAttribute(Attribute::Bold),
            Print(format!("  {token}\r\n")),
            ResetColor,
            Print("\r\n"),
            SetForegroundColor(DIM),
            Print("  Copy this token now. It will be saved to config.yaml\r\n"),
            Print("  and seeded to your AI tool MCP configurations.\r\n"),
            Print("\r\n"),
            Print(" ────────────────────────────────────────────────────────────────────────\r\n"),
            Print("  Press Enter to continue\r\n"),
            ResetColor,
        )?;
        stdout.flush()?;

        let ev = event::read()?;
        let Event::Key(key) = ev else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Enter | KeyCode::Esc => break,
            _ => {}
        }
    }

    drop(_guard);
    // Also print to stdout so it appears in terminal scrollback
    println!("\nEpisteme bearer token: {token}\n");
    Ok(())
}

// ---------------------------------------------------------------------------
// Redis config TUI (only available when compiled with `redis-cache` feature)
// ---------------------------------------------------------------------------

#[cfg(feature = "redis-cache")]
pub struct RedisConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub db: u16,
    pub ttl: u64,
}

/// Interactive Redis config screen. Returns None if user skips.
#[cfg(feature = "redis-cache")]
pub fn configure_redis_tui(current: RedisConfig) -> io::Result<Option<RedisConfig>> {
    if !io::stdin().is_terminal() {
        return Ok(None);
    }

    // Ask enable/skip first
    let enable = run_yes_no_tui("Redis cache", "Configure Redis now?", true)?;
    if !enable {
        return Ok(None);
    }

    // Field editing: host, port, db, ttl
    let host = prompt_field_tui("Redis cache", "host", &current.host)?;
    let port = prompt_numeric_tui::<u16>("Redis cache", "port", current.port)?;
    let db = prompt_numeric_tui::<u16>("Redis cache", "db", current.db)?;
    let ttl = prompt_numeric_tui::<u64>("Redis cache", "ttl (seconds)", current.ttl)?;

    Ok(Some(RedisConfig {
        enabled: true,
        host,
        port,
        db,
        ttl,
    }))
}

/// Interactive telemetry consent screen. Returns true if user consents.
pub fn configure_telemetry_tui() -> io::Result<bool> {
    if !io::stdin().is_terminal() {
        return Ok(true);
    }
    run_yes_no_tui(
        "Telemetry",
        "Share anonymous usage data to improve Episteme?",
        true,
    )
}

fn run_yes_no_tui(title: &str, question: &str, default_yes: bool) -> io::Result<bool> {
    let options = if default_yes {
        vec![("Yes", true), ("No", false)]
    } else {
        vec![("No", false), ("Yes", true)]
    };
    let mut cursor = 0usize;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        Hide,
        MoveTo(0, 0),
        Clear(ClearType::All)
    )?;

    struct RawGuard;
    impl Drop for RawGuard {
        fn drop(&mut self) {
            let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
            let _ = disable_raw_mode();
        }
    }
    let _guard = RawGuard;

    loop {
        queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        tui_header(&mut stdout, title)?;
        queue!(
            stdout,
            Print("\r\n"),
            SetForegroundColor(HI),
            Print(format!("  {question}\r\n")),
            ResetColor,
            Print("\r\n"),
        )?;

        for (i, (label, _)) in options.iter().enumerate() {
            let row_hi = i == cursor;
            if row_hi {
                queue!(
                    stdout,
                    SetForegroundColor(HI),
                    SetAttribute(Attribute::Bold),
                    Print(format!(" › [•]  {label}\r\n")),
                    ResetColor,
                )?;
            } else {
                queue!(
                    stdout,
                    SetForegroundColor(DIM),
                    Print(format!("   [ ]  {label}\r\n")),
                    ResetColor,
                )?;
            }
        }

        queue!(
            stdout,
            Print("\r\n"),
            SetForegroundColor(DIM),
            Print(" ────────────────────────────────────────────────────────────────────────\r\n"),
            Print("  ↑/↓ Move   Enter Confirm\r\n"),
            ResetColor,
        )?;
        stdout.flush()?;

        let ev = event::read()?;
        let Event::Key(key) = ev else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => cursor = cursor.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => cursor = (cursor + 1).min(options.len() - 1),
            KeyCode::Enter => return Ok(options[cursor].1),
            KeyCode::Esc | KeyCode::Char('q') => return Ok(!default_yes),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(!default_yes);
            }
            _ => {}
        }
    }
}

fn prompt_field_tui(title: &str, label: &str, default: &str) -> io::Result<String> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        Hide,
        MoveTo(0, 0),
        Clear(ClearType::All)
    )?;

    struct RawGuard;
    impl Drop for RawGuard {
        fn drop(&mut self) {
            let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
            let _ = disable_raw_mode();
        }
    }
    let _guard = RawGuard;

    let mut input = String::new();

    loop {
        queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        tui_header(&mut stdout, title)?;

        let display = if input.is_empty() {
            format!(" › {label}: {default}_")
        } else {
            format!(" › {label}: {input}_")
        };
        queue!(
            stdout,
            Print("\r\n"),
            SetForegroundColor(HI),
            SetAttribute(Attribute::Bold),
            Print(&display),
            ResetColor,
            Print("\r\n\r\n"),
            SetForegroundColor(DIM),
            Print(" ────────────────────────────────────────────────────────────────────────\r\n"),
            Print("  Type value · Enter to confirm · Esc to keep default\r\n"),
            ResetColor,
        )?;
        stdout.flush()?;

        let ev = event::read()?;
        let Event::Key(key) = ev else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Enter => {
                return Ok(if input.is_empty() {
                    default.to_owned()
                } else {
                    input
                });
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                input.push(c);
            }
            KeyCode::Esc => return Ok(default.to_owned()),
            _ => {}
        }
    }
}

fn prompt_numeric_tui<T>(title: &str, label: &str, default: T) -> io::Result<T>
where
    T: std::str::FromStr + std::fmt::Display + Copy,
{
    let default_str = default.to_string();
    let raw = prompt_field_tui(title, label, &default_str)?;
    Ok(raw.parse::<T>().unwrap_or(default))
}

fn tui_header(w: &mut impl Write, subtitle: &str) -> io::Result<()> {
    queue!(
        w,
        SetForegroundColor(ACCENT),
        SetAttribute(Attribute::Bold),
        Print(" ────────────────────────────────────────────────────────────────────────\r\n"),
        Print("  "),
        ResetColor,
        SetForegroundColor(ACCENT),
        SetAttribute(Attribute::Bold),
        Print("Episteme"),
        ResetColor,
        Print(format!("  ·  {subtitle}\r\n")),
        SetForegroundColor(ACCENT),
        SetAttribute(Attribute::Bold),
        Print(" ────────────────────────────────────────────────────────────────────────\r\n"),
        ResetColor,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::installer::Transport;

    /// In a non-TTY context (stdin is not a terminal, e.g. CI), `select_transport`
    /// must return HTTP + default port without blocking.
    #[test]
    fn select_transport_non_tty_returns_http_default() {
        // stdin is redirected (not a terminal) in the test harness.
        if io::stdin().is_terminal() {
            // Skip when running interactively; the non-TTY branch is what we test.
            return;
        }
        let result = select_transport().unwrap();
        assert_eq!(
            result,
            Transport::Http {
                port: 43175,
                token: None,
            }
        );
    }
}
