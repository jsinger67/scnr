# About `scnr`

## WIP

This crate is a work in progress and not ready for use yet.

I strive for a scanner/lexer with sufficient regex support and minimal compile time.
The scanner should support multiple scanner modes out of the box.
Scanner modes are known from Lex/Flex as
[Start conditions](https://www.cs.princeton.edu/~appel/modern/c/software/flex/flex.html#SEC11).

## Guard rails

* The scanners should be built quickly.
* The scanners will probably never support `u8`, i.e. patterns are of type convertible to `&str` and
the input is of type convertible to `&str`. We concentrate on programming languages rather than byte
sequences.

## Not supported regex features

We don't support **anchored matches**, i.e. ^, $, \b, \B, \A, \z and so on, are not available.
Mostly, this can be tolerated because of the overall properties of the scanner, and especially the
fact that the longest match will win mitigates the need for such anchors.

To elaborate this a bit more:

Lets say you have a pattern for the keyword 'if' and a pattern for an identifier
/[a-zA-Z_][a-zA-Z0-9_]*/. Both could match the 'if' but the keyword will win iff you have its
pattern inserted before the pattern of the identifier. If the scanner encounters an input like,
e.g. 'ifi' the identifier will match because of the longest match rule. With these guaranties it is
simply unnecessary to declare the keyword 'if' with attached word boundaries (\b).

Also we currently don't support **flags** (i, m, s, R, U, u, x), like in ```r"(?i)a+(?-i)b+"```.
I need to evaluate if this is a problem, but at the moment I belief that this is tolerable.

There is no need for **capture groups** in the context of token matching, so I see no necessity to
implement this feature.
