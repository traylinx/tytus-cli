//! Wizard framework — reusable visual components for interactive CLI UX.
//!
//! All primitives respect non-TTY contexts (CI, pipes): when stdout isn't a TTY,
//! spinners become plain text and prompts fail with a helpful error.

use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use std::time::Duration;

/// Flush stdout immediately. Needed because Rust's println! is
/// BLOCK-buffered when stdout is connected to a pipe (e.g., the tray's
/// tytus-tray process spawning `tytus test` with Stdio::piped() so
/// output can stream into the Tower web UI via SSE). Without an explicit
/// flush, every wizard-helper line accumulates in stdout's BufWriter
/// and only releases at process exit — the user sees nothing for ~10s,
/// then the entire output appears at once. Cheap on TTY (already
/// line-buffered, flush is a no-op).
#[inline]
pub(crate) fn flush() {
    let _ = std::io::stdout().flush();
}

/// ASCII logo shown on welcome screens.
pub const LOGO: &str = r#"
   ████████╗██╗   ██╗████████╗██╗   ██╗███████╗
   ╚══██╔══╝╚██╗ ██╔╝╚══██╔══╝██║   ██║██╔════╝
      ██║    ╚████╔╝    ██║   ██║   ██║███████╗
      ██║     ╚██╔╝     ██║   ██║   ██║╚════██║
      ██║      ██║      ██║   ╚██████╔╝███████║
      ╚═╝      ╚═╝      ╚═╝    ╚═════╝ ╚══════╝
     Your own private AI — talk to it from any terminal
"#;

/// Short banner for less prominent contexts.
pub const MINI_LOGO: &str = "🦞 Tytus";

/// Check if we're running in an interactive terminal (TTY).
/// Returns false if --headless flag is set, TYTUS_HEADLESS=1 env var is present,
/// or stdout is not a TTY. LaunchAgents can allocate a pseudo-TTY, so the env
/// var / flag is the reliable override for automated contexts.
pub fn is_interactive() -> bool {
    if std::env::var("TYTUS_HEADLESS").is_ok_and(|v| v == "1") {
        return false;
    }
    Term::stdout().features().is_attended()
}

/// Print the big logo in cyan.
pub fn print_logo() {
    println!("{}", style(LOGO).cyan().bold());
    flush();
}

/// Print a section header with a decorated line.
pub fn print_header(text: &str) {
    let line = "━".repeat(60);
    println!();
    println!("{}", style(&line).cyan().dim());
    println!("  {}", style(text).cyan().bold());
    println!("{}", style(&line).cyan().dim());
    println!();
    flush();
}

/// Print a step indicator like "Step 2/5: Connecting..."
pub fn print_step(current: usize, total: usize, text: &str) {
    let prefix = format!("[{}/{}]", current, total);
    println!("{} {}", style(prefix).cyan().bold(), style(text).bold());
    flush();
}

/// Status icons with colors.
pub fn icon_ok() -> console::StyledObject<&'static str> {
    style("✓").green().bold()
}

pub fn icon_fail() -> console::StyledObject<&'static str> {
    style("✗").red().bold()
}

pub fn icon_warn() -> console::StyledObject<&'static str> {
    style("⚠").yellow().bold()
}

pub fn icon_info() -> console::StyledObject<&'static str> {
    style("ℹ").blue().bold()
}

pub fn icon_arrow() -> console::StyledObject<&'static str> {
    style("→").cyan().bold()
}

/// Print a green success line.
pub fn print_ok(msg: &str) {
    println!("  {} {}", icon_ok(), msg);
    flush();
}

/// Print a red failure line.
pub fn print_fail(msg: &str) {
    println!("  {} {}", icon_fail(), msg);
    flush();
}

/// Print a yellow warning line.
pub fn print_warn(msg: &str) {
    println!("  {} {}", icon_warn(), style(msg).yellow());
    flush();
}

/// Print a blue info line.
pub fn print_info(msg: &str) {
    println!("  {} {}", icon_info(), msg);
    flush();
}

/// Print a cyan arrow hint / next-action line.
pub fn print_hint(msg: &str) {
    println!("  {} {}", icon_arrow(), style(msg).cyan());
    flush();
}

