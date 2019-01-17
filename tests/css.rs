// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate simplecss;

use simplecss::{Tokenizer, Token, Combinator, Error, ErrorPos};

macro_rules! test {
    ($name:ident, $text:expr, $( $token:expr ),*) => {
        #[test]
        fn $name() {
            let mut t = Tokenizer::new($text);
            $(
                assert_eq!(t.parse_next().unwrap(), $token);
            )*
            assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
        }
    };
}

macro_rules! test_selectors {
    ($name:ident, $text:expr, $( $token:expr ),*) => {
        #[test]
        fn $name() {
            let mut t = Tokenizer::new($text);
            $(
                assert_eq!(t.parse_next().unwrap(), $token);
            )*
            assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
            assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "red"));
            assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
            assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
        }
    };
}

macro_rules! test_err {
    ($name:ident, $text:expr, $err:expr) => {
        #[test]
        fn $name() {
            let mut t = Tokenizer::new($text);
            assert_eq!(t.parse_next().unwrap_err(), $err);
        }
    };
}

test_selectors!(selectors_1,
    "* { color: red }",
    Token::UniversalSelector
);

test_selectors!(selectors_2,
    "p { color: red }",
    Token::TypeSelector("p")
);

test_selectors!(selectors_3,
    ":first-child { color: red }",
    Token::PseudoClass { selector: "first-child", value: None }
);

test_selectors!(selectors_4,
    ":lang(fr) { color: red }",
    Token::PseudoClass { selector: "lang", value: Some("fr") }
);

test_selectors!(selectors_6,
    ".cls { color: red }",
    Token::ClassSelector("cls")
);

test_selectors!(selectors_7,
    "#p2 { color: red }",
    Token::IdSelector("p2")
);

test_selectors!(selectors_8,
    "#p2{color:red}",
    Token::IdSelector("p2")
);

test_selectors!(selectors_9,
    " div { color:red }",
    Token::TypeSelector("div")
);

test_selectors!(complex_selectors_1,
    "h1 p { color: red; }",
    Token::TypeSelector("h1"),
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("p")
);

test_selectors!(complex_selectors_2,
    "h1 p g k { color: red; }",
    Token::TypeSelector("h1"),
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("p"),
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("g"),
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("k")
);

test_selectors!(complex_selectors_3,
    "[rel=\"author\"], [rel=\"alternate\"] { color: red; }",
    Token::AttributeSelector("rel=\"author\""),
    Token::Comma,
    Token::AttributeSelector("rel=\"alternate\"")
);

test_selectors!(complex_selectors_4,
    "div:after, div:before { color: red; }",
    Token::TypeSelector("div"),
    Token::PseudoClass { selector: "after", value: None },
    Token::Comma,
    Token::TypeSelector("div"),
    Token::PseudoClass { selector: "before", value: None }
);

test_selectors!(complex_selectors_5,
    "p.valid { color: red; }",
    Token::TypeSelector("p"),
    Token::ClassSelector("valid")
);

test_selectors!(complex_selectors_6,
    ".test:first-letter { color: red; }",
    Token::ClassSelector("test"),
    Token::PseudoClass { selector: "first-letter", value: None }
);

test_selectors!(complex_selectors_7,
    ".test, .control { color: red; }",
    Token::ClassSelector("test"),
    Token::Comma,
    Token::ClassSelector("control")
);

test_selectors!(complex_selectors_8,
    "div>h1 { color: red; }",
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::GreaterThan),
    Token::TypeSelector("h1")
);

test_selectors!(complex_selectors_9,
    "div > h1 { color: red; }",
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::GreaterThan),
    Token::TypeSelector("h1")
);

test_selectors!(complex_selectors_10,
    "div+h1 { color: red; }",
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("h1")
);

test_selectors!(complex_selectors_11,
    "div+h1 { color: red; }",
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("h1")
);

test_selectors!(complex_selectors_12,
    "p.test:first-letter { color: red; }",
    Token::TypeSelector("p"),
    Token::ClassSelector("test"),
    Token::PseudoClass { selector: "first-letter", value: None }
);

