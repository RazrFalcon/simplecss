# simplecss

A very simple streaming parser/tokenizer for [CSS 2.1](https://www.w3.org/TR/CSS21/)
data format without heap allocations.

Since it's very simple we will start with limitations:

## Limitations
- [At-rules](https://www.w3.org/TR/CSS21/syndata.html#at-rules) are not supported.

  `@import`, `@media`, etc. will lead to a parsing error.
- The ident token must be ASCII only.

  CSS like `#аттр { имя:значение }` will lead to a parsing error.
- Property values are not parsed.

  In CSS like `* { width: 5px }` you will get `width` property with `5px` values as a string.
- Attribute selector rule is not parsed.

  `[foo~="warning"]` will be parsed as `Token::AttributeSelector("foo~=\"warning\"")`.
- There are no data validation.

  - Pseudo-class tokens can contain any text, language pseudo-class can contain any text or even none.
  - Declarations can contain any kind of names and values.
- All comments will be ignored.

  They didn't have it's own `Token` item.
- CDO/CDC comments are not supported.
- Parser is case sensitive. All keywords should be lowercase.
- Unicode escape, like `\26`, is not supported.
- No spec-defined error handling.

  If something will go wrong you will get an error. Parser will not recover an invalid input.
  [Details](https://www.w3.org/TR/CSS21/syndata.html#rule-sets).

## Where to use
`simplecss` can be useful for parsing a very simple or predefined CSS.

It's tiny, dependency free and pretty fast.

## Examples

Simple

```text
* { color : red }
| | |           |
| | |           +- Token::BlockEnd
| | +- Token::Declaration("color", "red")
| +- Token::BlockStart
+- Token::UniversalSelector
```

Complex

```text
div#x:first-letter em[id] + .hh1 { color : red }
|  | |            || |    | |    | |           |
|  | |            || |    | |    | |           +- Token::BlockEnd
|  | |            || |    | |    | +- Token::Declaration("color", "red")
|  | |            || |    | |    +- Token::BlockStart
|  | |            || |    | +- Token::ClassSelector("hh1")
|  | |            || |    +- Token::Combinator(Combinator::Plus)
|  | |            || +- Token::AttributeSelector("id")
|  | |            |+- Token::TypeSelector("em")
|  | |            +- Token::Combinator(Combinator::Space)
|  | +- Token::PseudoClass("first-letter")
|  +- Token::IdSelector("x")
+- Token::TypeSelector("div")
```
