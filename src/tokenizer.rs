// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use stream;
use stream::Stream;
use error::Error;

/// CSS combinator.
#[derive(PartialEq,Debug)]
pub enum Combinator {
    /// Descendant selector
    Space,
    /// Child selector
    GreaterThan,
    /// Adjacent sibling selector
    Plus,
}

/// CSS token.
#[derive(PartialEq,Debug)]
pub enum Token<'a> {
    /// Universal selector
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#universal-selector
    UniversalSelector,
    /// Type selector
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#type-selectors
    TypeSelector(&'a str),
    /// ID selector
    ///
    /// Value contains ident without `#`.
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#id-selectors
    IdSelector(&'a str),
    /// Class selector
    ///
    /// Value contains ident without `.`.
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#class-html
    ClassSelector(&'a str),
    /// Attribute selector
    ///
    /// We do not parse it's content yet, so value contains everything between `[]`.
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#attribute-selectors
    AttributeSelector(&'a str),
    /// Pseudo-class
    ///
    /// Value contains ident without `:`.
    /// Selector: `"nth-child"`, value: The thing between the braces - `Some("3")`
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#pseudo-class-selectors
    PseudoClass { selector: &'a str, value: Option<&'a str> },
    /// `Combinator`
    Combinator(Combinator),
    /// Rules separator
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#grouping
    Comma,
    /// Block start
    ///
    /// Indicates `{`.
    ///
    /// https://www.w3.org/TR/CSS21/syndata.html#rule-sets
    BlockStart,
    /// Block end
    ///
    /// Indicates `}`.
    ///
    /// https://www.w3.org/TR/CSS21/syndata.html#rule-sets
    BlockEnd,
    /// Declaration
    ///
    /// Contains property name and property value.
    ///
    /// https://www.w3.org/TR/CSS21/syndata.html#declaration
    Declaration(&'a str, &'a str),
    /// End of stream
    ///
    /// Parsing is finished.
    EndOfStream,
}

#[derive(PartialEq)]
enum State {
    Rule,
    Declaration,
}

/// CSS tokenizer.
pub struct Tokenizer<'a> {
    stream: Stream<'a>,
    state: State,
    after_selector: bool,
    at_start: bool,
}

impl<'a> Tokenizer<'a> {
    /// Constructs a new `Tokenizer`.
    pub fn new(text: &str) -> Tokenizer {
        Tokenizer {
            stream: Stream::new(text.as_bytes()),
            state: State::Rule,
            after_selector: false,
            at_start: true,
        }
    }

    /// Constructs a new bounded `Tokenizer`.
    ///
    /// It can be useful if CSS data is inside other data, like HTML.
    /// Using this method you will get an absolute error positions and not relative,
    /// like when using [`new()`].
    ///
    /// [`new()`]: #method.new
    pub fn new_bound(text: &str, start: usize, end: usize) -> Tokenizer {
        Tokenizer {
            stream: Stream::new_bound(text.as_bytes(), start, end),
            state: State::Rule,
            after_selector: false,
            at_start: true,
        }
    }

    /// Returns a current position in the text.
    pub fn pos(&self) -> usize {
        self.stream.pos()
    }

    /// Parses a next token.
    pub fn parse_next(&mut self) -> Result<Token<'a>, Error> {
        if self.at_start {
            self.stream.skip_spaces();
            self.at_start = false;
        }

        if self.stream.at_end() {
            return Ok(Token::EndOfStream);
        }

        match self.state {
            State::Rule         => self.consume_rule(),
            State::Declaration  => self.consume_declaration(),
        }
    }

    fn consume_rule(&mut self) -> Result<Token<'a>, Error> {
        match self.stream.curr_char_raw() {
            b'@' => {
                return Err(Error::UnsupportedToken(self.stream.gen_error_pos()));
            }
            b'#' => {
                self.after_selector = true;
                self.stream.advance_raw(1);
                let s = try!(self.consume_ident());
                return Ok(Token::IdSelector(s));
            }
            b'.' => {
                self.after_selector = true;
                self.stream.advance_raw(1);
                let s = try!(self.consume_ident());
                return Ok(Token::ClassSelector(s));
            }
            b'*' => {
                self.after_selector = true;
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::UniversalSelector);
            }
            b':' => {
                self.after_selector = true;
                self.stream.advance_raw(1);
                let s = try!(self.consume_ident());

                if self.stream.length_to(b'(') == Ok(0) {
                    // Item is a thing()
                    self.stream.advance_raw(1); // (
                    let inner_len = self.stream.length_to(b')')?;
                    let inner = self.stream.read_raw_str(inner_len);
                    self.stream.advance_raw(1); // )
                    return Ok(Token::PseudoClass { selector: s, value: Some(inner) });
                } else {
                    return Ok(Token::PseudoClass { selector: s, value: None });
                }
            }
            b'[' => {
                self.after_selector = true;
                self.stream.advance_raw(1);
                let len = try!(self.stream.length_to(b']'));
                let s = self.stream.read_raw_str(len);
                self.stream.advance_raw(1); // ]
                self.stream.skip_spaces();
                return Ok(Token::AttributeSelector(s));
            }
            b',' => {
                self.after_selector = false;
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::Comma);
            }
            b'{' => {
                self.after_selector = false;
                self.state = State::Declaration;
                self.stream.advance_raw(1);
                return Ok(Token::BlockStart);
            }
            b'>' => {
                if self.after_selector {
                    self.after_selector = false;
                    self.stream.advance_raw(1);
                    self.stream.skip_spaces();
                    return Ok(Token::Combinator(Combinator::GreaterThan));
                } else {
                    return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                }
            }
            b'+' => {
                if self.after_selector {
                    self.after_selector = false;
                    self.stream.advance_raw(1);
                    self.stream.skip_spaces();
                    return Ok(Token::Combinator(Combinator::Plus));
                } else {
                    return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                }
            }
            b'/' => {
                if try!(self.consume_comment()) {
                    return self.parse_next();
                } else {
                    return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                }
            }
            _ => {
                if self.stream.is_space_raw() {
                    self.stream.skip_spaces();

                    if !self.after_selector {
                        return self.parse_next();
                    }

                    return match try!(self.stream.curr_char()) {
                        b'{' | b'/' | b'>' | b'+' | b'*' => self.parse_next(),
                        _ => {
                            self.after_selector = false;
                            Ok(Token::Combinator(Combinator::Space))
                        }
                    };
                }

                self.after_selector = true;
                let s = try!(self.consume_ident());
                return Ok(Token::TypeSelector(s));
            }
        }
    }

    fn consume_declaration(&mut self) -> Result<Token<'a>, Error> {
        self.stream.skip_spaces();

        match self.stream.curr_char_raw() {
            b'}' => {
                self.state = State::Rule;
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::BlockEnd);
            }
            b'/' => {
                if try!(self.consume_comment()) {
                    return self.parse_next();
                } else {
                    return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                }
            }
            _ => {
                let name = try!(self.consume_ident());

                self.stream.skip_spaces();

                if try!(self.stream.is_char_eq(b'/')) {
                    if !try!(self.consume_comment()) {
                        return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                    }
                }

                self.stream.advance_raw(1); // :
                self.stream.skip_spaces();

                if try!(self.stream.is_char_eq(b'/')) {
                    if !try!(self.consume_comment()) {
                        return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                    }
                }

                let len = try!(self.stream.length_to_either(b';', b'}'));
                if len == 0 {
                    return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                }

                let mut value = self.stream.read_raw_str(len);
                // trim spaces at the end of the value
                if let Some(p) = value.as_bytes().iter().rposition(|c| !stream::is_space(*c)) {
                    value = &value[0..(p + 1)];
                }

                self.stream.skip_spaces();
                while try!(self.stream.is_char_eq(b';')) {
                    self.stream.advance_raw(1);
                    self.stream.skip_spaces();
                }

                return Ok(Token::Declaration(name, value));
            }
        }
    }

    fn consume_ident(&mut self) -> Result<&'a str, Error> {
        let start = self.stream.pos();

        while !self.stream.at_end() {
            if self.stream.is_ident_raw() {
                try!(self.stream.advance(1));
            } else {
                break;
            }
        }

        if start == self.stream.pos() {
            return Err(Error::UnknownToken(self.stream.gen_error_pos()));
        }

        let s = self.stream.slice_region_raw_str(start, self.stream.pos());
        Ok(s)
    }

    fn consume_comment(&mut self) -> Result<bool, Error>  {
        self.stream.advance_raw(1);

        if try!(self.stream.is_char_eq(b'*')) {
            self.stream.advance_raw(1); // *

            while !self.stream.at_end() {
                let len = try!(self.stream.length_to(b'*'));
                try!(self.stream.advance(len + 1));
                if try!(self.stream.is_char_eq(b'/')) {
                    self.stream.advance_raw(1);
                    break;
                }
            }

            return Ok(true);
        } else {
            return Ok(false);
        }
    }
}
