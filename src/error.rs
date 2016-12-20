// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;

/// Position of an error.
///
/// Position indicates row/line and column. Starting positions is 1:1.
#[derive(Clone,Copy,PartialEq)]
pub struct ErrorPos {
    #[allow(missing_docs)]
    pub row: usize,
    #[allow(missing_docs)]
    pub col: usize,
}

impl ErrorPos {
    /// Constructs a new error position.
    pub fn new(row: usize, col: usize) -> ErrorPos {
        ErrorPos {
            row: row,
            col: col,
        }
    }
}

impl fmt::Debug for ErrorPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", &self.row, &self.col)
    }
}

/// List of all supported errors.
#[derive(Clone,Copy,PartialEq)]
pub enum Error {
    /// The steam ended earlier than we expected.
    ///
    /// Should only appear on invalid input data.
    UnexpectedEndOfStream(ErrorPos),
    /// Can appear during moving along the data stream.
    InvalidAdvance {
        /// The advance step.
        expected: isize,
        /// Full length of the steam.
        total: usize,
        /// Absolute stream position.
        pos: ErrorPos,
    },
    /// Unsupported token.
    UnsupportedToken(ErrorPos),
    /// Unknown token.
    UnknownToken(ErrorPos),
}

impl fmt::Debug for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnexpectedEndOfStream(ref pos) =>
                write!(f, "Unexpected end of stream at {:?}", pos),
            Error::InvalidAdvance{ref expected, ref total, ref pos} =>
                write!(f, "Attempt to advance to the pos {} from {:?}, but total len is {}",
                       expected, pos, total),
            Error::UnsupportedToken(ref pos) =>
                write!(f, "Unsupported token at {:?}", pos),
            Error::UnknownToken(ref pos) =>
                write!(f, "Unknown token at {:?}", pos),
        }
    }
}
