pub mod ast;
pub mod grammar;
pub mod parser;

pub struct Scanner {
    cursor: usize,
    characters: Vec<char>,
}

impl Scanner {
    pub(crate) fn new(string: &str) -> Self {
        return Self {
            cursor: 0,
            characters: string.chars().collect(),
        };
    }

    /// Returns true if further progress is not possible.
    pub(crate) fn is_done(&self) -> bool {
        return self.cursor == self.characters.len();
    }

    /// Returns the next character without advancing the cursor.
    /// AKA "lookahead"
    pub(crate) fn peek(&self) -> Option<&char> {
        return self.characters.get(self.cursor);
    }

    /// Returns the next character (if available) and advances the cursor.
    pub(crate) fn pop(&mut self) -> Option<&char> {
        let result = self.characters.get(self.cursor);
        if result.is_some() {
            self.cursor += 1;
        }
        return result;
    }

    /// Returns true if the `target` is found at the current cursor position,
    /// and advances the cursor.
    /// Otherwise, returns false leaving the cursor unchanged.
    pub(crate) fn take(&mut self, target: &char) -> bool {
        let result = self
            .characters
            .get(self.cursor)
            .is_some_and(|c| c == target);
        if result {
            self.cursor += 1;
        }
        return result;
    }

    /// Returns true if the `target` is found at the current cursor position,
    /// and advances the cursor.
    /// Otherwise, returns false leaving the cursor unchanged.
    pub(crate) fn take_str(&mut self, target: &str) -> bool {
        let end = self.cursor + target.len();
        if self.characters.len() < end {
            return false;
        }
        let actual: String = String::from_iter(self.characters[self.cursor..end].iter());
        if actual == target {
            self.cursor = end;
            return true;
        }
        return false;
    }

    /// Returns true if the `target` any whitespace was skipped and the cursor
    /// advanced.
    /// Otherwise, returns false leaving the cursor unchanged.
    pub(crate) fn skip_whitespace(&mut self) -> bool {
        let start = self.cursor;
        loop {
            if !self
                .characters
                .get(self.cursor)
                .is_some_and(|c| c.is_whitespace())
            {
                return self.cursor > start;
            }
            self.cursor += 1;
        }
    }

    /// Invoke `cb` once. If the result is not `None`, return it and advance
    /// the cursor. Otherwise, return None and leave the cursor unchanged.
    pub(crate) fn transform<T>(&mut self, cb: impl FnOnce(&char) -> Option<T>) -> Option<T> {
        let result = self.characters.get(self.cursor).and_then(cb);
        if result.is_some() {
            self.cursor += 1;
        }
        return result;
    }

    /// Finds the last position, that is not whitespace.
    /// If none is found `0` is returned.
    pub(crate) fn subtract_whitespace(&self) -> usize {
        let mut cursor = match self.cursor {
            0 => 0,
            c => c - 1,
        };
        loop {
            if !self
                .characters
                .get(cursor)
                .is_some_and(|c| c.is_whitespace())
            {
                return cursor;
            }
            cursor -= 1;
        }
    }

    /// Create a String from all remaining characters.
    /// If `is_done() == true` this returns `""`.
    pub(crate) fn rest(&self) -> String {
        return String::from_iter(self.characters[self.cursor..].iter());
    }
}
