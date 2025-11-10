//! Terminal Core
//!
//! Cross-platform terminal emulation and PTY management:
//! - PTY creation and management (portable-pty)
//! - VT100/ANSI escape sequence parsing
//! - Terminal session lifecycle
//! - Input/output handling

pub mod pty;
pub mod parser;
pub mod session;

pub use pty::{PtyHandle, PtyConfig};
pub use parser::{AnsiParser, ParsedEvent};
pub use session::{TerminalSession, SessionConfig};

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert!(true);
    }
}
