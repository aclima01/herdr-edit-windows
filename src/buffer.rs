//! The editable text buffer: a `ropey` rope with a cursor and modeless edit operations.
//!
//! Positions are character indices, not bytes or graphemes: `cursor_col` counts characters
//! from the line start, excluding the line ending. Movement keeps a `goal_col` so vertical
//! motion holds the desired column across short lines. Line endings are normalized to `\n`
//! on load (see `App::open_path`), so only `\n` terminates a line here.

use ropey::Rope;

/// A text buffer with an insertion cursor.
#[derive(Debug)]
pub struct Buffer {
    rope: Rope,
    pub cursor_line: usize,
    pub cursor_col: usize,
    /// The column vertical movement aims for, so moving through a short line and back keeps
    /// the original column.
    goal_col: usize,
    pub modified: bool,
}

impl Buffer {
    /// Build from `text`, cursor at the start, unmodified.
    pub fn from_str(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            cursor_line: 0,
            cursor_col: 0,
            goal_col: 0,
            modified: false,
        }
    }

    /// Total number of lines. A trailing newline yields a final empty line, editable like any other.
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Line `i` as a string, without its trailing newline. Empty for an out-of-range index.
    pub fn line_text(&self, i: usize) -> String {
        if i >= self.rope.len_lines() {
            return String::new();
        }
        let line = self.rope.line(i);
        let mut end = line.len_chars();
        if end > 0 && line.char(end - 1) == '\n' {
            end -= 1;
        }
        line.slice(..end).to_string()
    }

    /// Number of characters in line `i`, excluding its trailing newline.
    pub fn line_len(&self, i: usize) -> usize {
        if i >= self.rope.len_lines() {
            return 0;
        }
        let line = self.rope.line(i);
        let mut n = line.len_chars();
        if n > 0 && line.char(n - 1) == '\n' {
            n -= 1;
        }
        n
    }

    /// The whole buffer as a string, for saving or re-highlighting.
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn clear_modified(&mut self) {
        self.modified = false;
    }

    /// The rope character index of the cursor.
    fn cursor_char(&self) -> usize {
        self.rope.line_to_char(self.cursor_line) + self.cursor_col
    }

    // --- editing -----------------------------------------------------------

    /// Insert `c` at the cursor and step right.
    pub fn insert_char(&mut self, c: char) {
        let idx = self.cursor_char();
        self.rope.insert_char(idx, c);
        self.cursor_col += 1;
        self.goal_col = self.cursor_col;
        self.modified = true;
    }

    /// Split the line at the cursor, moving it to the start of the new line.
    pub fn insert_newline(&mut self) {
        let idx = self.cursor_char();
        self.rope.insert_char(idx, '\n');
        self.cursor_line += 1;
        self.cursor_col = 0;
        self.goal_col = 0;
        self.modified = true;
    }

    /// Delete the character before the cursor, joining with the previous line at column 0.
    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let idx = self.cursor_char();
            self.rope.remove(idx - 1..idx);
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            let prev_len = self.line_len(self.cursor_line - 1);
            let idx = self.cursor_char();
            self.rope.remove(idx - 1..idx); // the newline ending the previous line
            self.cursor_line -= 1;
            self.cursor_col = prev_len;
        } else {
            return;
        }
        self.goal_col = self.cursor_col;
        self.modified = true;
    }

    /// Delete the character at the cursor, or join the next line when at end of line.
    pub fn delete_forward(&mut self) {
        let idx = self.cursor_char();
        if self.cursor_col < self.line_len(self.cursor_line) {
            self.rope.remove(idx..idx + 1);
            self.modified = true;
        } else if self.cursor_line + 1 < self.line_count() {
            self.rope.remove(idx..idx + 1); // the newline ending this line
            self.modified = true;
        }
    }

    // --- movement ----------------------------------------------------------

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.line_len(self.cursor_line);
        }
        self.goal_col = self.cursor_col;
    }

    pub fn move_right(&mut self) {
        if self.cursor_col < self.line_len(self.cursor_line) {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < self.line_count() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
        self.goal_col = self.cursor_col;
    }

    pub fn move_up(&mut self, n: usize) {
        self.cursor_line = self.cursor_line.saturating_sub(n);
        self.cursor_col = self.goal_col.min(self.line_len(self.cursor_line));
    }

    pub fn move_down(&mut self, n: usize) {
        self.cursor_line = (self.cursor_line + n).min(self.line_count().saturating_sub(1));
        self.cursor_col = self.goal_col.min(self.line_len(self.cursor_line));
    }

    pub fn move_home(&mut self) {
        self.cursor_col = 0;
        self.goal_col = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor_col = self.line_len(self.cursor_line);
        self.goal_col = self.cursor_col;
    }
}

#[cfg(test)]
mod tests {
    use super::Buffer;

    #[test]
    fn insert_and_newline_build_text() {
        let mut b = Buffer::from_str("");
        for c in "ab".chars() {
            b.insert_char(c);
        }
        b.insert_newline();
        b.insert_char('c');
        assert_eq!(b.text(), "ab\nc");
        assert_eq!((b.cursor_line, b.cursor_col), (1, 1));
        assert!(b.modified);
    }

    #[test]
    fn backspace_joins_lines() {
        let mut b = Buffer::from_str("ab\ncd");
        b.cursor_line = 1;
        b.cursor_col = 0;
        b.backspace();
        assert_eq!(b.text(), "abcd");
        assert_eq!((b.cursor_line, b.cursor_col), (0, 2));
    }

    #[test]
    fn delete_forward_at_line_end_joins_next() {
        let mut b = Buffer::from_str("ab\ncd");
        b.cursor_col = 2; // end of "ab"
        b.delete_forward();
        assert_eq!(b.text(), "abcd");
    }

    #[test]
    fn vertical_move_holds_goal_column() {
        let mut b = Buffer::from_str("hello\nhi\nworld");
        b.cursor_col = 5; // end of "hello"
        b.goal_col = 5;
        b.move_down(1); // onto "hi", clamped to col 2
        assert_eq!(b.cursor_col, 2);
        b.move_down(1); // onto "world", goal restores col 5
        assert_eq!(b.cursor_col, 5);
    }

    #[test]
    fn line_len_excludes_newline() {
        let b = Buffer::from_str("abc\n");
        assert_eq!(b.line_len(0), 3);
        assert_eq!(b.line_count(), 2); // trailing empty line
        assert_eq!(b.line_len(1), 0);
    }
}
