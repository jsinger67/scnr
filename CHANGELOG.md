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

## 0.3.1 - Not released yet

- Add support of unicode named classes `XID_Start` and `XID_Continue` by the help of the
`unicode-xid` crate

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