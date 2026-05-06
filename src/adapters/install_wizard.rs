//! Full-screen install picker: arrow keys / j-k, Space toggles, A toggles all, Enter confirms.

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::{Attribute, Print, ResetColor, SetAttribute, SetForegroundColor};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, queue};
use std::io::{self, IsTerminal, Write};

use crate::adapters::installer::ClaudeTransport;

const DEFAULT_MCP_PORT: u16 = 43175;

/// Prompt the user to choose HTTP or stdio transport for Claude Code MCP.
///
/// In a non-TTY environment (CI, pipes), silently returns HTTP + default port.
pub fn select_claude_transport() -> io::Result<ClaudeTransport> {
    if !io::stdin().is_terminal() {
        return Ok(ClaudeTransport::Http {
            port: DEFAULT_MCP_PORT,
        });
    }
    run_transport_tui()
}

fn run_transport_tui() -> io::Result<ClaudeTransport> {
    let options: &[(&str, &str)] = &[
        ("HTTP", "recommended — persistent server, instant tool calls"),
        ("stdio", "spawns a new process per tool call"),
    ];
    let mut cursor = 0usize;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide, MoveTo(0, 0), Clear(ClearType::All))?;

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
        if key.kind != KeyEventKind::Press { continue }

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
        Ok(ClaudeTransport::Http { port })
    } else {
        Ok(ClaudeTransport::Stdio)
    }
}

fn draw_transport(w: &mut impl Write, options: &[(&str, &str)], cursor: usize) -> io::Result<()> {
    queue!(w, MoveTo(0, 0), Clear(ClearType::All))?;
    queue!(
        w,
        SetForegroundColor(ACCENT),
        SetAttribute(Attribute::Bold),
        Print(" ╭────────────────────────────────────────────────────────────────────────╮\r\n"),
        Print(" │ "),
        ResetColor,
        SetForegroundColor(ACCENT),
        SetAttribute(Attribute::Bold),
        Print("Syntagma"),
        ResetColor,
        Print("  ·  MCP transport for Claude Code"),
        SetForegroundColor(ACCENT),
        Print("                                    │\r\n"),
        Print(" ╰────────────────────────────────────────────────────────────────────────╯\r\n"),
        ResetColor,
        Print("\r\n"),
    )?;

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
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide, MoveTo(0, 0), Clear(ClearType::All))?;

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
        queue!(
            stdout,
            SetForegroundColor(ACCENT),
            SetAttribute(Attribute::Bold),
            Print(" ╭────────────────────────────────────────────────────────────────────────╮\r\n"),
            Print(" │ "),
            ResetColor,
            SetForegroundColor(ACCENT),
            SetAttribute(Attribute::Bold),
            Print("Syntagma"),
            ResetColor,
            Print("  ·  MCP HTTP port"),
            SetForegroundColor(ACCENT),
            Print("                                                 │\r\n"),
            Print(" ╰────────────────────────────────────────────────────────────────────────╯\r\n"),
            ResetColor,
            Print("\r\n"),
        )?;

        let display = if input.is_empty() {
            format!(" › Port: {default}_")
        } else {
            format!(" › Port: {input}_")
        };
        queue!(
            stdout,
            SetForegroundColor(HI),
            SetAttribute(Attribute::Bold),
            Print(&display),
            ResetColor,
            Print("\r\n\r\n"),
            SetForegroundColor(DIM),
            Print(" ────────────────────────────────────────────────────────────────────────\r\n"),
            Print("  Type a port number · Enter to confirm\r\n"),
            ResetColor,
        )?;
        stdout.flush()?;

        let ev = event::read()?;
        let Event::Key(key) = ev else { continue };
        if key.kind != KeyEventKind::Press { continue }

        match key.code {
            KeyCode::Enter => {
                let port = input.trim().parse::<u16>().unwrap_or(default);
                return Ok(port);
            }
            KeyCode::Backspace => { input.pop(); }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if input.len() < 5 { input.push(c); }
            }
            KeyCode::Esc => return Ok(default),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(default);
            }
            _ => {}
        }
    }
}

const ACCENT: crossterm::style::Color = crossterm::style::Color::Cyan;
const DIM: crossterm::style::Color = crossterm::style::Color::DarkGrey;
const HI: crossterm::style::Color = crossterm::style::Color::Yellow;

const TOOLS: &[(&str, &str)] = &[
    ("claude", "Claude Code (CLI + IDE)"),
    ("cursor", "Cursor (IDE)"),
    ("codex", "OpenAI Codex CLI"),
    ("gemini", "Google Gemini CLI"),
    ("opencode", "OpenCode"),
    ("cline", "Cline (VS Code)"),
];

/// Interactive multi-select when stdin+stdout are TTYs.
/// Pre-selects tools whose names appear in `installed`.
/// On error or non-interactive, returns `Err` so caller can fall back.
pub fn interactive_select_tools(installed: &[&str]) -> io::Result<Vec<String>> {
    let mut checked: Vec<bool> = TOOLS
        .iter()
        .map(|(name, _)| installed.contains(name))
        .collect();

    let result = run_tui(&mut checked)?;

    if result.is_empty() {
        return Ok(vec![]);
    }

    Ok(result)
}

