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
    /// `@` rule (excluding the `@` sign itself). The content is not parsed,
    /// for example `@keyframes mymove` = `AtRule("keyframes"), AtStr("mymove")`.
    AtRule(&'a str),
    /// Raw Str inside of block
    DeclarationStr(&'a str),
    /// String following an @rule
    AtStr(&'a str),
    /// Same as PseudoClass, but with two colons (`::thing`).
    DoublePseudoClass { selector: &'a str, value: Option<&'a str> },
    /// End of stream
    ///
    /// Parsing is finished.
    EndOfStream,
}

#[derive(PartialEq)]
enum State {
    Rule,
    Declaration,
    DeclarationRule,
}

/// CSS tokenizer.
pub struct Tokenizer<'a> {
    stream: Stream<'a>,
    state: State,
    after_selector: bool,
    has_at_rule: bool,
    at_start: bool,
}

impl<'a> Tokenizer<'a> {
    /// Constructs a new `Tokenizer`.
    pub fn new(text: &str) -> Tokenizer {
        Tokenizer {
            stream: Stream::new(text.as_bytes()),
            state: State::Rule,
            after_selector: false,
            has_at_rule: false,
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
            has_at_rule: false,
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
            State::DeclarationRule => self.consume_declaration(),
        }
    }

    fn consume_rule(&mut self) -> Result<Token<'a>, Error> {
        match self.stream.curr_char_raw() {
            b'@' => {
                self.after_selector = true;
                self.has_at_rule = true;
                self.stream.advance_raw(1);
                let s = self.consume_ident()?;
                return Ok(Token::AtRule(s));
            }
            b'#' => {
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                let s = try!(self.consume_ident());
                return Ok(Token::IdSelector(s));
            }
            b'.' => {
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                let s = try!(self.consume_ident());
                return Ok(Token::ClassSelector(s));
            }
            b'*' => {
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::UniversalSelector);
            }
            b':' => {
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);

                // Whether this selector is a ::selector.
                let is_double_colon = self.stream.is_char_eq(b':')?;
                if is_double_colon {
                    self.stream.advance_raw(1); // consume the second :
                }

                let s = try!(self.consume_ident());

                if self.stream.curr_char() == Ok(b'(') {
                    // Item is a thing()
                    self.stream.advance_raw(1); // (
                    let inner_len = self.stream.length_to(b')')?;
                    let inner = self.stream.read_raw_str(inner_len);
                    self.stream.advance_raw(1); // )
                    return Ok(if is_double_colon {
                        Token::DoublePseudoClass { selector: s, value: Some(inner) }
                    } else {
                        Token::PseudoClass { selector: s, value: Some(inner) }
                    });
                } else {
                    return Ok(if is_double_colon {
                        Token::DoublePseudoClass { selector: s, value: None }
                    } else {
                        Token::PseudoClass { selector: s, value: None }
                    });
                }
            }
            b'[' => {
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                let len = try!(self.stream.length_to(b']'));
                let s = self.stream.read_raw_str(len);
                self.stream.advance_raw(1); // ]
                self.stream.skip_spaces();
                return Ok(Token::AttributeSelector(s));
            }
            b',' => {
                self.after_selector = false;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::Comma);
            }
            b'{' => {
                self.after_selector = false;
                self.has_at_rule = false;
                self.state = State::Declaration;
                self.stream.advance_raw(1);
                return Ok(Token::BlockStart);
            }
            b'>' => {
                if self.after_selector {
                    self.after_selector = false;
                    self.has_at_rule = false;
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
                    self.has_at_rule = false;
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

                    match self.stream.curr_char()? {
                        b'{' | b'/' | b'>' | b'+' | b'*' => { return self.parse_next(); },
                        _ => {
                            self.after_selector = false;
                            if !self.has_at_rule {
                                return Ok(Token::Combinator(Combinator::Space));
                            }
                        }
                    }
                }

                let s = try!(self.consume_ident());
                let token_type = if self.has_at_rule {
                    self.has_at_rule = true;
                    Token::AtStr(s)
                } else {
                    self.has_at_rule = false;
                    Token::TypeSelector(s)
                };

                self.after_selector = true;
                return Ok(token_type);
            }
        }
    }

    fn consume_declaration(&mut self) -> Result<Token<'a>, Error> {
        self.stream.skip_spaces();
        self.has_at_rule = false;

        match self.stream.curr_char_raw() {
            b'}' => {
                if self.state == State::DeclarationRule {
                    self.state = State::Declaration;
                } else if self.state == State::Declaration {
                    self.state = State::Rule;
                }
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::BlockEnd);
            },
            b'{' => {
                if self.state == State::Rule {
                    self.state = State::Declaration;
                } else if self.state == State::Declaration {
                    self.state = State::DeclarationRule;
                }
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::BlockStart);
            },
            b'/' => {
                if try!(self.consume_comment()) {
                    return self.parse_next();
                } else {
                    return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                }
            }
            _ => {
                let name = self.consume_ident()?;

                self.stream.skip_spaces();

                if self.stream.is_char_eq(b'/')? {
                    if !try!(self.consume_comment()) {
                        return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                    }
                }

                if self.stream.is_char_eq(b'{')? {
                    if name.is_empty() {
                        return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                    } else {
                        return Ok(Token::DeclarationStr(name));
                    }
                }

                self.stream.advance_raw(1); // :
                self.stream.skip_spaces();

                if self.stream.is_char_eq(b'/')? {
                    if !try!(self.consume_comment()) {
                        return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                    }
                }

                let len = self.stream.length_to_either(&[b';', b'}'])?;

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

                Ok(Token::Declaration(name, value))
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
