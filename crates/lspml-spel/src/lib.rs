pub mod ast;
pub mod grammar;
pub mod parser;

pub struct Scanner {
    cursor: usize,
    characters: Vec<char>,
    byte_offsets: Vec<usize>,
}

impl Scanner {
    pub(crate) fn new(string: &str) -> Self {
        let characters: Vec<char> = string.chars().collect();
        let mut byte_offsets = Vec::with_capacity(characters.len() + 1);
        let mut offset = 0;
        for character in &characters {
            byte_offsets.push(offset);
            offset += character.len_utf8();
        }
        byte_offsets.push(offset);
        Self {
            cursor: 0,
            characters,
            byte_offsets,
        }
    }

    /// Byte offset of the cursor within the scanned string.
    pub(crate) fn pos(&self) -> u16 {
        self.byte_offsets
            .get(self.cursor)
            .copied()
            .map_or_else(|| self.byte_len(), |offset| offset as u16)
    }

    /// Byte length of the whole scanned string.
    pub(crate) fn byte_len(&self) -> u16 {
        self.byte_offsets.last().copied().unwrap_or(0) as u16
    }

    /// Returns true if further progress is not possible.
    pub(crate) fn is_done(&self) -> bool {
        self.cursor == self.characters.len()
    }

    /// Returns the next character without advancing the cursor.
    /// AKA "lookahead"
    pub(crate) fn peek(&self) -> Option<&char> {
        self.characters.get(self.cursor)
    }

    /// Returns the next character (if available) and advances the cursor.
    pub(crate) fn pop(&mut self) -> Option<&char> {
        let result = self.characters.get(self.cursor);
        if result.is_some() {
            self.cursor += 1;
        }
        result
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
        result
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
        false
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
        result
    }

    /// Byte offset of the last character before the cursor that is not
    /// whitespace. If none is found `0` is returned.
    pub(crate) fn subtract_whitespace(&self) -> u16 {
        let index = self.characters[..self.cursor.min(self.characters.len())]
            .iter()
            .rposition(|character| !character.is_whitespace())
            .unwrap_or(0);
        self.byte_offsets.get(index).copied().unwrap_or(0) as u16
    }

    /// Create a String from all remaining characters.
    /// If `is_done() == true` this returns `""`.
    pub(crate) fn rest(&self) -> String {
        String::from_iter(self.characters[self.cursor..].iter())
    }
}
