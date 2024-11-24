# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

Be aware that this project is still v0.y.z which means that anything can change anytime:

> "4. Major version zero (0.y.z) is for initial development. Anything MAY change at any time. The
> public API SHOULD NOT be considered stable."
>
> (Semantic Versioning Specification)

## Indicating incompatible changes on major version zero

We defined for this project that while being on major version zero we mark incompatible changes with
new minor version numbers. Please note that this is no version handling covered by `Semver`.

## 0.5.0 - 2024-11-24

- Cleanup of the library
- Removed DFA scanner variant and making NFA implementation the default, hence `use_nfa()` on
`ScannerBuilder` is no more needed and was removed
- Update of documentation
- Performance optimization of scanner phase of compiled NFA

## 0.4.0 - 2024-11-12

- Support for lookahead, negative and positive. Please see README.md for details.
- Support for Scanners based on NFAs. These scanners can handle overlapping character classes.
Call `use_nfa()` on the scanner builder before calling `build()`.

```rust
let scanner = ScannerBuilder::new()
    .add_scanner_modes(&*MODES)
    .use_nfa()
    .build()
    .unwrap();
let find_iter = scanner.find_iter(INPUT).with_positions();
let matches: Vec<MatchExt> = find_iter.collect();
```

## 0.3.3 - 2024-10-11

- Provide an iterator adapter `WithPositions` to convert the iterator over type `Match` to an
iterator over types `MatchExt` which contains line and column information for the start position as
well as the end position of each match.
```rust
let scanner = ScannerBuilder::new().add_scanner_modes(&*MODES).build().unwrap();
let find_iter = scanner.find_iter(INPUT).with_positions();
let matches: Vec<MatchExt> = find_iter.collect();
```
- Fixed handling of current scanner mode. There was a bug that scanner mode switching from the
outside had no effect on cloned `ScannerImpl` instances. This was fixed by removing the mode from
the `Scanner` and leaving it only on the `ScannerImpl`.

- We also allow now to set the scanner mode on a `FindMatches` and even on a `WithPositions` by
implementing the new trait `ScannerModeSwitcher` for both of them.

- Add some documentation like PlantUML overview diagram to the `doc` folder. Also moved
`matching_state.dot` into this folder to have anything in one place. For viewing the PlantUML
diagram in Visual Studio Code I recommend the excellent
[PlantUML extension](https://marketplace.visualstudio.com/items?itemName=jebbs.plantuml).
Let me add that this overview diagram is in no way complete. It should just give a rough overview.

## 0.3.2 - 2024-09-09

- Performance: `Scanner` no more holds `ScannerImpl` in a `Rc<RefCell<>>` to save time during
creation of a new `find_iter`. Instead `ScannerImpl` is now `Clone` by wrapping the match functions
array in an `Arc`. This makes the `Scanner` usable as static global again and has the same effect
regarding performance.
- `Scanner::mode_name` returns a `Option<&str>` again, instead of `Option<String>` which saves an
additional heap allocation.


## 0.3.1 - 2024-09-07

- Add support for lots of unicode named classes like `XID_Start` and `XID_Continue` by the help of
the `seshat-unicode` crate
- Performance: Scanner holds ScannerImpl in a `Rc<RefCell<>>` to save time during creation of a new
`find_iter`
- Add support for generating compiled DFAs as DOT files to scanner implementation

## 0.3.0 - 2024-08-29

### Breaking changes
- Renamed `Scanner::trace_compiled_dfa_as_dot` to `Scanner::log_compiled_dfas_as_dot`
### Non-breaking changes
- Fixed some help comments
- Fixed the `Display` implementation of `DFA`
- Added a new test to module `internal::match_function`
- Added new function `FindMatches::with_offset` to support resetting the input test
- Added new function `FindMatches::offset` to retrieve the total offset of the char indices 
iterator in bytes.

## 0.2.0 - 2024-08-19

### Breaking changes
- `Scanner::find_iter` now returns a `FindMatches` directly instead of `Result<FindMatches>` because
the construction is basically infallible.
### Non-breaking changes
- Add a new API `add_patterns` to `ScannerBuilder` to support simple use cases with only one scanner
state.
- Add derive `Debug` trait to `Scanner`
- Add CHANGELOG

## 0.1.1 - 2024-08-17

- Changed description in Cargo.toml

## 0.1.0 - 2024-08-17

- First release