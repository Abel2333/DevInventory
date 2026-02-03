use std::{
    io::{self, Stdout, stdout},
    sync::Once,
};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};

static PANIC_HOOK: Once = Once::new();

/// A short alias of terminal type
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialze the terminal
pub fn init() -> std::io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    set_panic_hook();
    Terminal::new(CrosstermBackend::new(stdout()))
}

/// Install a one-time panic hook
fn set_panic_hook() {
    PANIC_HOOK.call_once(|| {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_hook| {
            let _ = restore();
            hook(panic_hook)
        }));
    })
}

/// Restore the terminal
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
