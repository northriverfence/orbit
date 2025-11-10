//! Terminal buffer - stores screen content and cursor state

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharCell {
    pub ch: char,
    pub fg: u8,
    pub bg: u8,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
}

impl Default for CharCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: 7, // White
            bg: 0, // Black
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            inverse: false,
        }
    }
}

pub struct TerminalBuffer {
    cols: u16,
    rows: u16,
    cursor_col: u16,
    cursor_row: u16,
    cells: Vec<CharCell>,
    current_style: CharCell,
}

impl TerminalBuffer {
    pub fn new(cols: u16, rows: u16) -> Self {
        let size = (cols as usize) * (rows as usize);
        Self {
            cols,
            rows,
            cursor_col: 0,
            cursor_row: 0,
            cells: vec![CharCell::default(); size],
            current_style: CharCell::default(),
        }
    }

    pub fn cols(&self) -> u16 {
        self.cols
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }

    pub fn cursor_col(&self) -> u16 {
        self.cursor_col
    }

    pub fn cursor_row(&self) -> u16 {
        self.cursor_row
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        let new_size = (cols as usize) * (rows as usize);
        self.cells.resize(new_size, CharCell::default());
        self.cols = cols;
        self.rows = rows;
        self.cursor_col = self.cursor_col.min(cols.saturating_sub(1));
        self.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
    }

    fn index(&self, col: u16, row: u16) -> usize {
        (row as usize) * (self.cols as usize) + (col as usize)
    }

    pub fn put_char(&mut self, ch: char) {
        if self.cursor_col >= self.cols {
            self.newline();
        }

        let idx = self.index(self.cursor_col, self.cursor_row);
        if idx < self.cells.len() {
            self.cells[idx] = CharCell {
                ch,
                ..self.current_style
            };
        }

        self.cursor_col += 1;
    }

    pub fn newline(&mut self) {
        self.cursor_col = 0;
        if self.cursor_row + 1 < self.rows {
            self.cursor_row += 1;
        } else {
            // Scroll up
            self.scroll_up(1);
        }
    }

    pub fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    pub fn tab(&mut self) {
        // Move to next multiple of 8
        let next_tab = ((self.cursor_col / 8) + 1) * 8;
        self.cursor_col = next_tab.min(self.cols - 1);
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    pub fn cursor_up(&mut self, n: u16) {
        self.cursor_row = self.cursor_row.saturating_sub(n);
    }

    pub fn cursor_down(&mut self, n: u16) {
        self.cursor_row = (self.cursor_row + n).min(self.rows - 1);
    }

    pub fn cursor_forward(&mut self, n: u16) {
        self.cursor_col = (self.cursor_col + n).min(self.cols - 1);
    }

    pub fn cursor_backward(&mut self, n: u16) {
        self.cursor_col = self.cursor_col.saturating_sub(n);
    }

    pub fn cursor_goto(&mut self, col: u16, row: u16) {
        self.cursor_col = col.min(self.cols - 1);
        self.cursor_row = row.min(self.rows - 1);
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = CharCell::default();
        }
        self.cursor_col = 0;
        self.cursor_row = 0;
    }

    pub fn clear_line(&mut self) {
        let row_start = self.index(0, self.cursor_row);
        let row_end = row_start + (self.cols as usize);
        for cell in &mut self.cells[row_start..row_end] {
            *cell = CharCell::default();
        }
    }

    pub fn clear_line_right(&mut self) {
        let start = self.index(self.cursor_col, self.cursor_row);
        let end = self.index(self.cols - 1, self.cursor_row) + 1;
        for cell in &mut self.cells[start..end] {
            *cell = CharCell::default();
        }
    }

    pub fn clear_line_left(&mut self) {
        let start = self.index(0, self.cursor_row);
        let end = self.index(self.cursor_col, self.cursor_row) + 1;
        for cell in &mut self.cells[start..end] {
            *cell = CharCell::default();
        }
    }

    pub fn clear_below_cursor(&mut self) {
        self.clear_line_right();
        for row in (self.cursor_row + 1)..self.rows {
            let start = self.index(0, row);
            let end = start + (self.cols as usize);
            for cell in &mut self.cells[start..end] {
                *cell = CharCell::default();
            }
        }
    }

    pub fn clear_above_cursor(&mut self) {
        self.clear_line_left();
        for row in 0..self.cursor_row {
            let start = self.index(0, row);
            let end = start + (self.cols as usize);
            for cell in &mut self.cells[start..end] {
                *cell = CharCell::default();
            }
        }
    }

    fn scroll_up(&mut self, n: u16) {
        let n = n as usize;
        let cols = self.cols as usize;
        let scroll_amount = n * cols;

        // Move lines up
        self.cells.copy_within(scroll_amount.., 0);

        // Clear bottom lines
        let clear_start = self.cells.len() - scroll_amount;
        for cell in &mut self.cells[clear_start..] {
            *cell = CharCell::default();
        }
    }

    // Style methods
    pub fn reset_style(&mut self) {
        self.current_style = CharCell::default();
    }

    pub fn set_bold(&mut self, bold: bool) {
        self.current_style.bold = bold;
    }

    pub fn set_dim(&mut self, dim: bool) {
        self.current_style.dim = dim;
    }

    pub fn set_italic(&mut self, italic: bool) {
        self.current_style.italic = italic;
    }

    pub fn set_underline(&mut self, underline: bool) {
        self.current_style.underline = underline;
    }

    pub fn set_inverse(&mut self, inverse: bool) {
        self.current_style.inverse = inverse;
    }

    pub fn set_fg_color(&mut self, color: u16) {
        self.current_style.fg = color as u8;
    }

    pub fn set_bg_color(&mut self, color: u16) {
        self.current_style.bg = color as u8;
    }

    pub fn reset(&mut self) {
        self.clear();
        self.current_style = CharCell::default();
    }

    // Export methods
    pub fn get_lines_json(&self) -> String {
        let mut lines = Vec::new();
        for row in 0..self.rows {
            let start = self.index(0, row);
            let end = start + (self.cols as usize);
            let line: Vec<&CharCell> = self.cells[start..end].iter().collect();
            lines.push(line);
        }
        serde_json::to_string(&lines).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn get_screen_text(&self) -> String {
        let mut result = String::new();
        for row in 0..self.rows {
            for col in 0..self.cols {
                let idx = self.index(col, row);
                if idx < self.cells.len() {
                    result.push(self.cells[idx].ch);
                }
            }
            if row < self.rows - 1 {
                result.push('\n');
            }
        }
        result
    }
}
