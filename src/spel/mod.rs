// use anyhow::{Error, Result};
mod ast;
pub(crate) mod parser;

pub(crate) struct Scanner {
    cursor: usize,
    characters: Vec<char>,
}

// #[derive(Debug, PartialEq, Clone)]
// pub(crate) enum Action<T> {
//     /// If next iteration returns None, return T without advancing
//     /// the cursor.
//     Request(T),

//     /// If the next iteration returns None, return None without advancing
//     /// the cursor.
//     Require,

//     /// Immediately advance the cursor and return T.
//     Return(T),
// }

impl Scanner {
    pub(crate) fn new(string: &str) -> Self {
        return Self {
            cursor: 0,
            characters: string.chars().collect(),
        };
    }

    // /// Returns the current cursor. Useful for reporting errors.
    // pub(crate) fn cursor(&self) -> usize {
    //     return self.cursor;
    // }

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

    // /// Returns true if the `target` is found at the current cursor position,
    // /// and advances the cursor.
    // /// Otherwise, returns false leaving the cursor unchanged.
    // pub(crate) fn take_if(&mut self, cb: impl FnOnce(&char) -> bool) -> bool {
    //     let result = self.characters.get(self.cursor).is_some_and(cb);
    //     if result {
    //         self.cursor += 1;
    //     }
    //     return result;
    // }

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

    // /// Iteratively directs the advancement of the cursor and the return
    // /// of translated values.
    // pub(crate) fn scan<T>(
    //     &mut self,
    //     cb: impl Fn(&str) -> Option<Action<T>>,
    // ) -> Result<Option<T>, Error> {
    //     let mut sequence = String::new();
    //     let mut require = false;
    //     let mut request = None;
    //     loop {
    //         match self.characters.get(self.cursor) {
    //             Some(target) => {
    //                 sequence.push(*target);
    //                 match cb(&sequence) {
    //                     Some(Action::Return(result)) => {
    //                         self.cursor += 1;
    //                         return Ok(Some(result));
    //                     }
    //                     Some(Action::Request(result)) => {
    //                         self.cursor += 1;
    //                         require = false;
    //                         request = Some(result);
    //                     }
    //                     Some(Action::Require) => {
    //                         self.cursor += 1;
    //                         require = true;
    //                     }
    //                     None if require => {
    //                         return Err(anyhow::anyhow!("unexpected character \"{}\"", target))
    //                     }
    //                     None => return Ok(request),
    //                 }
    //             }
    //             None if require => return Err(anyhow::anyhow!("unexpected end")),
    //             None => return Ok(request),
    //         }
    //     }
    // }

    /// Invoke `cb` once. If the result is not `None`, return it and advance
    /// the cursor. Otherwise, return None and leave the cursor unchanged.
    pub(crate) fn transform<T>(&mut self, cb: impl FnOnce(&char) -> Option<T>) -> Option<T> {
        let result = self.characters.get(self.cursor).and_then(cb);
        if result.is_some() {
            self.cursor += 1;
        }
        return result;
    }
}