fn selected_names(checked: &[bool]) -> Vec<String> {
    TOOLS
        .iter()
        .zip(checked.iter())
        .filter_map(|((name, _), &on)| on.then_some(name.to_string()))
        .collect()
}

fn run_tui(checked: &mut [bool]) -> io::Result<Vec<String>> {
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

    let mut cursor = 0usize;
    let mut warning = false;

    loop {
        draw(&mut stdout, checked, cursor, warning)?;
        stdout.flush()?;

        let ev = event::read()?;
        let Event::Key(key) = ev else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        warning = false;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                cursor = cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                cursor = (cursor + 1).min(TOOLS.len() - 1);
            }
            KeyCode::Char(' ') => {
                checked[cursor] = !checked[cursor];
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                let all_on = checked.iter().all(|&c| c);
                checked.fill(!all_on);
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                checked.fill(false);
            }
            KeyCode::Enter => {
                if checked.iter().any(|&c| c) {
                    return Ok(selected_names(checked));
                }
                warning = true;
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                return Ok(vec![]);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(vec![]);
            }
            _ => {}
        }
    }
}

fn draw(
    w: &mut impl Write,
    checked: &[bool],
    cursor: usize,
    warning: bool,
) -> io::Result<()> {
    queue!(w, MoveTo(0, 0), Clear(ClearType::All))?;

    queue!(
        w,
        SetForegroundColor(ACCENT),
        SetAttribute(Attribute::Bold),
        Print(" ╭────────────────────────────────────────────────────────────────────────╮\r\n"),
        Print(" │ "),
        ResetColor,
        SetForegroundColor(ACCENT),
        SetAttribute(Attribute::Bold),
        Print("Syntagma"),
        ResetColor,
        Print("  ·  Install integrations"),
        SetForegroundColor(ACCENT),
        Print("                                         │\r\n"),
        Print(" ╰────────────────────────────────────────────────────────────────────────╯\r\n"),
        ResetColor,
        Print("\r\n"),
    )?;

    for (i, ((name, desc), on)) in TOOLS.iter().zip(checked.iter()).enumerate() {
        let row_hi = i == cursor;
        let mark = if *on { "[x]" } else { "[ ]" };
        let prefix = if row_hi { " › " } else { "   " };

        if row_hi {
            queue!(
                w,
                SetForegroundColor(HI),
                SetAttribute(Attribute::Bold),
                Print(prefix),
                Print(mark),
                Print("  "),
                Print(format!("{name:<12}")),
                ResetColor,
                SetForegroundColor(DIM),
                Print("  "),
                Print(truncate_desc(desc, 44)),
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
                Print(format!("{name:<12}")),
                SetForegroundColor(DIM),
                Print("  "),
                Print(truncate_desc(desc, 44)),
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
    )?;

    if warning {
        queue!(
            w,
            SetForegroundColor(crossterm::style::Color::Red),
            SetAttribute(Attribute::Bold),
            Print("  Select at least one tool.\r\n"),
            ResetColor,
        )?;
    } else {
        queue!(
            w,
            SetForegroundColor(DIM),
            Print("  ↑/↓ Move   Space Toggle   A All   N Clear   Enter Confirm   Esc/Q Quit\r\n"),
            ResetColor,
        )?;
    }

    Ok(())
}

fn truncate_desc(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        return s.to_string();
    }
    let take = max_chars.saturating_sub(1);
    s.chars().take(take).chain(std::iter::once('…')).collect()
}

/// Non-TTY (CI, pipes): comma-separated indices or `a` / `all` for everything.
pub fn fallback_select_tools() -> Vec<String> {
    eprintln!();
    eprintln!("Syntagma — Select integrations to install");
    eprintln!("──────────────────────────────────────────");
    for (i, (name, desc)) in TOOLS.iter().enumerate() {
        eprintln!("  [{}] {:<12} {}", i + 1, name, desc);
    }
    eprintln!("  [a] All of the above");
    eprintln!();
    eprint!("Selection (e.g. 1,3 or a): ");
    let _ = io::stderr().flush();

    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return vec![];
    }
    let line = line.trim().to_lowercase();

    if line == "a" || line == "all" {
        return TOOLS.iter().map(|(name, _)| name.to_string()).collect();
    }

    let mut selected = Vec::new();
    for token in line.split(',') {
        let token = token.trim();
        if let Ok(n) = token.parse::<usize>()
            && n >= 1
            && n <= TOOLS.len()
        {
            selected.push(TOOLS[n - 1].0.to_string());
        }
    }
    selected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::installer::ClaudeTransport;

    #[test]
    fn selected_names_filters_checked() {
        let checked = vec![true, false, true, false, false, false];
        assert_eq!(
            selected_names(&checked),
            vec!["claude".to_string(), "codex".to_string()]
        );
    }

    /// In a non-TTY context (stdin is not a terminal, e.g. CI), `select_claude_transport`
    /// must return HTTP + default port without blocking.
    #[test]
    fn select_claude_transport_non_tty_returns_http_default() {
        // stdin is redirected (not a terminal) in the test harness.
        if io::stdin().is_terminal() {
            // Skip when running interactively; the non-TTY branch is what we test.
            return;
        }
        let result = select_claude_transport().unwrap();
        assert_eq!(result, ClaudeTransport::Http { port: 43175 });
    }
}
