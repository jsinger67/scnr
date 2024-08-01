# About `scnr`

## WIP

This crate is a work in progress and not ready for use yet.

I strive for a scanner/lexer with sufficient regex support and minimal compile time.
The scanner should support multiple scanner modes out of the box.
Scanner modes are known from Lex/Flex as
[Start conditions](https://www.cs.princeton.edu/~appel/modern/c/software/flex/flex.html#SEC11).

## Guard rails

* The scanners should be built quickly.
* They will probably never support `u8`, i.e. patterns are types convertible to `&[&str]` and the
input is of type convertible to `&str`.

## Not supported regex features

It doesn't supports **anchored matches**, i.e. ^, $, \b, \B, \A, \z and so on, are not available.
Mostly, this can be tolerated because of the overall properties of the scanner. Also the fact that
the longest match will win mitigates the need for such anchors.

Also it currently doesn't support **flags** (i, m, s, R, U, u, x), like in ```r"(?i)a+(?-i)b+"```.
I need to evaluate if this is a problem, but a the moment I belief that this is tolerable.

There is no need for **capture groups** in the context of token matching, so I see no necessity to
implement this feature.
