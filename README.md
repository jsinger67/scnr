<!-- markdownlint-disable first-line-h1 -->

[![Rust](https://github.com/jsinger67/scnr/actions/workflows/rust.yml/badge.svg)](https://github.com/jsinger67/scnr/actions/workflows/rust.yml)
[![Docs.rs](https://docs.rs/scnr/badge.svg)](https://docs.rs/scnr)
[![Crates.io](https://img.shields.io/crates/v/scnr.svg)](https://crates.io/crates/scnr)

<!-- markdownlint-enable first-line-h1 -->

# About `scnr`

This crate provides a scanner/lexer with sufficient regex support and minimal compile time.
The scanners support multiple scanner modes out of the box.
Scanner modes are known from Lex/Flex as
[Start conditions](https://www.cs.princeton.edu/~appel/modern/c/software/flex/flex.html#SEC11).

The crate is still in an early phase and not ready for production yet. Early adopters can quite
safely use it. In case you find a bug, please report it.

## How to use it

```rust
use scnr::ScannerBuilder;

static PATTERNS: &[&str] = &[
    r";",                          // Semicolon
    r"0|[1-9][0-9]*",              // Number
    r"//.*(\r\n|\r|\n)",           // Line comment
    r"/\*([.\r\n--*]|\*[^/])*\*/", // Block comment
    r"[a-zA-Z_]\w*",               // Identifier
    r"=",                          // Assignment
];

const INPUT: &str = r#"
// This is a comment
a = 10;
b = 20;
/* This is a block comment
   that spans multiple lines */
c = a;
"#;

fn main() {
    let scanner = ScannerBuilder::new()
        .add_patterns(PATTERNS)
        .build()
        .expect("ScannerBuilder error");
    let find_iter = scanner.find_iter(INPUT);
    for ma in find_iter {
        println!("Match: {:?}: '{}'", ma, &INPUT[ma.span().range()]);
    }
}
```

## Guard rails

* The scanners should be built quickly.
* Scanners are based on NFAs internally.
* The scanners only support `&str`, i.e. patterns are of type `&str` and the input is of type
`&str`. `scnr` focuses on programming languages rather than byte sequences.

## Not supported regex features

We don't support **anchored matches**, i.e. `^`, `$`, `\b`, `\B`, `\A`, `\z` and so on, are not
available.
Mostly, this can be tolerated because of the overall properties of the scanner, and especially the
fact that the longest match will win mitigates the need for such anchors.

To elaborate this a bit more:

Lets say you have a pattern for the keyword `if` and a pattern for an identifier
`/[a-zA-Z_][a-zA-Z0-9_]*/`. Both could match the `if` but the keyword will win if you have its
pattern inserted **before** the pattern of the identifier. If the scanner encounters an input like,
e.g. `ifi` the identifier will match because of the longest match rule. With these guaranties it is
simply unnecessary to declare the keyword 'if' with attached word boundaries (`\b`).

Also we currently don't support **flags** (`i`, `m`, `s`, `R`, `U`, `u`, `x`), like in
```r"(?i)a+(?-i)b+"```.
I need to evaluate if this is a problem, but at the moment I belief that this is tolerable.

There is no need for **capture groups** in the context of token matching, so I see no necessity to
implement this feature.

Also we don't support **non-greedy** repetitions in the spirit of flex. See more about greediness
below.

## Not supported Flex features

As follows from the above regex restrictions anchors `^` and `$` are currently not supported.

## Lookahead

As of version 0.4.0 `scnr` supports *trailing contexts*, like in Flex, e.g. ```ab/cd```.

Additionally to Flex `scnr` supports not only positive lookahead but also negative lookahead.

The configuration of these look ahead constraints can be done via the `Pattern` struct which now
contains an optional member `lookahead`. The inner type of the Option is `Lookahead` that contains
a patter string and a flag that determines whether the lookahead pattern should match (positive
lookahead) or not match (negative lookahead).

To configure a scanner with patterns that contain lookahead expressions you have to use 
`add_scanner_mode` or `add_scanner_modes` of the `ScannerBuilder`.

With the help of a positive lookahead you can define a semantic like
```
match pattern R only if it is followed by pattern S
```
On the other hand with a negative lookahead you can define a semantic like
```
match pattern R only if it is NOT followed by pattern S
```

The lookahead patterns denoted above as `S` are not considered as part of the matched string.

## Greediness of repetitions

The generated scanners work with *compact NFAs* in which all repetition patterns like `*` and `+`
match **greedily**.

The `scnr` scanner generator does not directly support non-greedy quantifiers like *? or +? found in
some other regex engines. However, you can achieve non-greedy behavior by carefully structuring your
regular expressions and using scanner modes to control the matching process.

For example, you can use scanner modes to switch between different states in your scanner, allowing
you to control when and how patterns are matched. This approach can help you simulate non-greedy
behavior by ensuring that the scanner only matches the minimal necessary input before switching to a
different mode.


### Scanner modes

As an example for a possible realization of a non-greedy behavior we take the simple case of a block
comment known from languages like C++ a.s.o. The regex would normally look like this.
```regex
/\*.*?\*/
```

You would simple make the repetition non-greedy by adding a question mark to the repetition operator.
In `scnr` you would instead create a second scanner mode, here named `COMMENT`.

This mode is entered on the **comment start** `/\\*`, then handles all tokens inside a comment and
enters INITIAL mode on the **comment end** `\\*/` again.

The scanner modes can be defined for instance in json:

```json
[
  {
    "name": "INITIAL",
    "patterns": [
      { "pattern": "/\\*", "token_type": 1}
    ],
    "transitions": [[1, 1]]
  },
  {
    "name": "COMMENT",
    "patterns": [
      { "pattern": "\\*/", "token_type": 2},
      { "pattern": "[.\\r\\n]", "token_type": 3}
    ],
    "transitions": [[2, 0]]
  }
]
```

> Note, that this kind of JSON data can be deserialized to `Vec<ScannerMode>` thanks to `serde` and
`serde_json`.

Above you see two modes. The scanner always starts in mode 0, usually INITIAL. When encountering a
token type 1, **comment start**, it switches to mode 1, `COMMENT`. Here the **comment end** token
type 2 has higher precedence than the `[.\\r\\n]` token 3, simply by having a lower index in the
patterns slice. On token 2 it switches to mode INITIAL again. All other tokens are covered by token
type 3, **comment content**.

In this scenario the parser knows that token type 3 is **comment content** and can handle it
accordingly.
