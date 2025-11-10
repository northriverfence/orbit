//! Terminal WASM - High-performance terminal emulation in WebAssembly
//!
//! This module provides:
//! - ANSI/VT100 escape sequence parsing
//! - Terminal buffer management
//! - Screen rendering
//! - SSH key generation (future)

use wasm_bindgen::prelude::*;
use web_sys::console;

mod parser;
mod buffer;

pub use parser::AnsiParser;
pub use buffer::TerminalBuffer;

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    console::log_1(&"Terminal WASM module initialized".into());
}

/// Get module version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Terminal emulator combining parser and buffer
#[wasm_bindgen]
pub struct Terminal {
    buffer: TerminalBuffer,
    parser: AnsiParser,
}

#[wasm_bindgen]
impl Terminal {
    /// Create a new terminal
    #[wasm_bindgen(constructor)]
    pub fn new(cols: u16, rows: u16) -> Self {
        console::log_1(&format!("Creating terminal: {}x{}", cols, rows).into());

        Self {
            buffer: TerminalBuffer::new(cols, rows),
            parser: AnsiParser::new(),
        }
    }

    /// Write data to terminal (processes ANSI sequences)
    pub fn write(&mut self, data: &str) -> Result<(), JsValue> {
        self.parser.parse(data, &mut self.buffer);
        Ok(())
    }

    /// Write raw bytes (as Uint8Array from JavaScript)
    pub fn write_bytes(&mut self, data: &[u8]) -> Result<(), JsValue> {
        match std::str::from_utf8(data) {
            Ok(text) => {
                self.parser.parse(text, &mut self.buffer);
                Ok(())
            }
            Err(e) => Err(JsValue::from_str(&format!("Invalid UTF-8: {}", e))),
        }
    }

    /// Get terminal dimensions
    pub fn cols(&self) -> u16 {
        self.buffer.cols()
    }

    pub fn rows(&self) -> u16 {
        self.buffer.rows()
    }

    /// Resize terminal
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.buffer.resize(cols, rows);
    }

    /// Get cursor position
    pub fn cursor_col(&self) -> u16 {
        self.buffer.cursor_col()
    }

    pub fn cursor_row(&self) -> u16 {
        self.buffer.cursor_row()
    }

    /// Get visible lines as JSON
    pub fn get_lines_json(&self) -> String {
        self.buffer.get_lines_json()
    }

    /// Get entire screen as text
    pub fn get_screen_text(&self) -> String {
        self.buffer.get_screen_text()
    }

    /// Clear screen
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Reset terminal state
    pub fn reset(&mut self) {
        self.buffer.reset();
        self.parser.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_terminal_creation() {
        let term = Terminal::new(80, 24);
        assert_eq!(term.cols(), 80);
        assert_eq!(term.rows(), 24);
    }

    #[wasm_bindgen_test]
    fn test_write_text() {
        let mut term = Terminal::new(80, 24);
        term.write("Hello, World!").unwrap();
        let screen = term.get_screen_text();
        assert!(screen.contains("Hello, World!"));
    }
}