/// Print a boxed message (for important info).
pub fn print_box(title: &str, lines: &[&str]) {
    let width = lines
        .iter()
        .map(|l| console::measure_text_width(l))
        .max()
        .unwrap_or(0)
        .max(console::measure_text_width(title))
        + 4;

    let top = format!("╭─ {} {}╮", style(title).bold(), "─".repeat(width.saturating_sub(title.chars().count() + 4)));
    let bot = format!("╰{}╯", "─".repeat(width));
    println!("{}", style(top).cyan());
    flush();
    for line in lines {
        let padding = width.saturating_sub(console::measure_text_width(line) + 2);
        println!("{} {}{} {}", style("│").cyan(), line, " ".repeat(padding), style("│").cyan());
    }
    println!("{}", style(bot).cyan());
    flush();
}

/// Create a spinner with a message. Respects non-TTY: becomes plain print.
pub fn spinner(msg: &str) -> ProgressBar {
    if !is_interactive() {
        // In non-TTY mode, just print the message immediately and return a hidden bar.
        println!("  → {}", msg);
        flush();
        return ProgressBar::hidden();
    }
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("  {spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message(msg.to_string());
    pb
}

/// Create a progress bar for a multi-step operation.
pub fn progress_bar(total: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  {msg}\n  {bar:40.cyan/blue} {pos}/{len}")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ "),
    );
    pb.set_message(msg.to_string());
    pb
}

/// Finish a spinner with a success message.
pub fn finish_ok(pb: &ProgressBar, msg: &str) {
    if pb.is_hidden() {
        println!("    {} {}", icon_ok(), msg);
        flush();
    } else {
        pb.finish_with_message(format!("{} {}", icon_ok(), msg));
    }
}

/// Finish a spinner with a failure message.
pub fn finish_fail(pb: &ProgressBar, msg: &str) {
    if pb.is_hidden() {
        println!("    {} {}", icon_fail(), msg);
        flush();
    } else {
        pb.finish_with_message(format!("{} {}", icon_fail(), msg));
    }
}

/// Typed-out welcome text animation (slow reveal, only in TTY).
pub fn type_out(text: &str) {
    if !is_interactive() {
        println!("{}", text);
        flush();
        return;
    }
    for ch in text.chars() {
        print!("{}", ch);
        std::io::stdout().flush().ok();
        std::thread::sleep(Duration::from_millis(12));
    }
    println!();
}

/// Clear the terminal (only in TTY).
pub fn clear() {
    if is_interactive() {
        let _ = Term::stdout().clear_screen();
    }
}

/// Interactive prompt: select one from a list.
/// Falls back to returning the first option if not a TTY.
pub fn select<'a>(prompt: &str, options: &[&'a str]) -> Result<&'a str, String> {
    if !is_interactive() {
        return options.first().copied().ok_or_else(|| "no options".into());
    }
    inquire::Select::new(prompt, options.to_vec())
        .prompt()
        .map_err(|e| e.to_string())
}

/// Interactive prompt: confirm yes/no.
pub fn confirm(prompt: &str, default: bool) -> Result<bool, String> {
    if !is_interactive() {
        return Ok(default);
    }
    inquire::Confirm::new(prompt)
        .with_default(default)
        .prompt()
        .map_err(|e| e.to_string())
}

/// Interactive prompt: free text input.
pub fn text_input(prompt: &str, default: Option<&str>) -> Result<String, String> {
    if !is_interactive() {
        return Ok(default.unwrap_or("").to_string());
    }
    let mut p = inquire::Text::new(prompt);
    if let Some(d) = default {
        p = p.with_default(d);
    }
    p.prompt().map_err(|e| e.to_string())
}

/// Print a decorated success message (big checkmark box).
pub fn print_success_banner(msg: &str) {
    println!();
    println!("  {} {}", style("🎉").bold(), style(msg).green().bold());
    println!();
    flush();
}

/// Print a command hint styled as code.
pub fn cmd(text: &str) -> console::StyledObject<String> {
    style(format!("  {}", text)).cyan().bold()
}
