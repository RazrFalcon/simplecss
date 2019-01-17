// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::str;

use error::{Error, ErrorPos};

/// Streaming interface for `&[u8]` data.
#[derive(PartialEq,Clone,Copy)]
pub struct Stream<'a> {
    text: &'a [u8],
    pos: usize,
    end: usize,
}

#[inline]
fn is_letter(c: u8) -> bool {
    match c {
        b'A'...b'Z' => true,
        b'a'...b'z' => true,
        _ => false,
    }
}

#[inline]
fn is_digit(c: u8) -> bool {
    match c {
        b'0'...b'9' => true,
        _ => false,
    }
}

#[inline]
pub fn is_space(c: u8) -> bool {
    match c {
          b' '
        | b'\t'
        | b'\n'
        | b'\r' => true,
        _ => false,
    }
}

impl<'a> Stream<'a> {
    /// Constructs a new `Stream` from data.
    #[inline]
    pub fn new(text: &[u8]) -> Stream {
        Stream {
            text: text,
            pos: 0,
            end: text.len(),
        }
    }

    /// Constructs a new `Stream` from data.
    #[inline]
    pub fn new_bound(text: &[u8], start: usize, end: usize) -> Stream {
        assert!(start < end);

        Stream {
            text: text,
            pos: start,
            end: end,
        }
    }

    /// Returns current position.
    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Returns `true` if we are at the end of the stream.
    ///
    /// Any [`pos()`] value larger than original text length indicates stream end.
    ///
    /// Accessing stream after reaching end via safe methods will produce `simplecss::Error`.
    ///
    /// Accessing stream after reaching end via unsafe/_raw methods will produce
    /// rust bound checking error.
    ///
    /// [`pos()`]: #method.pos
    #[inline]
    pub fn at_end(&self) -> bool {
        self.pos >= self.end
    }

    /// Returns a char from current stream position.
    ///
    /// # Errors
    ///
    /// Returns `Error::UnexpectedEndOfStream` if we at the end of the stream.
    #[inline]
    pub fn curr_char(&self) -> Result<u8, Error> {
        if self.at_end() {
            return Err(self.gen_end_of_stream_error());
        }

        Ok(self.text[self.pos])
    }

    /// Unsafe version of [`curr_char()`].
    ///
    /// [`curr_char()`]: #method.curr_char
    #[inline]
    pub fn curr_char_raw(&self) -> u8 {
        self.text[self.pos]
    }

    /// Compares selected char with char from current stream position.
    ///
    /// # Errors
    ///
    /// Returns `Error::UnexpectedEndOfStream` if we at the end of the stream.
    #[inline]
    pub fn is_char_eq(&self, c: u8) -> Result<bool, Error> {
        if self.at_end() {
            return Err(self.gen_end_of_stream_error());
        }

        Ok(self.curr_char_raw() == c)
    }

    /// Advance by `n` chars.
    ///
    /// # Errors
    ///
    /// Returns `Error::AdvanceError` if new position beyond stream end.
    /// ```
    #[inline]
    pub fn advance(&mut self, n: usize) -> Result<(), Error> {
        try!(self.adv_bound_check(n));
        self.pos += n;
        Ok(())
    }

    /// Unsafe version of [`advance()`].
    ///
    /// [`advance()`]: #method.advance
    #[inline]
    pub fn advance_raw(&mut self, n: usize) {
        debug_assert!(self.pos + n <= self.end);
        self.pos += n;
    }

    /// Checks that char at the current position is (white)space.
    ///
    /// Accepted chars: ' ', '\n', '\r', '\t'.
    #[inline]
    pub fn is_space_raw(&self) -> bool {
        is_space(self.curr_char_raw())
    }

    pub fn is_ident_raw(&self) -> bool {
        let c = self.curr_char_raw();

           is_digit(c)
        || is_letter(c)
        || c == b'_'
        || c == b'-'
    }

    /// Skips (white)space's.
    #[inline]
    pub fn skip_spaces(&mut self) {
        while !self.at_end() && self.is_space_raw() {
            self.advance_raw(1);
        }
    }

    #[inline]
    fn get_char_raw(&self, pos: usize) -> u8 {
        self.text[pos]
    }

    /// Calculates length to the selected char.
    #[inline]
    pub fn length_to(&self, c: u8) -> Result<usize, Error> {
        let mut n = 0;
        while self.pos + n != self.end {
            if self.get_char_raw(self.pos + n) == c {
                return Ok(n);
            } else {
                n += 1;
            }
        }

        Err(self.gen_end_of_stream_error())
    }

    pub fn length_to_either(&self, search_chars: &[u8]) -> Result<usize, Error> {
        let mut n = 0;
        while self.pos + n != self.end {
            let c = self.get_char_raw(self.pos + n);
            if search_chars.contains(&c) {
                return Ok(n);
            } else {
                n += 1;
            }
        }

        Err(self.gen_end_of_stream_error())
    }

    /// Returns reference to data with length `len` and advance stream to the same length.
    #[inline]
    pub fn read_raw_str(&mut self, len: usize) -> &'a str {
        let s = &self.text[self.pos..(self.pos + len)];
        self.advance_raw(s.len());
        str::from_utf8(s).unwrap()
    }

    /// Returns data of stream within selected region.
    #[inline]
    pub fn slice_region_raw_str(&self, start: usize, end: usize) -> &'a str {
        str::from_utf8(&self.text[start..end]).unwrap()
    }

    fn calc_current_row(&self) -> usize {
        let mut row = 1;
        row += self.text.iter().take(self.pos).filter(|c| **c == b'\n').count();
        row
    }

    fn calc_current_col(&self) -> usize {
        let mut col = 1;
        for n in 0..self.pos {
            if n > 0 && self.text[n-1] == b'\n' {
                col = 2;
            } else {
                col += 1;
            }
        }

        col
    }

    /// Calculates a current absolute position.
    pub fn gen_error_pos(&self) -> ErrorPos {
        let row = self.calc_current_row();
        let col = self.calc_current_col();
        ErrorPos::new(row, col)
    }

    /// Generates a new `UnexpectedEndOfStream` error from the current position.
    pub fn gen_end_of_stream_error(&self) -> Error {
        Error::UnexpectedEndOfStream(self.gen_error_pos())
    }

    fn adv_bound_check(&self, n: usize) -> Result<(), Error> {
        let new_pos = self.pos + n;
        if new_pos > self.end {
            return Err(Error::InvalidAdvance{
                expected: new_pos as isize,
                total: self.end,
                pos: self.gen_error_pos(),
            });
        }

        Ok(())
    }
}