test_selectors!(complex_selectors_13,
    "#div1
+
p { color: red; }",
    Token::IdSelector("div1"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("p")
);

test_selectors!(complex_selectors_14,
    "button[type=\"submit\"] { color: red; }",
    Token::TypeSelector("button"),
    Token::AttributeSelector("type=\"submit\"")
);

test_selectors!(complex_selectors_15,
    "div em[id] { color: red; }",
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("em"),
    Token::AttributeSelector("id")
);

test_selectors!(complex_selectors_16,
    "div * em { color: red; }",
    Token::TypeSelector("div"),
    Token::UniversalSelector,
    Token::TypeSelector("em")
);

test_selectors!(complex_selectors_17,
    "div#div1 { color: red; }",
    Token::TypeSelector("div"),
    Token::IdSelector("div1")
);

test_selectors!(complex_selectors_18,
    "div#x:first-letter { color: red; }",
    Token::TypeSelector("div"),
    Token::IdSelector("x"),
    Token::PseudoClass { selector: "first-letter", value: None }
);

test_selectors!(complex_selectors_19,
    "[class=foo] + div + div + div + div { color: red; }",
    Token::AttributeSelector("class=foo"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("div")
);

test_selectors!(complex_selectors_20,
    "input[type=\"radio\"]:focus + label { color: red; }",
    Token::TypeSelector("input"),
    Token::AttributeSelector("type=\"radio\""),
    Token::PseudoClass { selector: "focus", value: None },
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("label")
);

test_selectors!(complex_selectors_21,
    ":visited:active { color: red; }",
    Token::PseudoClass { selector: "visited", value: None },
    Token::PseudoClass { selector: "active", value: None }
);

// it's actually invalid, but we do not validate it
test_selectors!(complex_selectors_22,
    "p:first-line p, #p1 { color: red; }",
    Token::TypeSelector("p"),
    Token::PseudoClass { selector: "first-line", value: None },
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("p"),
    Token::Comma,
    Token::IdSelector("p1")
);

test_selectors!(complex_selectors_23,
    "p * { color: red; }",
    Token::TypeSelector("p"),
    Token::UniversalSelector
);

test_selectors!(complex_selectors_24,
    "*:active { color: red; }",
    Token::UniversalSelector,
    Token::PseudoClass { selector: "active", value: None }
);

test_selectors!(complex_selectors_25,
    "html > body > *:first-line  { color: red; }",
    Token::TypeSelector("html"),
    Token::Combinator(Combinator::GreaterThan),
    Token::TypeSelector("body"),
    Token::Combinator(Combinator::GreaterThan),
    Token::UniversalSelector,
    Token::PseudoClass { selector: "first-line", value: None }
);

test_selectors!(attribute_selector_1,
    "[attr=\"test\"] { color: red }",
    Token::AttributeSelector("attr=\"test\"")
);

test_selectors!(attribute_selector_2,
    "[attr=\"test\"][attr2=\"test2\"] { color: red }",
    Token::AttributeSelector("attr=\"test\""),
    Token::AttributeSelector("attr2=\"test2\"")
);

test!(blocks_1,
"p { color: red; }
p { color: red; }",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd,
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(blocks_2,
"p{color:red;}p{color:red;}",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd,
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(blocks_3,
"p {
    color:red;
}",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(blocks_4,
"p
{
    color:red;
}",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(blocks_5,
    "p{}",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::BlockEnd
);

#[test]
fn declarations_1() {
    let vec = vec![
        "p {color:red}",
        "p {color:red;}",
        "p {color:red }",
        "p { color: red; }",
        "p { color : red ; }",
        "p {  color  :  red  ;  } ",
        "p { color : red ; }"
    ];

    for css in vec {
        let mut t = Tokenizer::new(css);
        assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("p"));
        assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
        assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "red"));
        assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
        assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
    }
}

