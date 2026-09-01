use std::io::{Stdout, stdout};

use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

/// Terminal claimed for full-screen drawing and released when the session is dropped.
pub struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    /// Claim the terminal for full-screen drawing by enabling raw mode and entering
    /// the alternate screen, then build the drawing surface over standard output.
    pub fn open() -> Result<Self, std::io::Error> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        Ok(Self { terminal })
    }

    /// Borrow the drawing surface so a run loop can render frames onto it.
    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalSession {
    /// Return the terminal to the shell, even while unwinding, by disabling raw mode
    /// and leaving the alternate screen; failures are discarded because a destructor
    /// has no caller to report them to.
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
    }
}
