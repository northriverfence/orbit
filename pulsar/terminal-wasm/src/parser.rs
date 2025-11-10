//! ANSI/VT100 escape sequence parser
//!
//! High-performance parser using the vte crate

use vte::{Perform, Parser as VteParser};
use crate::buffer::TerminalBuffer;

pub struct AnsiParser {
    vte_parser: VteParser,
}

impl AnsiParser {
    pub fn new() -> Self {
        Self {
            vte_parser: VteParser::new(),
        }
    }

    pub fn parse(&mut self, data: &str, buffer: &mut TerminalBuffer) {
        let mut performer = BufferPerformer { buffer };
        for byte in data.bytes() {
            self.vte_parser.advance(&mut performer, byte);
        }
    }

    pub fn reset(&mut self) {
        self.vte_parser = VteParser::new();
    }
}

/// Performer that writes to TerminalBuffer
struct BufferPerformer<'a> {
    buffer: &'a mut TerminalBuffer,
}

impl<'a> Perform for BufferPerformer<'a> {
    fn print(&mut self, c: char) {
        self.buffer.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.buffer.newline(),
            b'\r' => self.buffer.carriage_return(),
            b'\t' => self.buffer.tab(),
            0x08 => self.buffer.backspace(), // Backspace
            0x07 => {}, // Bell - ignore
            _ => {},
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // DCS sequences - not implemented yet
    }

    fn put(&mut self, _byte: u8) {
        // DCS data - not implemented yet
    }

    fn unhook(&mut self) {
        // DCS end - not implemented yet
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // OSC sequences - not implemented yet
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        match action {
            'A' => {
                // Cursor up
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1) as u16;
                self.buffer.cursor_up(n);
            }
            'B' => {
                // Cursor down
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1) as u16;
                self.buffer.cursor_down(n);
            }
            'C' => {
                // Cursor forward
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1) as u16;
                self.buffer.cursor_forward(n);
            }
            'D' => {
                // Cursor backward
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1) as u16;
                self.buffer.cursor_backward(n);
            }
            'H' | 'f' => {
                // Cursor position
                let mut iter = params.iter();
                let row = iter.next().map(|p| p[0]).unwrap_or(1) as u16;
                let col = iter.next().map(|p| p[0]).unwrap_or(1) as u16;
                self.buffer.cursor_goto(col.saturating_sub(1), row.saturating_sub(1));
            }
            'J' => {
                // Erase in display
                let n = params.iter().next().map(|p| p[0]).unwrap_or(0);
                match n {
                    0 => self.buffer.clear_below_cursor(),
                    1 => self.buffer.clear_above_cursor(),
                    2 => self.buffer.clear(),
                    _ => {},
                }
            }
            'K' => {
                // Erase in line
                let n = params.iter().next().map(|p| p[0]).unwrap_or(0);
                match n {
                    0 => self.buffer.clear_line_right(),
                    1 => self.buffer.clear_line_left(),
                    2 => self.buffer.clear_line(),
                    _ => {},
                }
            }
            'm' => {
                // SGR - Select Graphic Rendition (colors, styles)
                if params.is_empty() {
                    self.buffer.reset_style();
                } else {
                    for param in params.iter() {
                        let n = param[0];
                        match n {
                            0 => self.buffer.reset_style(),
                            1 => self.buffer.set_bold(true),
                            2 => self.buffer.set_dim(true),
                            3 => self.buffer.set_italic(true),
                            4 => self.buffer.set_underline(true),
                            7 => self.buffer.set_inverse(true),
                            22 => {
                                self.buffer.set_bold(false);
                                self.buffer.set_dim(false);
                            }
                            23 => self.buffer.set_italic(false),
                            24 => self.buffer.set_underline(false),
                            27 => self.buffer.set_inverse(false),
                            30..=37 => self.buffer.set_fg_color(n - 30),
                            40..=47 => self.buffer.set_bg_color(n - 40),
                            _ => {},
                        }
                    }
                }
            }
            _ => {
                // Unhandled CSI sequence
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // ESC sequences - not implemented yet
    }
}