test!(declarations_2,
    "p { color:red;;;;color:red; }",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(declarations_3,
    "* {list-style-image: url(\"img.png\");}",
    Token::UniversalSelector,
    Token::BlockStart,
    Token::Declaration("list-style-image", "url(\"img.png\")"),
    Token::BlockEnd
);

test!(declarations_4,
    "* { color: white ! important; }",
    Token::UniversalSelector,
    Token::BlockStart,
    Token::Declaration("color", "white ! important"),
    Token::BlockEnd
);

test!(declarations_5,
    "* { border: 1em solid blue; background: navy url(support/diamond.png) -2em -2em no-repeat }",
    Token::UniversalSelector,
    Token::BlockStart,
    Token::Declaration("border", "1em solid blue"),
    Token::Declaration("background", "navy url(support/diamond.png) -2em -2em no-repeat"),
    Token::BlockEnd
);

test!(declarations_6,
    "* {stroke-width:2}",
    Token::UniversalSelector,
    Token::BlockStart,
    Token::Declaration("stroke-width", "2"),
    Token::BlockEnd
);

test!(comment_1,
    "/* .test { color: green ! important; } */
    * { color: red; }",
    Token::UniversalSelector,
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(comment_2,
    "p /* comment */ { color:red }",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(comment_3,
    "p /* comment */ div { color:red }",
    Token::TypeSelector("p"),
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("div"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(comment_4,
    "div { /**/color: red; }",
    Token::TypeSelector("div"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(comment_5,
    "div { /**/color: red; }",
    Token::TypeSelector("div"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(comment_6,
    "div { /* *\\/*/color: red; }",
    Token::TypeSelector("div"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

// TODO: comment can be between 'red' and ';'
test!(comment_7,
"/*Comment*/div/*Comment*/
{
  /*Comment*/color/*Comment*/: /*Comment*/red;
  /*Comment*/
}/*Comment*/",
    Token::TypeSelector("div"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(comment_8,
" /*
   * Comment
   */
  div
 {
    color : red
 }",
    Token::TypeSelector("div"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test_err!(invalid_2,
    "# div1",
    Error::UnknownToken(ErrorPos::new(1, 2))
);

// test_err!(invalid_3,
//     "#1div ",
//     Error::UnknownToken(ErrorPos::new(1, 2))
// );

test_err!(invalid_4,
    "@import",
    Error::UnsupportedToken(ErrorPos::new(1, 1))
);

#[test]
fn invalid_5() {
    let mut t = Tokenizer::new("div { {color: red;} }");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 7)));
}

#[test]
fn invalid_6() {
    let mut t = Tokenizer::new("div { (color: red;) }");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 7)));
}

#[test]
fn invalid_7() {
    let mut t = Tokenizer::new("div { [color: red;] }");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 7)));
}

#[test]
fn invalid_8() {
    let mut t = Tokenizer::new("div { color: }");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    // assert_eq!(t.parse_next().unwrap(), Token::Property("color"));
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 14)));
}

#[test]
fn invalid_9() {
    let mut t = Tokenizer::new("div");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    // TODO: should be UnexpectedEndOfStream
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

#[test]
fn invalid_10() {
    let mut t = Tokenizer::new("div { /\\*;color: green;*/ }");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 8)));
}

#[test]
fn invalid_11() {
    let mut t = Tokenizer::new("div { /*\\*/*/color: red; }");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 12)));
}

test_err!(invalid_12,
    ".平和 { color: red; }",
    Error::UnknownToken(ErrorPos::new(1, 2))
);

// #[test]
// fn invalid_13() {
//     let mut t = Tokenizer::new("div { causta: \"}\" + ({7} * '\'') }");
//     assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
//     assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
//     assert_eq!(t.parse_next().unwrap(), Token::Declaration("causta", "\""));
//     assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 15)));
// }

#[test]
fn invalid_14() {
    let mut t = Tokenizer::new(
"div
{
    \"this is a string]}\"\"[{\\\"'\";  /*should be parsed as a string but be ignored*/
    {{}}[]'';                     /*should be parsed as nested blocks and a string but be ignored*/
    color: green;
}");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(3, 5)));
}

test_err!(invalid_15,
    ".\\xC3\\xA9 { color: red; }",
    Error::UnknownToken(ErrorPos::new(1, 2))
);

test_err!(invalid_16,
    "::invalidPseudoElement",
    Error::UnknownToken(ErrorPos::new(1, 2))
);

#[test]
fn invalid_17() {
    let mut t = Tokenizer::new(" ");
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

#[test]
fn invalid_18() {
    let mut t = Tokenizer::new("div > >");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::Combinator(Combinator::GreaterThan));
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 7)));
}
