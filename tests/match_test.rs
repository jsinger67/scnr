/// This file contains a hopefully increasing number of match tests to verify the correctness of the
/// scanner.
///
/// Some tests are based on the https://github.com/kkos/oniguruma/blob/master/test/test_utf8.c file
/// from the Oniguruma project.
/// Copyright (c) 2002-2019 K.Kosako kkosako0@gmail.com All rights reserved.
use scnr::ScannerBuilder;

/// Test data for the match tests.
/// The test data consists of a pattern, an input string, the expected matches, and an optional error
/// message.
#[derive(Debug)]
struct TestData {
    pattern: &'static str,
    input: &'static str,
    expected: &'static [(&'static str, usize, usize)],
    error_msg: Option<&'static str>,
}

// A macros to easily create a TestData struct.

// Valid pattern, input, and expected matches.
macro_rules! td {
    ($pattern:expr, $input:expr, $expected:expr) => {
        TestData {
            pattern: $pattern,
            input: $input,
            expected: $expected,
            error_msg: None,
        }
    };
}

// Invalid pattern and expected error message.
macro_rules! tu {
    ($pattern:expr, $input:expr, $expected:expr, $result:expr) => {
        TestData {
            pattern: $pattern,
            input: "",
            expected: &[],
            error_msg: Some($result),
        }
    };
}

// Pattern that causes a regex parse error
macro_rules! tr {
    ($pattern:expr, $input:expr, $expected:expr) => {
        TestData {
            pattern: $pattern,
            input: "",
            expected: &[],
            error_msg: Some("regex parse error"),
        }
    };
}

const TEST_DATA: &[TestData] = &[
    // ---------------------------------------------------------------------------------------------
    // The following tests are extracted from the test_utf8.c file from the Oniguruma project.
    // ---------------------------------------------------------------------------------------------
    td!(r#""#, "", &[]),
    tu!(r#"^"#, "", &[], "StartLine"),
    tu!(r#"^a"#, "\na", &[("n", 1, 2)], "StartLine"),
    tu!(r#"$"#, "", &[], "EndLine"),
    tr!(r#"$\O"#, "bb\n", &[("", 2, 3)]),
    tr!(r#"\G"#, "", &[]),
    tu!(r#"\A"#, "", &[], "StartText"),
    tr!(r#"\Z"#, "", &[]),
    tu!(r#"\z"#, "", &[], "EndText"),
    tu!(r#"^$"#, "", &[], "StartLine"),
    // td!(r#"\ca"#, "\001", &[("\", 0, 1)]),
    // td!(r#"\C-b"#, "\002", &[("\", 0, 1)]),
    // td!(r#"\c\\"#, "\034", &[("\", 0, 1)]),
    // td!(r#"q[\c\\]"#, "q\034", &[("q\", 0, 2)]),
    td!(r#""#, "a", &[]),
    td!(r#"a"#, "a", &[("a", 0, 1)]),
    td!(r#"\x61"#, "a", &[("a", 0, 1)]),
    td!(r#"aa"#, "aa", &[("aa", 0, 2)]),
    td!(r#"aaa"#, "aaa", &[("aaa", 0, 3)]),
    td!(
        r#"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"#,
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        &[("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", 0, 35)]
    ),
    td!(r#"ab"#, "ab", &[("ab", 0, 2)]),
    td!(r#"b"#, "ab", &[("b", 1, 2)]),
    td!(r#"bc"#, "abc", &[("bc", 1, 3)]),
    tu!(
        r#"(?i:#RET#)"#,
        "#INS##RET#",
        &[("#RET#", 5, 10)],
        "CaseInsensitive"
    ),
    // td!(r#"\17"#, "\017", &[("\", 0, 1)]),
    td!(r#"\x1f"#, "\x1f", &[("\x1f", 0, 1)]),
    tr!(r#"a(?#....\\JJJJ)b"#, "ab", &[("ab", 0, 2)]),
    tu!(
        r#"(?x)  G (o O(?-x)oO) g L"#,
        "GoOoOgLe",
        &[("GoOoOgL", 0, 7)],
        "IgnoreWhitespace"
    ),
    td!(r#"."#, "a", &[("a", 0, 1)]),
    td!(r#".."#, "ab", &[("ab", 0, 2)]),
    td!(r#"\w"#, "e", &[("e", 0, 1)]),
    td!(r#"\s"#, " ", &[(" ", 0, 1)]),
    td!(r#"\S"#, "b", &[("b", 0, 1)]),
    td!(r#"\d"#, "4", &[("4", 0, 1)]),
    tu!(r#"\b"#, "z ", &[], "WordBoundary"),
    tu!(r#"\b"#, " z", &[("", 1, 1)], "WordBoundary"),
    tu!(r#"\b"#, "  z ", &[("", 2, 2)], "WordBoundary"),
    tu!(r#"\B"#, "zz ", &[("", 1, 1)], "NotWordBoundary"),
    tu!(r#"\B"#, "z ", &[("", 2, 2)], "NotWordBoundary"),
    tu!(r#"\B"#, " z", &[], "NotWordBoundary"),
    td!(r#"[ab]"#, "b", &[("b", 0, 1)]),
    td!(r#"[a-z]"#, "t", &[("t", 0, 1)]),
    td!(r#"[^a]"#, "\n", &[("\n", 0, 1)]),
    td!(r#"[]]"#, "]", &[("]", 0, 1)]),
    td!(r#"[\^]+"#, "0^^1", &[("^^", 1, 3)]),
    td!(r#"[b-]"#, "b", &[("b", 0, 1)]),
    td!(r#"[b-]"#, "-", &[("-", 0, 1)]),
    td!(r#"[\w]"#, "z", &[("z", 0, 1)]),
    td!(r#"[\W]"#, "b$", &[("$", 1, 2)]),
    td!(r#"[\d]"#, "5", &[("5", 0, 1)]),
    td!(r#"[\D]"#, "t", &[("t", 0, 1)]),
    td!(r#"[\s]"#, " ", &[(" ", 0, 1)]),
    td!(r#"[\S]"#, "b", &[("b", 0, 1)]),
    td!(r#"[\w\d]"#, "2", &[("2", 0, 1)]),
    td!(r#"[[:upper:]]"#, "B", &[("B", 0, 1)]),
    td!(r#"[*[:xdigit:]+]"#, "+", &[("+", 0, 1)]),
    td!(
        r#"[*[:xdigit:]+]"#,
        "GHIKK-9+*",
        &[("9", 6, 7), ("+", 7, 8), ("*", 8, 9)]
    ),
    td!(r#"[*[:xdigit:]+]"#, "-@^+", &[("+", 3, 4)]),
    td!(r#"[[:upper]]"#, ":", &[(":", 0, 1)]),
    td!(r#"[[:^upper:]]"#, "a", &[("a", 0, 1)]),
    td!(r#"[[:^lower:]]"#, "A", &[("A", 0, 1)]),
    td!(r#"[[:upper\] :]]"#, "]", &[("]", 0, 1)]),
    td!(r#"[[::]]"#, ":", &[(":", 0, 1)]),
    td!(r#"[[:::]]"#, ":", &[(":", 0, 1)]),
    td!(r#"[[:\]:]]*"#, ":]", &[(":]", 0, 2)]),
    td!(r#"[[:\[:]]*"#, ":[", &[(":[", 0, 2)]),
    td!(r#"[[:\]]]*"#, ":]", &[(":]", 0, 2)]),
    td!(r#"[\x24-\x27]"#, "\x26", &[("\x26", 0, 1)]),
    td!(r#"[\x5a-\x5c]"#, "\x5b", &[("\x5b", 0, 1)]),
    td!(r#"[\x6A-\x6D]"#, "\x6c", &[("\x6c", 0, 1)]),
    td!(r#"[\[]"#, "[", &[("[", 0, 1)]),
    td!(r#"[\]]"#, "]", &[("]", 0, 1)]),
    td!(r#"[&]"#, "&", &[("&", 0, 1)]),
    td!(r#"[[ab]]"#, "b", &[("b", 0, 1)]),
    td!(r#"[[ab]c]"#, "c", &[("c", 0, 1)]),
    td!(r#"[[ab]&&bc]"#, "b", &[("b", 0, 1)]),
    td!(r#"[a-z&&b-y&&c-x]"#, "w", &[("w", 0, 1)]),
    td!(r#"[[^a&&a]&&a-z]"#, "b", &[("b", 0, 1)]),
    td!(r#"[[^a-z&&bcdef]&&[^c-g]]"#, "h", &[("h", 0, 1)]),
    td!(r#"[^[^abc]&&[^cde]]"#, "c", &[("c", 0, 1)]),
    td!(r#"[^[^abc]&&[^cde]]"#, "e", &[("e", 0, 1)]),
    tr!(r#"[a-&&-a]"#, "-", &[("-", 0, 1)]),
    td!(r#"a\Wbc"#, "a bc", &[("a bc", 0, 4)]),
    td!(r#"a.b.c"#, "aabbc", &[("aabbc", 0, 5)]),
    td!(r#".\wb\W..c"#, "abb bcc", &[("abb bcc", 0, 7)]),
    td!(r#"\s\wzzz"#, " zzzz", &[(" zzzz", 0, 5)]),
    td!(r#"aa.b"#, "aabb", &[("aabb", 0, 4)]),
    td!(r#".a"#, "aa", &[("aa", 0, 2)]),
    tu!(r#"^a"#, "a", &[("a", 0, 1)], "StartLine"),
    tu!(r#"^a$"#, "a", &[("a", 0, 1)], "StartLine"),
    tu!(r#"^\w$"#, "a", &[("a", 0, 1)], "StartLine"),
    tu!(r#"^\wab$"#, "zab", &[("zab", 0, 3)], "StartLine"),
    tu!(
        r#"^\wabcdef$"#,
        "zabcdef",
        &[("zabcdef", 0, 7)],
        "StartLine"
    ),
    tu!(
        r#"^\w...def$"#,
        "zabcdef",
        &[("zabcdef", 0, 7)],
        "StartLine"
    ),
    td!(r#"\w\w\s\Waaa\d"#, "aa  aaa4", &[("aa  aaa4", 0, 8)]),
    tr!(r#"\A\Z"#, "", &[]),
    tu!(r#"\Axyz"#, "xyz", &[("xyz", 0, 3)], "StartText"),
    tr!(r#"xyz\Z"#, "xyz", &[("xyz", 0, 3)]),
    tu!(r#"xyz\z"#, "xyz", &[("xyz", 0, 3)], "EndText"),
    tr!(r#"a\Z"#, "a", &[("a", 0, 1)]),
    tr!(r#"\Gaz"#, "az", &[("az", 0, 2)]),
    td!(r#"\^\$"#, "^$", &[("^$", 0, 2)]),
    tu!(r#"^x?y"#, "xy", &[("xy", 0, 2)], "StartLine"),
    tu!(r#"^(x?y)"#, "xy", &[("xy", 0, 2)], "StartLine"),
    td!(r#"\w"#, "_", &[("_", 0, 1)]),
    tr!(r#"(?=z)z"#, "z", &[("z", 0, 1)]),
    tr!(r#"(?!z)a"#, "a", &[("a", 0, 1)]),
    tu!(r#"(?i:a)"#, "a", &[("a", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:a)"#, "A", &[("A", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:A)"#, "a", &[("a", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:i)"#, "I", &[("I", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:I)"#, "i", &[("i", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:[A-Z])"#, "i", &[("i", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:[a-z])"#, "I", &[("I", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:ss)"#, "ss", &[("ss", 0, 2)], "CaseInsensitive"),
    tu!(r#"(?i:ss)"#, "Ss", &[("Ss", 0, 2)], "CaseInsensitive"),
    tu!(r#"(?i:ss)"#, "SS", &[("SS", 0, 2)], "CaseInsensitive"),
    // tu!(r#"(?i:ss)"#, "\xc5\xbfS", &[("\xc", 0, 3)], "CaseInsensitive"),
    // tu!(r#"(?i:ss)"#, "s\xc5\xbf", &[("s\x", 0, 3)], "CaseInsensitive"),
    // tu!(r#"(?i:ss)"#, "\xc3\x9f", &[("\x", 0, 2)], "CaseInsensitive"),
    // tu!(r#"(?i:ss)"#, "\xe1\xba\x9e", &[("\xe", 0, 3)], "CaseInsensitive"),
    tu!(r#"(?i:xssy)"#, "xssy", &[("xssy", 0, 4)], "CaseInsensitive"),
    tu!(r#"(?i:xssy)"#, "xSsy", &[("xSsy", 0, 4)], "CaseInsensitive"),
    tu!(r#"(?i:xssy)"#, "xSSy", &[("xSSy", 0, 4)], "CaseInsensitive"),
    // tu!(r#"(?i:xssy)"#, "x\xc5\xbfSy", &[("x\xc5", 0, 5)], "CaseInsensitive"),
    // tu!(r#"(?i:xssy)"#, "xs\xc5\xbfy", &[("xs\xc", 0, 5)], "CaseInsensitive"),
    // tu!(r#"(?i:xssy)"#, "x\xc3\x9fy", &[("x\xc", 0, 4)], "CaseInsensitive"),
    // tu!(r#"(?i:xssy)"#, "x\xe1\xba\x9ey", &[("x\xe1", 0, 5)], "CaseInsensitive"),
    tu!(
        r#"(?i:x\xc3\x9fy)"#,
        "xssy",
        &[("xssy", 0, 4)],
        "CaseInsensitive"
    ),
    tu!(
        r#"(?i:x\xc3\x9fy)"#,
        "xSSy",
        &[("xSSy", 0, 4)],
        "CaseInsensitive"
    ),
    tu!(r#"(?i:\xc3\x9f)"#, "ss", &[("ss", 0, 2)], "CaseInsensitive"),
    tu!(r#"(?i:\xc3\x9f)"#, "SS", &[("SS", 0, 2)], "CaseInsensitive"),
    tu!(
        r#"(?i:[\xc3\x9f])"#,
        "ss",
        &[("ss", 0, 2)],
        "CaseInsensitive"
    ),
    tu!(
        r#"(?i:[\xc3\x9f])"#,
        "SS",
        &[("SS", 0, 2)],
        "CaseInsensitive"
    ),
    tr!(r#"(?i)(?<!ss)z"#, "qqz", &[("z", 2, 3)]),
    tu!(r#"(?i:[A-Z])"#, "a", &[("a", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:[f-m])"#, "H", &[("H", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:[f-m])"#, "h", &[("h", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:[A-c])"#, "D", &[("D", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:[!-k])"#, "Z", &[("Z", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:[!-k])"#, "7", &[("7", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:[T-}])"#, "b", &[("b", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:[T-}])"#, "{", &[("{", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i:\?a)"#, "?A", &[("?A", 0, 2)], "CaseInsensitive"),
    tu!(r#"(?i:\*A)"#, "*a", &[("*a", 0, 2)], "CaseInsensitive"),
    tu!(r#"(?m:.)"#, "\n", &[("\n", 0, 1)], "MultiLine"),
    tu!(r#"(?m:a.)"#, "a\n", &[("a\n", 0, 2)], "MultiLine"),
    tu!(r#"(?m:.b)"#, "a\nb", &[("\n", 1, 3)], "MultiLine"),
    td!(r#".*abc"#, "dddabdd\nddabc", &[("ddabc", 8, 13)]),
    td!(
        r#".+abc"#,
        "dddabdd\nddabcaa\naaaabc",
        &[("ddabc", 8, 13), ("aaaabc", 16, 22)]
    ),
    tu!(
        r#"(?m:.*abc)"#,
        "dddabddabc",
        &[("dddabddabc", 0, 10)],
        "MultiLine"
    ),
    td!(r#"a?"#, "", &[]),
    td!(r#"a?"#, "b", &[]),
    td!(r#"a?"#, "a", &[("a", 0, 1)]),
    td!(r#"a*"#, "", &[]),
    td!(r#"a*"#, "a", &[("a", 0, 1)]),
    td!(r#"a*"#, "aaa", &[("aaa", 0, 3)]),
    td!(r#"a*"#, "baaaa", &[("aaaa", 1, 5)]),
    td!(r#"a+"#, "a", &[("a", 0, 1)]),
    td!(r#"a+"#, "aaaa", &[("aaaa", 0, 4)]),
    td!(r#"a+"#, "aabbb", &[("aa", 0, 2)]),
    td!(r#"a+"#, "baaaa", &[("aaaa", 1, 5)]),
    td!(r#".?"#, "", &[]),
    td!(r#".?"#, "f", &[("f", 0, 1)]),
    td!(r#".?"#, "\n", &[]),
    td!(r#".*"#, "", &[]),
    td!(r#".*"#, "abcde", &[("abcde", 0, 5)]),
    td!(r#".+"#, "z", &[("z", 0, 1)]),
    td!(r#".+"#, "zdswer\n", &[("zdswer", 0, 6)]),
    tr!(r#"(.*)a\1f"#, "babfbac", &[("babf", 0, 4)]),
    tr!(r#"(.*)a\1f"#, "bacbabf", &[("babf", 3, 7)]),
    tr!(r#"((.*)a\2f)"#, "bacbabf", &[("babf", 3, 7)]),
    tr!(
        r#"(.*)a\1f"#,
        "baczzzzzz\nbazz\nzzzzbabf",
        &[("zzba", 19, 23)]
    ),
    td!(r#"(?:x?)?"#, "", &[]),
    td!(r#"(?:x?)?"#, "x", &[("x", 0, 1)]),
    td!(r#"(?:x?)?"#, "xx", &[("x", 0, 1), ("x", 1, 2)]),
    td!(r#"(?:x?)*"#, "", &[]),
    td!(r#"(?:x?)*"#, "x", &[("x", 0, 1)]),
    td!(r#"(?:x?)*"#, "xx", &[("xx", 0, 2)]),
    td!(r#"(?:x?)+"#, "", &[]),
    td!(r#"(?:x?)+"#, "x", &[("x", 0, 1)]),
    td!(r#"(?:x?)+"#, "xx", &[("xx", 0, 2)]),
    td!(r#"(?:x?)\?\?"#, "", &[]),
    td!(r#"(?:x?)\?\?"#, "x", &[]),
    td!(r#"(?:x?)\?\?"#, "xx", &[]),
    tu!(r#"(?:x?)*?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x?)*?"#, "x", &[], "Non-greedy"),
    tu!(r#"(?:x?)*?"#, "xx", &[], "Non-greedy"),
    tu!(r#"(?:x?)+?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x?)+?"#, "x", &[("x", 0, 1)], "Non-greedy"),
    tu!(r#"(?:x?)+?"#, "xx", &[("x", 0, 1)], "Non-greedy"),
    td!(r#"(?:x*)?"#, "", &[]),
    td!(r#"(?:x*)?"#, "x", &[("x", 0, 1)]),
    td!(r#"(?:x*)?"#, "xx", &[("xx", 0, 2)]),
    td!(r#"(?:x*)*"#, "", &[]),
    td!(r#"(?:x*)*"#, "x", &[("x", 0, 1)]),
    td!(r#"(?:x*)*"#, "xx", &[("xx", 0, 2)]),
    td!(r#"(?:x*)+"#, "", &[]),
    td!(r#"(?:x*)+"#, "x", &[("x", 0, 1)]),
    td!(r#"(?:x*)+"#, "xx", &[("xx", 0, 2)]),
    td!(r#"(?:x*)\?\?"#, "", &[]),
    td!(r#"(?:x*)\?\?"#, "x", &[]),
    td!(r#"(?:x*)\?\?"#, "xx", &[]),
    tu!(r#"(?:x*)*?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x*)*?"#, "x", &[], "Non-greedy"),
    tu!(r#"(?:x*)*?"#, "xx", &[], "Non-greedy"),
    tu!(r#"(?:x*)+?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x*)+?"#, "x", &[("x", 0, 1)], "Non-greedy"),
    tu!(r#"(?:x*)+?"#, "xx", &[("xx", 0, 2)], "Non-greedy"),
    td!(r#"(?:x+)?"#, "", &[]),
    td!(r#"(?:x+)?"#, "x", &[("x", 0, 1)]),
    td!(r#"(?:x+)?"#, "xx", &[("xx", 0, 2)]),
    td!(r#"(?:x+)*"#, "", &[]),
    td!(r#"(?:x+)*"#, "x", &[("x", 0, 1)]),
    td!(r#"(?:x+)*"#, "xx", &[("xx", 0, 2)]),
    td!(r#"(?:x+)+"#, "x", &[("x", 0, 1)]),
    td!(r#"(?:x+)+"#, "xx", &[("xx", 0, 2)]),
    td!(r#"(?:x+)\?\?"#, "", &[]),
    td!(r#"(?:x+)\?\?"#, "x", &[]),
    td!(r#"(?:x+)\?\?"#, "xx", &[]),
    tu!(r#"(?:x+)*?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x+)*?"#, "x", &[], "Non-greedy"),
    tu!(r#"(?:x+)*?"#, "xx", &[], "Non-greedy"),
    tu!(r#"(?:x+)+?"#, "x", &[("x", 0, 1)], "Non-greedy"),
    tu!(r#"(?:x+)+?"#, "xx", &[("xx", 0, 2)], "Non-greedy"),
    td!(r#"(?:x\?\?)?"#, "", &[]),
    td!(r#"(?:x\?\?)?"#, "x", &[]),
    td!(r#"(?:x\?\?)?"#, "xx", &[]),
    td!(r#"(?:x\?\?)*"#, "", &[]),
    td!(r#"(?:x\?\?)*"#, "x", &[]),
    td!(r#"(?:x\?\?)*"#, "xx", &[]),
    td!(r#"(?:x\?\?)+"#, "", &[]),
    td!(r#"(?:x\?\?)+"#, "x", &[]),
    td!(r#"(?:x\?\?)+"#, "xx", &[]),
    td!(r#"(?:x\?\?)\?\?"#, "", &[]),
    td!(r#"(?:x\?\?)\?\?"#, "x", &[]),
    td!(r#"(?:x\?\?)\?\?"#, "xx", &[]),
    tu!(r#"(?:x\?\?)*?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x\?\?)*?"#, "x", &[], "Non-greedy"),
    tu!(r#"(?:x\?\?)*?"#, "xx", &[], "Non-greedy"),
    tu!(r#"(?:x\?\?)+?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x\?\?)+?"#, "x", &[], "Non-greedy"),
    tu!(r#"(?:x\?\?)+?"#, "xx", &[], "Non-greedy"),
    tu!(r#"(?:x*?)?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x*?)?"#, "x", &[], "Non-greedy"),
    tu!(r#"(?:x*?)?"#, "xx", &[], "Non-greedy"),
    tu!(r#"(?:x*?)*"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x*?)*"#, "x", &[], "Non-greedy"),
    tu!(r#"(?:x*?)*"#, "xx", &[], "Non-greedy"),
    tu!(r#"(?:x*?)+"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x*?)+"#, "x", &[], "Non-greedy"),
    tu!(r#"(?:x*?)+"#, "xx", &[], "Non-greedy"),
    tu!(r#"(?:x*?)\?\?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x*?)\?\?"#, "x", &[], "Non-greedy"),
    tu!(r#"(?:x*?)\?\?"#, "xx", &[], "Non-greedy"),
    tu!(r#"(?:x*?)*?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x*?)*?"#, "x", &[], "Non-greedy"),
    tu!(r#"(?:x*?)*?"#, "xx", &[], "Non-greedy"),
    tu!(r#"(?:x*?)+?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x*?)+?"#, "x", &[], "Non-greedy"),
    tu!(r#"(?:x*?)+?"#, "xx", &[], "Non-greedy"),
    tu!(r#"(?:x+?)?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x+?)?"#, "x", &[("x", 0, 1)], "Non-greedy"),
    tu!(r#"(?:x+?)?"#, "xx", &[("x", 0, 1)], "Non-greedy"),
    tu!(r#"(?:x+?)*"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x+?)*"#, "x", &[("x", 0, 1)], "Non-greedy"),
    tu!(r#"(?:x+?)*"#, "xx", &[("xx", 0, 2)], "Non-greedy"),
    tu!(r#"(?:x+?)+"#, "x", &[("x", 0, 1)], "Non-greedy"),
    tu!(r#"(?:x+?)+"#, "xx", &[("xx", 0, 2)], "Non-greedy"),
    tu!(r#"(?:x+?)\?\?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x+?)\?\?"#, "x", &[], "Non-greedy"),
    tu!(r#"(?:x+?)\?\?"#, "xx", &[], "Non-greedy"),
    tu!(r#"(?:x+?)*?"#, "", &[], "Non-greedy"),
    tu!(r#"(?:x+?)*?"#, "x", &[], "Non-greedy"),
    tu!(r#"(?:x+?)*?"#, "xx", &[], "Non-greedy"),
    tu!(r#"(?:x+?)+?"#, "x", &[("x", 0, 1)], "Non-greedy"),
    tu!(r#"(?:x+?)+?"#, "xx", &[("x", 0, 1)], "Non-greedy"),
    td!(r#"a|b"#, "a", &[("a", 0, 1)]),
    td!(r#"a|b"#, "b", &[("b", 0, 1)]),
    td!(r#"|a"#, "a", &[("a", 0, 1)]),
    td!(r#"(|a)"#, "a", &[("a", 0, 1)]),
    td!(r#"ab|bc"#, "ab", &[("ab", 0, 2)]),
    td!(r#"ab|bc"#, "bc", &[("bc", 0, 2)]),
    td!(r#"z(?:ab|bc)"#, "zbc", &[("zbc", 0, 3)]),
    td!(r#"a(?:ab|bc)c"#, "aabc", &[("aabc", 0, 4)]),
    td!(r#"ab|(?:ac|az)"#, "az", &[("az", 0, 2)]),
    td!(r#"a|b|c"#, "dc", &[("c", 1, 2)]),
    td!(
        r#"a|b|cd|efg|h|ijk|lmn|o|pq|rstuvwx|yz"#,
        "pqr",
        &[("pq", 0, 2)]
    ),
    tu!(r#"a|^z"#, "ba", &[("a", 1, 2)], "StartLine"),
    tu!(r#"a|^z"#, "za", &[("z", 0, 1)], "StartLine"),
    tr!(r#"a|\Gz"#, "bza", &[("a", 2, 3)]),
    tr!(r#"a|\Gz"#, "za", &[("z", 0, 1)]),
    tu!(r#"a|\Az"#, "bza", &[("a", 2, 3)], "StartText"),
    tu!(r#"a|\Az"#, "za", &[("z", 0, 1)], "StartText"),
    tr!(r#"a|b\Z"#, "ba", &[("a", 1, 2)]),
    tr!(r#"a|b\Z"#, "b", &[("b", 0, 1)]),
    tu!(r#"a|b\z"#, "ba", &[("a", 1, 2)], "EndText"),
    tu!(r#"a|b\z"#, "b", &[("b", 0, 1)], "EndText"),
    td!(r#"\w|\s"#, " ", &[(" ", 0, 1)]),
    td!(r#"\w|%"#, "%", &[("%", 0, 1)]),
    td!(r#"\w|[&$]"#, "&", &[("&", 0, 1)]),
    td!(r#"[b-d]|[^e-z]"#, "a", &[("a", 0, 1)]),
    td!(r#"(?:a|[c-f])|bz"#, "dz", &[("d", 0, 1)]),
    td!(r#"(?:a|[c-f])|bz"#, "bz", &[("bz", 0, 2)]),
    tr!(r#"abc|(?=zz)..f"#, "zzf", &[("zzf", 0, 3)]),
    tr!(r#"abc|(?!zz)..f"#, "abf", &[("abf", 0, 3)]),
    tr!(r#"(?=za)..a|(?=zz)..a"#, "zza", &[("zza", 0, 3)]),
    tr!(r#"(?>abd|a)c"#, "abdc", &[("abdc", 0, 4)]),
    td!(r#"a?|b"#, "a", &[("a", 0, 1)]),
    td!(r#"a?|b"#, "b", &[("b", 0, 1)]),
    td!(r#"a?|b"#, "", &[]),
    td!(r#"a*|b"#, "aa", &[("aa", 0, 2)]),
    td!(r#"a*|b*"#, "ba", &[("b", 0, 1), ("a", 1, 2)]),
    td!(r#"a*|b*"#, "ab", &[("a", 0, 1), ("b", 1, 2)]),
    td!(r#"a+|b*"#, "", &[]),
    td!(r#"a+|b*"#, "bbb", &[("bbb", 0, 3)]),
    td!(r#"a+|b*"#, "abbb", &[("a", 0, 1), ("bbb", 1, 4)]),
    td!(r#"(a|b)?"#, "b", &[("b", 0, 1)]),
    td!(r#"(a|b)*"#, "ba", &[("ba", 0, 2)]),
    td!(r#"(a|b)+"#, "bab", &[("bab", 0, 3)]),
    td!(r#"(ab|ca)+"#, "caabbc", &[("caab", 0, 4)]),
    td!(r#"(ab|ca)+"#, "aabca", &[("abca", 1, 5)]),
    td!(r#"(ab|ca)+"#, "abzca", &[("ab", 0, 2), ("ca", 3, 5)]),
    td!(r#"(a|bab)+"#, "ababa", &[("ababa", 0, 5)]),
    td!(r#"(a|bab)+"#, "ba", &[("a", 1, 2)]),
    td!(r#"(a|bab)+"#, "baaaba", &[("aaa", 1, 4), ("a", 5, 6)]),
    td!(r#"(?:a|b)(?:a|b)"#, "ab", &[("ab", 0, 2)]),
    td!(r#"(?:a*|b*)(?:a*|b*)"#, "aaabbb", &[("aaabbb", 0, 6)]),
    td!(r#"(?:a*|b*)(?:a+|b+)"#, "aaabbb", &[("aaabbb", 0, 6)]),
    td!(r#"(?:a+|b+){2}"#, "aaabbb", &[("aaabbb", 0, 6)]),
    td!(r#"h{0,}"#, "hhhh", &[("hhhh", 0, 4)]),
    td!(r#"(?:a+|b+){1,2}"#, "aaabbb", &[("aaabbb", 0, 6)]),
    tu!(r#"^a{2,}?a$"#, "aaa", &[("aaa", 0, 3)], "StartLine"),
    tu!(r#"^[a-z]{2,}?$"#, "aaa", &[("aaa", 0, 3)], "StartLine"),
    tu!(r#"(?:a+|\Ab*)cc"#, "cc", &[("cc", 0, 2)], "StartText"),
    tu!(r#"(?:^a+|b+)*c"#, "aabbbabc", &[("bc", 6, 8)], "StartLine"),
    tu!(
        r#"(?:^a+|b+)*c"#,
        "aabbbbc",
        &[("aabbbbc", 0, 7)],
        "StartLine"
    ),
    tu!(r#"a|(?i)c"#, "C", &[("C", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i)c|a"#, "C", &[("C", 0, 1)], "CaseInsensitive"),
    tu!(r#"(?i)c|a"#, "A", &[("A", 0, 1)], "CaseInsensitive"),
    tu!(r#"a(?i)b|c"#, "aB", &[("aB", 0, 2)], "CaseInsensitive"),
    tu!(r#"a(?i)b|c"#, "aC", &[("aC", 0, 2)], "CaseInsensitive"),
    tu!(r#"(?i:c)|a"#, "C", &[("C", 0, 1)], "CaseInsensitive"),
    td!(r#"[abc]?"#, "abc", &[("a", 0, 1), ("b", 1, 2), ("c", 2, 3)]),
    td!(r#"[abc]*"#, "abc", &[("abc", 0, 3)]),
    td!(r#"[^abc]*"#, "abc", &[]),
    td!(r#"a?\?"#, "aaa", &[]),
    td!(r#"ba?\?b"#, "bab", &[]), // Oniguruma: ("bab", 0, 3)
    tu!(r#"a*?"#, "aaa", &[], "Non-greedy"),
    tu!(r#"ba*?"#, "baa", &[("b", 0, 1)], "Non-greedy"),
    tu!(r#"ba*?b"#, "baab", &[("baab", 0, 4)], "Non-greedy"),
    tu!(r#"a+?"#, "aaa", &[("a", 0, 1)], "Non-greedy"),
    tu!(r#"ba+?"#, "baa", &[("ba", 0, 2)], "Non-greedy"),
    tu!(r#"ba+?b"#, "baab", &[("baab", 0, 4)], "Non-greedy"),
    td!(r#"(?:a?)?\?"#, "a", &[]),
    td!(r#"(?:a?\?)?"#, "a", &[]),
    tu!(r#"(?:a?)+?"#, "aaa", &[("a", 0, 1)], "Non-greedy"),
    td!(r#"(?:a+)?\?"#, "aaa", &[]),
    td!(r#"(?:a+)?\?b"#, "aaab", &[]), // Oniguruma: ("aaab", 0, 4)
    td!(r#"(?:ab)?{2}"#, "", &[]),
    td!(r#"(?:ab)?{2}"#, "ababa", &[("abab", 0, 4)]),
    td!(r#"(?:ab)*{0}"#, "ababa", &[]),
    td!(r#"(?:ab){3,}"#, "abababab", &[("abababab", 0, 8)]),
    td!(r#"(?:ab){2,4}"#, "ababab", &[("ababab", 0, 6)]),
    td!(r#"(?:ab){2,4}"#, "ababababab", &[("abababab", 0, 8)]),
    tu!(
        r#"(?:ab){2,4}?"#,
        "ababababab",
        &[("abab", 0, 4)],
        "Non-greedy"
    ),
    tr!(r#"(?:ab){,}"#, "ab{,}", &[("ab{,}", 0, 5)]),
    tu!(
        r#"(?:abc)+?{2}"#,
        "abcabcabc",
        &[("abcabc", 0, 6)],
        "Non-greedy"
    ),
    tu!(
        r#"(?:X*)(?i:xa)"#,
        "XXXa",
        &[("XXXa", 0, 4)],
        "CaseInsensitive"
    ),
    td!(r#"(d+)([^abc]z)"#, "dddz", &[("dddz", 0, 4)]),
    td!(r#"([^abc]*)([^abc]z)"#, "dddz", &[("dddz", 0, 4)]),
    td!(r#"(\w+)(\wz)"#, "dddz", &[("dddz", 0, 4)]),
    td!(r#"((ab))"#, "ab", &[("ab", 0, 2)]),
    tu!(r#"(^a)"#, "a", &[("a", 0, 1)], "StartLine"),
    tr!(r#"(abc)(?i:\1)"#, "abcABC", &[("abcABC", 0, 6)]),
    td!(r#"(?:abc)|(ABC)"#, "abc", &[("abc", 0, 3)]),
    tr!(r#"(?:(?:\1|z)(a))+$"#, "zaaa", &[("zaaa", 0, 4)]),
    tr!(r#"(a)(?=\1)"#, "aa", &[("a", 0, 1)]),
    tr!(r#"(a)\1"#, "aa", &[("aa", 0, 2)]),
    tr!(r#"(a?)\1"#, "aa", &[("aa", 0, 2)]),
    tr!(r#"(a?\?)\1"#, "aa", &[]),
    tr!(r#"(a*)\1"#, "aaaaa", &[("aaaa", 0, 4)]),
    tr!(r#"a(b*)\1"#, "abbbb", &[("abbbb", 0, 5)]),
    tr!(r#"a(b*)\1"#, "ab", &[("a", 0, 1)]),
    tr!(r#"(a*)(b*)\1\2"#, "aaabbaaabb", &[("aaabbaaabb", 0, 10)]),
    tr!(r#"(a*)(b*)\2"#, "aaabbbb", &[("aaabbbb", 0, 7)]),
    tr!(r#"(((((((a*)b))))))c\7"#, "aaabcaaa", &[("aaabcaaa", 0, 8)]),
    tr!(r#"(a)(b)(c)\2\1\3"#, "abcbac", &[("abcbac", 0, 6)]),
    tr!(r#"([a-d])\1"#, "cc", &[("cc", 0, 2)]),
    tr!(r#"(\w\d\s)\1"#, "f5 f5 ", &[("f5 f5 ", 0, 6)]),
    tr!(r#"(who|[a-c]{3})\1"#, "whowho", &[("whowho", 0, 6)]),
    tr!(
        r#"...(who|[a-c]{3})\1"#,
        "abcwhowho",
        &[("abcwhowho", 0, 9)]
    ),
    tr!(r#"(who|[a-c]{3})\1"#, "cbccbc", &[("cbccbc", 0, 6)]),
    tr!(r#"(^a)\1"#, "aa", &[("aa", 0, 2)]),
    tr!(r#"(a*\Z)\1"#, "a", &[("", 1, 1)]),
    tr!(r#".(a*\Z)\1"#, "ba", &[("a", 1, 2)]),
    tr!(r#"((?i:az))\1"#, "AzAz", &[("AzAz", 0, 4)]),
    tr!(r#"(?<=a)b"#, "ab", &[("b", 1, 2)]),
    tr!(r#"(?<=a|b)b"#, "bb", &[("b", 1, 2)]),
    tr!(r#"(?<=a|bc)b"#, "bcb", &[("b", 2, 3)]),
    tr!(r#"(?<=a|bc)b"#, "ab", &[("b", 1, 2)]),
    tr!(r#"(?<=a|bc||defghij|klmnopq|r)z"#, "rz", &[("z", 1, 2)]),
    tr!(r#"(?<=(?i:abc))d"#, "ABCd", &[("d", 3, 4)]),
    tr!(r#"(?<=^|b)c"#, " cbc", &[("c", 3, 4)]),
    tr!(r#"(?<=a|^|b)c"#, " cbc", &[("c", 3, 4)]),
    tr!(r#"(?<=a|(^)|b)c"#, " cbc", &[("c", 3, 4)]),
    tr!(r#"(?<=a|(^)|b)c"#, "cbc", &[("c", 0, 1)]),
    tr!(r#"(Q)(?<=a|(?(1))|b)c"#, "cQc", &[("Qc", 1, 3)]),
    tr!(r#"(?<=a|(?~END)|b)c"#, "ENDc", &[("c", 3, 4)]),
    tr!(r#"(?<!a|(?:^)|b)c"#, " cbc", &[("c", 1, 2)]),
    tr!(r#"(a)\g<1>"#, "aa", &[("aa", 0, 2)]),
    tr!(r#"(?<!a)b"#, "cb", &[("b", 1, 2)]),
    tr!(r#"(?<!a|bc)b"#, "bbb", &[("b", 0, 1)]),
    td!(r#"(?<name1>a)"#, "a", &[("a", 0, 1)]),
    tr!(r#"(?<name_2>ab)\g<name_2>"#, "abab", &[("abab", 0, 4)]),
    tr!(
        r#"(?<name_3>.zv.)\k<name_3>"#,
        "azvbazvb",
        &[("azvbazvb", 0, 8)]
    ),
    tr!(r#"(?<=\g<ab>)|-\zEND (?<ab>XyZ)"#, "XyZ", &[("", 3, 3)]),
    tr!(r#"(?<n>|a\g<n>)+"#, "", &[]),
    tr!(r#"(?<n>|\(\g<n>\))+$"#, "()(())", &[("()(())", 0, 6)]),
    tr!(r#"\g<n>(abc|df(?<n>.YZ){2,8}){0}"#, "XYZ", &[("XYZ", 0, 3)]),
    tr!(r#"\A(?<n>(a\g<n>)|)\z"#, "aaaa", &[("aaaa", 0, 4)]),
    tr!(
        r#"(?<n>|\g<m>\g<n>)\z|\zEND (?<m>a|(b)\g<m>)"#,
        "bbbbabba",
        &[("bbbbabba", 0, 8)]
    ),
    tr!(
        r#"(?<name1240>\w+\sx)a+\k<name1240>"#,
        "  fg xaaaaaaaafg x",
        &[("fg xaaaaaaaafg x", 2, 18)]
    ),
    tr!(r#"(.)(((?<_>a)))\k<_>"#, "zaa", &[("zaa", 0, 3)]),
    tr!(
        r#"((?<name1>\d)|(?<name2>\w))(\k<name1>|\k<name2>)"#,
        "ff",
        &[("ff", 0, 2)]
    ),
    tr!(r#"(?:(?<x>)|(?<x>efg))\k<x>"#, "", &[]),
    tr!(
        r#"(?:(?<x>abc)|(?<x>efg))\k<x>"#,
        "abcefgefg",
        &[("efgefg", 3, 9)]
    ),
    tr!(r#"(?<x>x)(?<x>xx)\k<x>"#, "xxxx", &[("xxxx", 0, 4)]),
    tr!(r#"(?<x>x)(?<x>xx)\k<x>"#, "xxxxz", &[("xxxx", 0, 4)]),
    tr!(
        r#"(?:(?<n1>.)|(?<n1>..)|(?<n1>...)|(?<n1>....)|(?<n1>.....)|(?<n1>......)|(?<n1>.......)|(?<n1>........)|(?<n1>.........)|(?<n1>..........)|(?<n1>...........)|(?<n1>............)|(?<n1>.............)|(?<n1>..............))\k<n1>$"#,
        "a-pyumpyum",
        &[("pyumpyum", 2, 10)]
    ),
    tr!(r#"(?<foo>a|\(\g<foo>\))"#, "a", &[("a", 0, 1)]),
    tr!(
        r#"(?<foo>a|\(\g<foo>\))"#,
        "((((((a))))))",
        &[("((((((a))))))", 0, 13)]
    ),
    tr!(
        r#"\g<bar>|\zEND(?<bar>.*abc$)"#,
        "abcxxxabc",
        &[("abcxxxabc", 0, 9)]
    ),
    tr!(r#"\g<1>|\zEND(.a.)"#, "bac", &[("bac", 0, 3)]),
    tr!(
        r#"\A(?:\g<pon>|\g<pan>|\zEND  (?<pan>a|c\g<pon>c)(?<pon>b|d\g<pan>d))$"#,
        "cdcbcdc",
        &[("cdcbcdc", 0, 7)]
    ),
    tr!(
        r#"\A(?<n>|a\g<m>)\z|\zEND (?<m>\g<n>)"#,
        "aaaa",
        &[("aaaa", 0, 4)]
    ),
    tr!(r#"(?<n>(a|b\g<n>c){3,5})"#, "baaaaca", &[("aaaa", 1, 5)]),
    tr!(
        r#"(?<n>(a|b\g<n>c){3,5})"#,
        "baaaacaaaaa",
        &[("baaaacaaaa", 0, 10)]
    ),
    tr!(
        r#"(?<pare>\(([^\(\)]++|\g<pare>)*+\))"#,
        "((a))",
        &[("((a))", 0, 5)]
    ),
    tr!(r#"()*\1"#, "", &[]),
    tr!(r#"(?:()|())*\1\2"#, "", &[]),
    td!(r#"(?:a*|b*)*c"#, "abadc", &[("c", 4, 5)]),
    td!(r#"x((.)*)*x"#, "0x1x2x3", &[("x1x2x", 1, 6)]),
    tr!(r#"x((.)*)*x(?i:\1)\Z"#, "0x1x2x1X2", &[("x1x2x1X2", 1, 9)]),
    tr!(r#"(?:()|()|()|()|()|())*\2\5"#, "", &[]),
    tr!(r#"(?:()|()|()|(x)|()|())*\2b\5"#, "b", &[("b", 0, 1)]),
    td!(r#"[0-9-a]"#, "-", &[("-", 0, 1)]),
    tr!(r#"\o{101}"#, "A", &[("A", 0, 1)]),
    tr!(r#"\A(a|b\g<1>c)\k<1+3>\z"#, "bbacca", &[("bbacca", 0, 6)]),
    tr!(
        r#"(?i)\A(a|b\g<1>c)\k<1+2>\z"#,
        "bBACcbac",
        &[("bBACcbac", 0, 8)]
    ),
    tr!(r#"(?i)(?<X>aa)|(?<X>bb)\k<X>"#, "BBbb", &[("BBbb", 0, 4)]),
    tr!(r#"(?:\k'+1'B|(A)C)*"#, "ACAB", &[("ACAB", 0, 4)]),
    tr!(r#"\g<+2>(abc)(ABC){0}"#, "ABCabc", &[("ABCabc", 0, 6)]),
    tr!(r#"A\g'0'|B()"#, "AAAAB", &[("AAAAB", 0, 5)]),
    tr!(r#"(a*)(?(1))aa"#, "aaaaa", &[("aaaaa", 0, 5)]),
    tr!(r#"(a*)(?(-1))aa"#, "aaaaa", &[("aaaaa", 0, 5)]),
    tr!(r#"(?<name>aaa)(?('name'))aa"#, "aaaaa", &[("aaaaa", 0, 5)]),
    tr!(r#"(a)(?(1)aa|bb)a"#, "aaaaa", &[("aaaa", 0, 4)]),
    tr!(r#"(?:aa|())(?(<1>)aa|bb)a"#, "aabba", &[("aabba", 0, 5)]),
    tr!(r#"(?:aa|())(?('1')aa|bb|cc)a"#, "aacca", &[("aacca", 0, 5)]),
    tr!(r#"(a)(?(1)|)c"#, "ac", &[("ac", 0, 2)]),
    tr!(r#"(a)(?(1+0)b|c)d"#, "abd", &[("abd", 0, 3)]),
    tr!(
        r#"(?:(?'name'a)|(?'name'b))(?('name')c|d)e"#,
        "ace",
        &[("ace", 0, 3)]
    ),
    tr!(
        r#"(?:(?'name'a)|(?'name'b))(?('name')c|d)e"#,
        "bce",
        &[("bce", 0, 3)]
    ),
    tr!(r#"\R"#, "\r\n", &[("\r", 0, 2)]),
    tr!(r#"\R"#, "\r", &[("\r", 0, 1)]),
    tr!(r#"\R"#, "\n", &[("\n", 0, 1)]),
    tr!(r#"\R"#, "\x0b", &[("\x0b", 0, 1)]),
    // tr!(r#"\R"#, "\xc2\x85", &[("\xc2", 0, 2)]),
    tr!(r#"\N"#, "a", &[("a", 0, 1)]),
    tr!(r#"\O"#, "a", &[("a", 0, 1)]),
    tr!(r#"\O"#, "\n", &[("\n", 0, 1)]),
    tr!(r#"(?m:\O)"#, "\n", &[("\n", 0, 1)]),
    tr!(r#"(?-m:\O)"#, "\n", &[("\n", 0, 1)]),
    tr!(r#"\K"#, "a", &[]),
    tr!(r#"a\K"#, "a", &[("", 1, 1)]),
    tr!(r#"a\Kb"#, "ab", &[("b", 1, 2)]),
    tr!(r#"(a\Kb|ac\Kd)"#, "acd", &[("d", 2, 3)]),
    tr!(r#"(a\Kb|\Kac\K)*"#, "acababacab", &[("b", 9, 10)]),
    tr!(r#"(?:()|())*\1"#, "abc", &[]),
    tr!(r#"(?:()|())*\2"#, "abc", &[]),
    tr!(r#"(?:()|()|())*\3\1"#, "abc", &[]),
    tr!(r#"(|(?:a(?:\g'1')*))b|"#, "abc", &[("ab", 0, 2)]),
    tr!(r#"^(\"|)(.*)\1$"#, "XX", &[("XX", 0, 2)]),
    tu!(
        r#"(abc|def|ghi|jkl|mno|pqr|stu){0,10}?\z"#,
        "admno",
        &[("mno", 2, 5)],
        "Non-greedy"
    ),
    tu!(
        r#"(abc|(def|ghi|jkl|mno|pqr){0,7}?){5}\z"#,
        "adpqrpqrpqr",
        &[("pqrpqrpqr", 2, 11)],
        "Non-greedy"
    ),
    tr!(r#"(?!abc).*\z"#, "abcde", &[("bcde", 1, 5)]),
    td!(r#"(.{2,})?"#, "abcde", &[("abcde", 0, 5)]),
    td!(
        r#"((a|b|c|d|e|f|g|h|i|j|k|l|m|n)+)?"#,
        "abcde",
        &[("abcde", 0, 5)]
    ),
    td!(
        r#"((a|b|c|d|e|f|g|h|i|j|k|l|m|n){3,})?"#,
        "abcde",
        &[("abcde", 0, 5)]
    ),
    td!(
        r#"((?:a(?:b|c|d|e|f|g|h|i|j|k|l|m|n))+)?"#,
        "abacadae",
        &[("abacadae", 0, 8)]
    ),
    tu!(
        r#"((?:a(?:b|c|d|e|f|g|h|i|j|k|l|m|n))+?)?z"#,
        "abacadaez",
        &[("abacadaez", 0, 9)],
        "Non-greedy"
    ),
    tu!(r#"\A((a|b)\?\?)?z"#, "bz", &[("bz", 0, 2)], "StartText"),
    tr!(r#"((?<x>abc){0}a\g<x>d)+"#, "aabcd", &[("aabcd", 0, 5)]),
    tr!(r#"((?(abc)true|false))+"#, "false", &[("false", 0, 5)]),
    tu!(
        r#"((?i:abc)d)+"#,
        "abcdABCd",
        &[("abcdABCd", 0, 8)],
        "CaseInsensitive"
    ),
    tr!(r#"((?<!abc)def)+"#, "bcdef", &[("def", 2, 5)]),
    tu!(r#"(\ba)+"#, "aaa", &[("a", 0, 1)], "WordBoundary"),
    tr!(r#"()(?<x>ab)(?(<x>)a|b)"#, "aba", &[("aba", 0, 3)]),
    tr!(r#"(?<=a.b)c"#, "azbc", &[("c", 3, 4)]),
    tr!(r#"(?<=(?(a)a|bb))z"#, "aaz", &[("z", 2, 3)]),
    td!(r#"[a]*\W"#, "aa@", &[("aa@", 0, 3)]),
    td!(r#"[a]*[b]"#, "aab", &[("aab", 0, 3)]),
    tr!(r#"(?<=ab(?<=ab))"#, "ab", &[("", 2, 2)]),
    tr!(r#"(?<x>a)(?<x>b)(\k<x>)+"#, "abbaab", &[("abbaab", 0, 6)]),
    tr!(r#"()(\1)(\2)"#, "abc", &[]),
    tr!(r#"((?(a)b|c))(\1)"#, "abab", &[("abab", 0, 4)]),
    tr!(r#"(?<x>$|b\g<x>)"#, "bbb", &[("bbb", 0, 3)]),
    tr!(r#"(?<x>(?(a)a|b)|c\g<x>)"#, "cccb", &[("cccb", 0, 4)]),
    tr!(r#"(a)(?(1)a*|b*)+"#, "aaaa", &[("aaaa", 0, 4)]),
    td!(r#"[[^abc]&&cde]*"#, "de", &[("de", 0, 2)]),
    td!(r#"(?:a?)+"#, "aa", &[("aa", 0, 2)]),
    tu!(r#"(?:a?)*?"#, "a", &[], "Non-greedy"),
    tu!(r#"(?:a*)*?"#, "a", &[], "Non-greedy"),
    tu!(r#"(?:a+?)*"#, "a", &[("a", 0, 1)], "Non-greedy"),
    tr!(r#"\h"#, "5", &[("5", 0, 1)]),
    tr!(r#"\H"#, "z", &[("z", 0, 1)]),
    tr!(r#"[\h]"#, "5", &[("5", 0, 1)]),
    tr!(r#"[\H]"#, "z", &[("z", 0, 1)]),
    tr!(r#"[\o{101}]"#, "A", &[("A", 0, 1)]),
    td!(r#"[\u0041]"#, "A", &[("A", 0, 1)]),
    tr!(r#"(?~)"#, "", &[]),
    tr!(r#"(?~)"#, "A", &[]),
    tr!(r#"(?~ab)"#, "abc", &[]),
    tr!(r#"(?~abc)"#, "abc", &[]),
    tr!(r#"(?~abc|ab)"#, "abc", &[]),
    tr!(r#"(?~ab|abc)"#, "abc", &[]),
    tr!(r#"(?~a.c)"#, "abc", &[]),
    tr!(r#"(?~a.c|ab)"#, "abc", &[]),
    tr!(r#"(?~ab|a.c)"#, "abc", &[]),
    tr!(r#"aaaaa(?~)"#, "aaaaaaaaaa", &[("aaaaa", 0, 5)]),
    tr!(r#"(?~(?:|aaa))"#, "aaa", &[]),
    tr!(r#"(?~aaa|)"#, "aaa", &[]),
    tr!(
        r#"a(?~(?~))."#,
        "abcdefghijklmnopqrstuvwxyz",
        &[("abcdefghijklmnopqrstuvwxyz", 0, 26)]
    ),
    tr!(r#"/\*(?~\*/)\*/"#, "/* */ */", &[("/* */", 0, 5)]),
    tr!(r#"(?~\w+)zzzzz"#, "zzzzz", &[("zzzzz", 0, 5)]),
    tr!(r#"(?~\w*)zzzzz"#, "zzzzz", &[("zzzzz", 0, 5)]),
    tr!(r#"(?~A.C|B)"#, "ABC", &[]),
    tr!(r#"(?~XYZ|ABC)a"#, "ABCa", &[("BCa", 1, 4)]),
    tr!(r#"(?~XYZ|ABC)a"#, "aABCa", &[("a", 0, 1)]),
    tr!(
        r#"<[^>]*>(?~[<>])</[^>]*>"#,
        "<a>vvv</a>   <b>  </b>",
        &[("<a>vvv</a>", 0, 10)]
    ),
    tr!(r#"(?~ab)"#, "ccc\ndab", &[("ccc\n", 0, 5)]),
    tr!(r#"(?m:(?~ab))"#, "ccc\ndab", &[("ccc\n", 0, 5)]),
    tr!(r#"(?-m:(?~ab))"#, "ccc\ndab", &[("ccc\n", 0, 5)]),
    tr!(
        r#"(?~abc)xyz"#,
        "xyz012345678901234567890123456789abc",
        &[("xyz", 0, 3)]
    ),
    tr!(r#"(?~|78|\d*)"#, "123456789", &[("123456", 0, 6)]),
    tr!(
        r#"(?~|def|(?:abc|de|f){0,100})"#,
        "abcdedeabcfdefabc",
        &[("abcdedeabcf", 0, 11)]
    ),
    tr!(r#"(?~|ab|.*)"#, "ccc\nddd", &[("ccc", 0, 3)]),
    tr!(r#"(?~|ab|\O*)"#, "ccc\ndab", &[("ccc\n", 0, 5)]),
    tr!(r#"(?~|ab|\O{2,10})"#, "ccc\ndab", &[("ccc\n", 0, 5)]),
    tr!(r#"(?~|ab|\O{1,10})"#, "ab", &[("b", 1, 2)]),
    tr!(r#"(?~|abc|\O{1,10})"#, "abc", &[("bc", 1, 3)]),
    tr!(r#"(?~|ab|\O{5,10})|abc"#, "abc", &[("abc", 0, 3)]),
    tr!(
        r#"(?~|ab|\O{1,10})"#,
        "cccccccccccab",
        &[("cccccccccc", 0, 10)]
    ),
    tr!(r#"(?~|aaa|)"#, "aaa", &[]),
    tr!(r#"(?~||a*)"#, "aaaaaa", &[]),
    tr!(r#"(?~||a*?)"#, "aaaaaa", &[]),
    tr!(r#"(a)(?~|b|\1)"#, "aaaaaa", &[("aa", 0, 2)]),
    tr!(r#"(a)(?~|bb|(?:a\1)*)"#, "aaaaaa", &[("aaaaa", 0, 5)]),
    tr!(
        r#"(b|c)(?~|abac|(?:a\1)*)"#,
        "abababacabab",
        &[("bab", 1, 4)]
    ),
    tr!(r#"(?~|aaaaa|a*+)"#, "aaaaa", &[]),
    tr!(r#"(?~|aaaaaa|a*+)b"#, "aaaaaab", &[("aaaaab", 1, 7)]),
    tr!(r#"(?~|abcd|(?>))"#, "zzzabcd", &[]),
    tr!(r#"(?~|abc|a*?)"#, "aaaabc", &[]),
    tr!(r#"(?~|abc)a*"#, "aaaaaabc", &[("aaaaa", 0, 5)]),
    tr!(r#"(?~|abc)a*z|aaaaaabc"#, "aaaaaabc", &[("aaaaaabc", 0, 8)]),
    tr!(r#"(?~|aaaaaa)a*"#, "aaaaaa", &[]),
    tr!(r#"(?~|abc)aaaa|aaaabc"#, "aaaabc", &[("aaaabc", 0, 6)]),
    tr!(r#"(?>(?~|abc))aaaa|aaaabc"#, "aaaabc", &[("aaaabc", 0, 6)]),
    tr!(r#"(?~|)a"#, "a", &[("a", 0, 1)]),
    tr!(r#"(?~|a)(?~|)a"#, "a", &[("a", 0, 1)]),
    tr!(
        r#"(?~|a).*(?~|)a"#,
        "bbbbbbbbbbbbbbbbbbbba",
        &[("bbbbbbbbbbbbbbbbbbbba", 0, 21)]
    ),
    tr!(
        r#"(?~|abc).*(xyz|pqr)(?~|)abc"#,
        "aaaaxyzaaapqrabc",
        &[("aaaaxyzaaapqrabc", 0, 16)]
    ),
    tr!(
        r#"(?~|abc).*(xyz|pqr)(?~|)abc"#,
        "aaaaxyzaaaabcpqrabc",
        &[("bcpqrabc", 11, 19)]
    ),
    td!(r#""#, "あ", &[]),
    // x2("あ", "あ", 0, 3);
    // x2("うう", "うう", 0, 6);
    // x2("あいう", "あいう", 0, 9);
    // x2("こここここここここここここここここここここここここここここここここここ", "こここここここここここここここここここここここここここここここここここ", 0, 105);
    // x2("あ", "いあ", 3, 6);
    // x2("いう", "あいう", 3, 9);
    // td!(r#"\xca\xb8"#, "\xca\xb8", &[("\x", 0, 2)]),
    // x2(".", "あ", 0, 3);
    // x2("..", "かき", 0, 6);
    // x2("\\w", "お", 0, 3);
    // x2("[\\W]", "う$", 3, 4);
    // x2("\\S", "そ", 0, 3);
    // x2("\\S", "漢", 0, 3);
    tu!(r#"\b"#, "気 ", &[], "WordBoundary"),
    tu!(r#"\b"#, " ほ", &[("", 1, 1)], "WordBoundary"),
    tu!(r#"\B"#, "せそ ", &[("", 3, 3)], "NotWordBoundary"),
    // x2("\\B", "う ", 4, 4);
    tu!(r#"\B"#, " い", &[], "NotWordBoundary"),
    // x2("[たち]", "ち", 0, 3);
    // x2("[う-お]", "え", 0, 3);
    // x2("[\\w]", "ね", 0, 3);
    // x2("[\\D]", "は", 0, 3);
    // x2("[\\S]", "へ", 0, 3);
    // x2("[\\w\\d]", "よ", 0, 3);
    // x2("[\\w\\d]", "   よ", 3, 6);
    // x2("鬼\\W車", "鬼 車", 0, 7);
    // x2("あ.い.う", "ああいいう", 0, 15);
    // x2(".\\wう\\W..ぞ", "えうう うぞぞ", 0, 19);
    // x2("\\s\\wこここ", " ここここ", 0, 13);
    // x2("ああ.け", "ああけけ", 0, 12);
    // x2(".お", "おお", 0, 6);
    // x2("^あ", "あ", 0, 3);
    // x2("^む$", "む", 0, 3);
    // x2("^\\w$", "に", 0, 3);
    // x2("^\\wかきくけこ$", "zかきくけこ", 0, 16);
    // x2("^\\w...うえお$", "zあいううえお", 0, 19);
    // x2("\\w\\w\\s\\Wおおお\\d", "aお  おおお4", 0, 16);
    // x2("\\Aたちつ", "たちつ", 0, 9);
    // x2("むめも\\Z", "むめも", 0, 9);
    // x2("かきく\\z", "かきく", 0, 9);
    // x2("かきく\\Z", "かきく\n", 0, 9);
    // x2("\\Gぽぴ", "ぽぴ", 0, 6);
    // x2("(?=せ)せ", "せ", 0, 3);
    // x2("(?!う)か", "か", 0, 3);
    // x2("(?i:あ)", "あ", 0, 3);
    // x2("(?i:ぶべ)", "ぶべ", 0, 6);
    // x2("(?m:よ.)", "よ\n", 0, 4);
    // x2("(?m:.め)", "ま\nめ", 3, 7);
    td!(r#"あ?"#, "", &[]),
    td!(r#"変?"#, "化", &[]),
    // x2("変?", "変", 0, 3);
    td!(r#"量*"#, "", &[]),
    // x2("量*", "量", 0, 3);
    // x2("子*", "子子子", 0, 9);
    td!(r#"馬*"#, "鹿馬馬馬馬", &[("馬馬馬馬", 3, 15)]), // Oniguruma []
    // x2("河+", "河", 0, 3);
    // x2("時+", "時時時時", 0, 12);
    // x2("え+", "ええううう", 0, 6);
    // x2("う+", "おうううう", 3, 15);
    // x2(".?", "た", 0, 3);
    // x2(".*", "ぱぴぷぺ", 0, 12);
    // x2(".+", "ろ", 0, 3);
    // x2(".+", "いうえか\n", 0, 12);
    // x2("あ|い", "あ", 0, 3);
    // x2("あ|い", "い", 0, 3);
    // x2("あい|いう", "あい", 0, 6);
    // x2("あい|いう", "いう", 0, 6);
    // x2("を(?:かき|きく)", "をかき", 0, 9);
    // x2("を(?:かき|きく)け", "をきくけ", 0, 12);
    // x2("あい|(?:あう|あを)", "あを", 0, 6);
    // x2("あ|い|う", "えう", 3, 6);
    // x2("あ|い|うえ|おかき|く|けこさ|しすせ|そ|たち|つてとなに|ぬね", "しすせ", 0, 9);
    // x2("あ|^わ", "ぶあ", 3, 6);
    // x2("あ|^を", "をあ", 0, 3);
    // x2("鬼|\\G車", "け車鬼", 6, 9);
    // x2("鬼|\\G車", "車鬼", 0, 3);
    // x2("鬼|\\A車", "b車鬼", 4, 7);
    // x2("鬼|\\A車", "車", 0, 3);
    // x2("鬼|車\\Z", "車鬼", 3, 6);
    // x2("鬼|車\\Z", "車", 0, 3);
    tr!(r#"鬼|車\Z"#, "車\n", &[("車\n", 0, 3)]),
    // x2("鬼|車\\z", "車鬼", 3, 6);
    // x2("鬼|車\\z", "車", 0, 3);
    // x2("\\w|\\s", "お", 0, 3);
    td!(r#"\w|%"#, "%お", &[("%", 0, 1), ("お", 1, 4)]),
    // x2("\\w|[&$]", "う&", 0, 3);
    // x2("[い-け]", "う", 0, 3);
    // x2("[い-け]|[^か-こ]", "あ", 0, 3);
    // x2("[い-け]|[^か-こ]", "か", 0, 3);
    td!(r#"[^あ]"#, "\n", &[("\n", 0, 1)]),
    // x2("(?:あ|[う-き])|いを", "うを", 0, 3);
    // x2("(?:あ|[う-き])|いを", "いを", 0, 6);
    // x2("あいう|(?=けけ)..ほ", "けけほ", 0, 9);
    // x2("あいう|(?!けけ)..ほ", "あいほ", 0, 9);
    // x2("(?=をあ)..あ|(?=をを)..あ", "ををあ", 0, 9);
    // x2("(?<=あ|いう)い", "いうい", 6, 9);
    // x2("(?>あいえ|あ)う", "あいえう", 0, 12);
    // x2("あ?|い", "あ", 0, 3);
    td!(r#"あ?|い"#, "い", &[("い", 0, 3)]), // Oniguruma []
    td!(r#"あ?|い"#, "", &[]),
    // x2("あ*|い", "ああ", 0, 6);
    td!(r#"あ*|い*"#, "いあ", &[(r#"い"#, 0, 3), (r#"あ"#, 3, 6)]), // Oniguruma []
    // x2("あ*|い*", "あい", 0, 3);
    td!(
        r#"[aあ]*|い*"#,
        "aあいいい",
        &[("aあ", 0, 4), ("いいい", 4, 13)]
    ), // Oniguruma [("aあいい", 0, 4)]
    td!(r#"あ+|い*"#, "", &[]),
    // x2("あ+|い*", "いいい", 0, 9);
    td!(r#"あ+|い*"#, "あいいい", &[("あ", 0, 3), ("いいい", 3, 12)]), // Oniguruma [("あいい", 0, 3)]
    td!(
        r#"あ+|い*"#,
        "aあいいい",
        &[(r#"あ"#, 1, 4), (r#"いいい"#, 4, 13)]
    ),
    // x2("(あ|い)?", "い", 0, 3);
    // x2("(あ|い)*", "いあ", 0, 6);
    // x2("(あ|い)+", "いあい", 0, 9);
    // x2("(あい|うあ)+", "うああいうえ", 0, 12);
    // x2("(あい|うえ)+", "うああいうえ", 6, 18);
    // x2("(あい|うあ)+", "ああいうあ", 3, 15);
    // x2("(あい|うあ)+", "あいをうあ", 0, 6);
    // x2("(あい|うあ)+", "$$zzzzあいをうあ", 6, 12);
    // x2("(あ|いあい)+", "あいあいあ", 0, 15);
    // x2("(あ|いあい)+", "いあ", 3, 6);
    // x2("(あ|いあい)+", "いあああいあ", 3, 12);
    // x2("(?:あ|い)(?:あ|い)", "あい", 0, 6);
    // x2("(?:あ*|い*)(?:あ*|い*)", "あああいいい", 0, 9);
    // x2("(?:あ*|い*)(?:あ+|い+)", "あああいいい", 0, 18);
    // x2("(?:あ+|い+){2}", "あああいいい", 0, 18);
    // x2("(?:あ+|い+){1,2}", "あああいいい", 0, 18);
    // x2("(?:あ+|\\Aい*)うう", "うう", 0, 6);
    // x2("(?:^あ+|い+)*う", "ああいいいあいう", 18, 24);
    // x2("(?:^あ+|い+)*う", "ああいいいいう", 0, 21);
    // x2("う{0,}", "うううう", 0, 12);
    // td!(r#"あ|(?i)c"#, "C", &[("C", 0, 1)]),
    // td!(r#"(?i)c|あ"#, "C", &[("C", 0, 1)]),
    // td!(r#"(?i:あ)|a"#, "a", &[("a", 0, 1)]),
    // td!(r#"[あいう]?"#, "あいう", &[("あいう", 0, 3)]),
    // x2("[あいう]*", "あいう", 0, 9);
    // td!(r#"[^あいう]*"#, "あいう", &[]),
    // td!(r#"あ?\?"#, "あああ", &[]),
    // x2("いあ?\?い", "いあい", 0, 9);
    // td!(r#"あ*?"#, "あああ", &[]),
    // td!(r#"いあ*?"#, "いああ", &[("いああ", 0, 3)]),
    // x2("いあ*?い", "いああい", 0, 12);
    // td!(r#"あ+?"#, "あああ", &[("あああ", 0, 3)]),
    // x2("いあ+?", "いああ", 0, 6);
    // x2("いあ+?い", "いああい", 0, 12);
    // td!(r#"(?:天?)?\?"#, "天", &[]),
    // td!(r#"(?:天?\?)?"#, "天", &[]),
    // td!(r#"(?:夢?)+?"#, "夢夢夢", &[("夢夢夢", 0, 3)]),
    // td!(r#"(?:風+)?\?"#, "風風風", &[]),
    // x2("(?:雪+)?\?霜", "雪雪雪霜", 0, 12);
    // td!(r#"(?:あい)?{2}"#, "", &[]),
    // x2("(?:鬼車)?{2}", "鬼車鬼車鬼", 0, 12);
    // td!(r#"(?:鬼車)*{0}"#, "鬼車鬼車鬼", &[]),
    // x2("(?:鬼車){3,}", "鬼車鬼車鬼車鬼車", 0, 24);
    // x2("(?:鬼車){2,4}", "鬼車鬼車鬼車", 0, 18);
    // x2("(?:鬼車){2,4}", "鬼車鬼車鬼車鬼車鬼車", 0, 24);
    // x2("(?:鬼車){2,4}?", "鬼車鬼車鬼車鬼車鬼車", 0, 12);
    // x2("(?:鬼車){,}", "鬼車{,}", 0, 9);
    // x2("(?:かきく)+?{2}", "かきくかきくかきく", 0, 18);
    // x2("((時間))", "時間", 0, 6);
    // x2("(^あ)", "あ", 0, 3);
    // x2("(無)\\1", "無無", 0, 6);
    // x2("(空?)\\1", "空空", 0, 6);
    // td!(r#"(空?\?)\1"#, "空空", &[]),
    // x2("(空*)\\1", "空空空空空", 0, 12);
    // x2("あ(い*)\\1", "あいいいい", 0, 15);
    // x2("あ(い*)\\1", "あい", 0, 3);
    // x2("(あ*)(い*)\\1\\2", "あああいいあああいい", 0, 30);
    // x2("(あ*)(い*)\\2", "あああいいいい", 0, 21);
    // x2("(((((((ぽ*)ぺ))))))ぴ\\7", "ぽぽぽぺぴぽぽぽ", 0, 24);
    // x2("(は)(ひ)(ふ)\\2\\1\\3", "はひふひはふ", 0, 18);
    // x2("([き-け])\\1", "くく", 0, 6);
    // x2("(\\w\\d\\s)\\1", "あ5 あ5 ", 0, 10);
    // x2("(誰？|[あ-う]{3})\\1", "誰？誰？", 0, 12);
    // x2("...(誰？|[あ-う]{3})\\1", "あaあ誰？誰？", 0, 19);
    // x2("(誰？|[あ-う]{3})\\1", "ういうういう", 0, 18);
    // x2("(^こ)\\1", "ここ", 0, 6);
    // x2("(あ*\\Z)\\1", "あ", 3, 3);
    // x2(".(あ*\\Z)\\1", "いあ", 3, 6);
    // x2("((?i:あvず))\\1", "あvずあvず", 0, 14);
    // x2("(?<愚か>変|\\(\\g<愚か>\\))", "((((((変))))))", 0, 15);
    // x2("\\A(?:\\g<阿_1>|\\g<云_2>|\\z終了  (?<阿_1>観|自\\g<云_2>自)(?<云_2>在|菩薩\\g<阿_1>菩薩))$", "菩薩自菩薩自在自菩薩自菩薩", 0, 39);
    // x2("[[ひふ]]", "ふ", 0, 3);
    // x2("[[いおう]か]", "か", 0, 3);
    // x2("[^[^あ]]", "あ", 0, 3);
    // x2("[[かきく]&&きく]", "く", 0, 3);
    // x2("[あ-ん&&い-を&&う-ゑ]", "ゑ", 0, 3);
    // x2("[[^あ&&あ]&&あ-ん]", "い", 0, 3);
    // x2("[[^あ-ん&&いうえお]&&[^う-か]]", "き", 0, 3);
    // x2("[^[^あいう]&&[^うえお]]", "う", 0, 3);
    // x2("[^[^あいう]&&[^うえお]]", "え", 0, 3);
    // td!(r#"[あ-&&-あ]"#, "-", &[("-", 0, 1)]),
    // x2("[^[^a-zあいう]&&[^bcdefgうえお]q-w]", "え", 0, 3);
    // td!(r#"[^[^a-zあいう]&&[^bcdefgうえお]g-w]"#, "f", &[("f", 0, 1)]),
    // td!(r#"[^[^a-zあいう]&&[^bcdefgうえお]g-w]"#, "g", &[("g", 0, 1)]),
    // x2("a<b>バージョンのダウンロード<\\/b>", "a<b>バージョンのダウンロード</b>", 0, 44);
    // x2(".<b>バージョンのダウンロード<\\/b>", "a<b>バージョンのダウンロード</b>", 0, 44);
    // x2("\\n?\\z", "こんにちは", 15, 15);
    // x2("(?m).*", "青赤黄", 0, 9);
    // x2("(?m).*a", "青赤黄a", 0, 10);
    // x2("\\p{Hiragana}", "ぴ", 0, 3);
    // td!(r#"\p{Emoji}"#, "\xE2\xAD\x90", &[("\xE", 0, 3)]),
    // td!(r#"\p{^Emoji}"#, "\xEF\xBC\x93", &[("\xE", 0, 3)]),
    // td!(r#"\p{Extended_Pictographic}"#, "\xE2\x9A\xA1", &[("\xE", 0, 3)]),
    // x2("\\p{Word}", "こ", 0, 3);
    // x2("[\\p{Word}]", "こ", 0, 3);
    // x2("[^\\p{^Word}]", "こ", 0, 3);
    // x2("[^\\p{^Word}&&\\p{ASCII}]", "こ", 0, 3);
    tu!(
        r#"[^\p{^Word}&&\p{ASCII}]"#,
        "a",
        &[("a", 0, 1)],
        "named class"
    ),
    // x2("[^[\\p{^Word}]&&[\\p{ASCII}]]", "こ", 0, 3);
    // x2("[^[\\p{ASCII}]&&[^\\p{Word}]]", "こ", 0, 3);
    // x2("[^[\\p{^Word}]&&[^\\p{ASCII}]]", "こ", 0, 3);
    // x2("[^\\x{104a}]", "こ", 0, 3);
    // x2("[^\\p{^Word}&&[^\\x{104a}]]", "こ", 0, 3);
    // x2("[^[\\p{^Word}]&&[^\\x{104a}]]", "こ", 0, 3);
    // x2("\\p{^Cntrl}", "こ", 0, 3);
    // x2("[\\p{^Cntrl}]", "こ", 0, 3);
    // x2("[^\\p{Cntrl}]", "こ", 0, 3);
    // x2("[^\\p{Cntrl}&&\\p{ASCII}]", "こ", 0, 3);
    tu!(
        r#"[^\p{Cntrl}&&\p{ASCII}]"#,
        "a",
        &[("a", 0, 1)],
        "named class"
    ),
    // x2("[^[\\p{^Cntrl}]&&[\\p{ASCII}]]", "こ", 0, 3);
    // x2("[^[\\p{ASCII}]&&[^\\p{Cntrl}]]", "こ", 0, 3);
    // x2("(?-W:\\p{Word})", "こ", 0, 3);
    tr!(r#"(?W:\p{Word})"#, "k", &[("k", 0, 1)]),
    // x2("(?-W:[[:word:]])", "こ", 0, 3);
    // x2("(?-D:\\p{Digit})", "３", 0, 3);
    // td!(r#"(?-S:\p{Space})"#, "\xc2\x85", &[("\x", 0, 2)]),
    // x2("(?-P:\\p{Word})", "こ", 0, 3);
    // x2("(?-W:\\w)", "こ", 0, 3);
    tr!(r#"(?-W:\w)"#, "k", &[("k", 0, 1)]),
    tr!(r#"(?W:\w)"#, "k", &[("k", 0, 1)]),
    // x2("(?W:\\W)", "こ", 0, 3);
    tr!(r#"(?-W:\b)"#, "こ", &[]),
    tr!(r#"(?-W:\b)"#, "h", &[]),
    tr!(r#"(?W:\b)"#, "h", &[]),
    tr!(r#"(?W:\B)"#, "こ", &[]),
    tr!(r#"(?-P:\b)"#, "こ", &[]),
    tr!(r#"(?-P:\b)"#, "h", &[]),
    tr!(r#"(?P:\b)"#, "h", &[]),
    tr!(r#"(?P:\B)"#, "こ", &[]),
    tu!(
        r#"\p{InBasicLatin}"#,
        "\x41",
        &[("\x41", 0, 1)],
        "named class"
    ),
    // td!(r#".\Y\O"#, "\x0d\x0a", &[("\x", 0, 2)]),
    // td!(r#".\Y."#, "\x67\xCC\x88", &[("\x6", 0, 3)]),
    // td!(r#"\y.\Y.\y"#, "\x67\xCC\x88", &[("\x6", 0, 3)]),
    // td!(r#"\y.\y"#, "\xEA\xB0\x81", &[("\xE", 0, 3)]),
    // td!(r#"^.\Y.\Y.$"#, "\xE1\x84\x80\xE1\x85\xA1\xE1\x86\xA8", &[("\xE1\x84\", 0, 9)]),
    // td!(r#".\Y."#, "\xE0\xAE\xA8\xE0\xAE\xBF", &[("\xE0\x", 0, 6)]),
    // td!(r#".\Y."#, "\xE0\xB8\x81\xE0\xB8\xB3", &[("\xE0\x", 0, 6)]),
    // td!(r#".\Y."#, "\xE0\xA4\xB7\xE0\xA4\xBF", &[("\xE0\x", 0, 6)]),
    // td!(r#"..\Y."#, "\xE3\x80\xB0\xE2\x80\x8D\xE2\xAD\x95", &[("\xE3\x80\", 0, 9)]),
    // td!(r#"...\Y."#, "\xE3\x80\xB0\xCC\x82\xE2\x80\x8D\xE2\xAD\x95", &[("\xE3\x80\xB", 0, 11)]),
    // td!(r#"^\X$"#, "\x0d\x0a", &[("\x", 0, 2)]),
    // td!(r#"^\X$"#, "\x67\xCC\x88", &[("\x6", 0, 3)]),
    // td!(r#"^\X$"#, "\xE1\x84\x80\xE1\x85\xA1\xE1\x86\xA8", &[("\xE1\x84\", 0, 9)]),
    // td!(r#"^\X$"#, "\xE0\xAE\xA8\xE0\xAE\xBF", &[("\xE0\x", 0, 6)]),
    // td!(r#"^\X$"#, "\xE0\xB8\x81\xE0\xB8\xB3", &[("\xE0\x", 0, 6)]),
    // td!(r#"^\X$"#, "\xE0\xA4\xB7\xE0\xA4\xBF", &[("\xE0\x", 0, 6)]),
    // td!(r#"h\Xllo"#, "ha\xCC\x80llo", &[("ha\xCC\", 0, 7)]),
    // td!(r#"(?y{g})\yabc\y"#, "abc", &[("abc", 0, 3)]),
    // td!(r#"(?y{g})\y\X\y"#, "abc", &[("a", 0, 1)]),
    // td!(r#"(?y{w})\yabc\y"#, "abc", &[("abc", 0, 3)]),
    // td!(r#"(?y{w})\X"#, "\r\n", &[("\r", 0, 2)]),
    // td!(r#"(?y{w})\X"#, "\x0cz", &[("\", 0, 1)]),
    // td!(r#"(?y{w})\X"#, "q\x0c", &[("q", 0, 1)]),
    // td!(r#"(?y{w})\X"#, "\xE2\x80\x8D\xE2\x9D\x87", &[("\xE2\x", 0, 6)]),
    // td!(r#"(?y{w})\X"#, "\x20\x20", &[("\x", 0, 2)]),
    // td!(r#"(?y{w})\X"#, "a\xE2\x80\x8D", &[("a\xE", 0, 4)]),
    // td!(r#"(?y{w})\y\X\y"#, "abc", &[("abc", 0, 3)]),
    // td!(r#"(?y{w})\y\X\y"#, "v\xCE\x87w", &[("v\xC", 0, 4)]),
    // td!(r#"(?y{w})\y\X\y"#, "\xD7\x93\x27", &[("\xD", 0, 3)]),
    // td!(r#"(?y{w})\y\X\y"#, "\xD7\x93\x22\xD7\x93", &[("\xD7\", 0, 5)]),
    // td!(r#"(?y{w})\X"#, "14 45", &[("14", 0, 2)]),
    // td!(r#"(?y{w})\X"#, "a14", &[("a14", 0, 3)]),
    // td!(r#"(?y{w})\X"#, "832e", &[("832e", 0, 4)]),
    // td!(r#"(?y{w})\X"#, "8\xEF\xBC\x8C\xDB\xB0", &[("8\xEF\", 0, 6)]),
    // x2("(?y{w})\\y\\X\\y", "ケン", 0, 6); // WB13
    // td!(r#"(?y{w})\y\X\y"#, "ケン\xE2\x80\xAFタ", &[("ケン\xE2\x80\x", 0, 12)]),
    // td!(r#"(?y{w})\y\X\y"#, "\x21\x23", &[("\", 0, 1)]),
    // x2("(?y{w})\\y\\X\\y", "山ア", 0, 3);
    // td!(r#"(?y{w})\X"#, "3.14", &[("3.14", 0, 4)]),
    // td!(r#"(?y{w})\X"#, "3 14", &[("3", 0, 1)]),
    // td!(r#"\x40"#, "@", &[("@", 0, 1)]),
    // td!(r#"\x1"#, "\x01", &[("\", 0, 1)]),
    // td!(r#"\x{1}"#, "\x01", &[("\", 0, 1)]),
    // td!(r#"\x{4E38}"#, "\xE4\xB8\xB8", &[("\xE", 0, 3)]),
    // td!(r#"\u4E38"#, "\xE4\xB8\xB8", &[("\xE", 0, 3)]),
    td!(r#"\u0040"#, "@", &[("@", 0, 1)]),
    tu!(r#"c.*\b"#, "abc", &[("c", 2, 3)], "WordBoundary"),
    tu!(r#"\b.*abc.*\b"#, "abc", &[("abc", 0, 3)], "WordBoundary"),
    tr!(
        r#"((?()0+)+++(((0\g<0>)0)|())++++((?(1)(0\g<0>))++++++0*())++++((?(1)(0\g<1>)+)++++++++++*())++++((?(1)((0)\g<0>)+)++())+0++*+++(((0\g<0>))*())++++((?(1)(0\g<0>)+)++++++++++*|)++++*+++((?(1)((0)\g<0>)+)+++++++++())++*|)++++((?()0))|"#,
        "abcde",
        &[]
    ),
    tr!(
        r#"(?:[ab]|(*MAX{2}).)*"#,
        "abcbaaccaaa",
        &[("abcbaac", 0, 7)]
    ),
    tr!(r#"(?(?{....})123|456)"#, "123", &[("123", 0, 3)]),
    tr!(r#"(?(*FAIL)123|456)"#, "456", &[("456", 0, 3)]),
    tr!(r#"\g'0'++{,0}"#, "abcdefgh", &[]),
    tr!(r#"\g'0'++{,0}?"#, "abcdefgh", &[]),
    tr!(r#"\g'0'++{,0}b"#, "abcdefgh", &[("b", 1, 2)]),
    tr!(r#"\g'0'++{,0}?def"#, "abcdefgh", &[("def", 3, 6)]),
    tu!(r#"a{1,3}?"#, "aaa", &[("a", 0, 1)], "Non-greedy"),
    td!(r#"a{3}"#, "aaa", &[("aaa", 0, 3)]),
    tu!(r#"a{3}?"#, "aaa", &[("aaa", 0, 3)], "Non-greedy"),
    tu!(r#"a{3}?"#, "aa", &[], "Non-greedy"),
    tu!(r#"a{3,3}?"#, "aaa", &[("aaa", 0, 3)], "Non-greedy"),
    td!(r#"a{1,3}+"#, "aaaaaa", &[("aaaaaa", 0, 6)]),
    td!(r#"a{3}+"#, "aaaaaa", &[("aaaaaa", 0, 6)]),
    td!(r#"a{3,3}+"#, "aaaaaa", &[("aaaaaa", 0, 6)]),
    tr!(r#"a{3,2}b"#, "aaab", &[("aaab", 0, 4)]),
    tr!(r#"a{3,2}b"#, "aaaab", &[("aaab", 1, 5)]),
    tr!(r#"a{3,2}b"#, "aab", &[("aab", 0, 3)]),
    tr!(r#"a{3,2}?"#, "", &[]),
    td!(r#"a{2,3}+a"#, "aaa", &[("aaa", 0, 3)]),
    // td!(r#"[\x{0}-\x{7fffffff}]"#, "a", &[("a", 0, 1)]),
    // td!(r#"[\x{7f}-\x{7fffffff}]"#, "\xe5\xae\xb6", &[("\xe", 0, 3)]),
    // td!(r#"[a[cdef]]"#, "a", &[("a", 0, 1)]),
    // td!(r#"[a[xyz]-c]"#, "a", &[("a", 0, 1)]),
    // td!(r#"[a[xyz]-c]"#, "-", &[("-", 0, 1)]),
    // td!(r#"[a[xyz]-c]"#, "c", &[("c", 0, 1)]),
    // td!(r#"(a.c|def)(.{4})(?<=\1)"#, "abcdabc", &[("abcdabc", 0, 7)]),
    // td!(r#"(a.c|de)(.{4})(?<=\1)"#, "abcdabc", &[("abcdabc", 0, 7)]),
    // td!(r#"(a.c|def)(.{5})(?<=d\1e)"#, "abcdabce", &[("abcdabce", 0, 8)]),
    // td!(r#"(a.c|.)d(?<=\k<1>d)"#, "zzzzzabcdabc", &[("abcd", 5, 9)]),
    // td!(r#"(?<=az*)abc"#, "azzzzzzzzzzabcdabcabc", &[("abc", 11, 14)]),
    // td!(r#"(?<=ab|abc|abcd)ef"#, "abcdef", &[("ef", 4, 6)]),
    // td!(r#"(?<=ta+|tb+|tc+|td+)zz"#, "tcccccccccczz", &[("zz", 11, 13)]),
    // td!(r#"(?<=t.{7}|t.{5}|t.{2}|t.)zz"#, "tczz", &[("zz", 2, 4)]),
    // td!(r#"(?<=t.{7}|t.{5}|t.{2})zz"#, "tczzzz", &[("zz", 3, 5)]),
    // td!(r#"(?<=t.{7}|t.{5}|t.{3})zz"#, "tczzazzbzz", &[("zz", 8, 10)]),
    // td!(r#"(?<=(ab|abc|abcd))ef"#, "abcdef", &[("ef", 4, 6)]),
    // td!(r#"(?<=(ta+|tb+|tc+|td+))zz"#, "tcccccccccczz", &[("zz", 11, 13)]),
    // td!(r#"(?<=(t.{7}|t.{5}|t.{2}|t.))zz"#, "tczz", &[("zz", 2, 4)]),
    // td!(r#"(?<=(t.{7}|t.{5}|t.{2}))zz"#, "tczzzz", &[("zz", 3, 5)]),
    // td!(r#"(?<=(t.{7}|t.{5}|t.{3}))zz"#, "tczzazzbzz", &[("zz", 8, 10)]),
    // td!(r#"(.{1,4})(.{1,4})(?<=\2\1)"#, "abaaba", &[("abaaba", 0, 6)]),
    // td!(r#"(.{1,4})(.{1,4})(?<=\2\1)"#, "ababab", &[("ababab", 0, 6)]),
    // td!(r#"(.{1,4})(.{1,4})(?<=\2\1)"#, "abcdabceabce", &[("abceabce", 4, 12)]),
    // td!(r#"(?<=a)"#, "a", &[("", 1, 1)]),
    // td!(r#"(?<=a.*\w)z"#, "abbbz", &[("z", 4, 5)]),
    // td!(r#"(?<=a.*\W)z"#, "abb z", &[("z", 4, 5)]),
    // td!(r#"(?<=a.*\b)z"#, "abb z", &[("z", 4, 5)]),
    // td!(r#"(?<=(?>abc))"#, "abc", &[("", 3, 3)]),
    // td!(r#"(?<=a\Xz)"#, "abz", &[("", 3, 3)]),
    // td!(r#"(?<=a+.*[efg])z"#, "abcdfz", &[("z", 5, 6)]),
    // td!(r#"(?<=a+.*[efg])z"#, "abcdfgz", &[("z", 6, 7)]),
    // td!(r#"(?<=a*.*[efg])z"#, "bcdfz", &[("z", 4, 5)]),
    // td!(r#"(?<=v|t|a+.*[efg])z"#, "abcdfz", &[("z", 5, 6)]),
    // td!(r#"(?<=v|t|^a+.*[efg])z"#, "abcdfz", &[("z", 5, 6)]),
    // td!(r#"(?<=^(?:v|t|a+.*[efg]))z"#, "abcdfz", &[("z", 5, 6)]),
    // td!(r#"(?<=v|^t|a+.*[efg])z"#, "uabcdfz", &[("z", 6, 7)]),
    // td!(r#"^..(?<=(a{,2}))\1z"#, "aaz", &[("aaz", 0, 3)]),
    // td!(r#"(?<=(?<= )| )"#, "abcde fg", &[("", 6, 6)]),
    // td!(r#"(?<=D|)(?<=@!nnnnnnnnnIIIIn;{1}D?()|<x@x*xxxD|)(?<=@xxx|xxxxx\g<1>;{1}x)"#, "(?<=D|)(?<=@!nnnnnnnnnIIIIn;{1}D?()|<x@x*xxxD|)(?<=@xxx|xxxxx\\g<1>;{1}x)", &[("", 55, 55)]),
    // td!(r#"(?<=;()|)\g<1>"#, "", &[]),
    // td!(r#"(?<=;()|)\k<1>"#, ";", &[("", 1, 1)]),
    // td!(r#"(())\g<3>{0}(?<=|())"#, "abc", &[]),
    // td!(r#"(?<=()|)\1{0}"#, "abc", &[]),
    // td!(r#"(?<=(?<=abc))def"#, "abcdef", &[("def", 3, 6)]),
    // td!(r#"(?<=ab(?<=.+b)c)def"#, "abcdef", &[("def", 3, 6)]),
    // td!(r#"(?<!ab.)(?<=.bc)def"#, "abcdefcbcdef", &[("def", 9, 12)]),
    // td!(r#"(?<!x+|abc)def"#, "xxxxxxxxzdef", &[("def", 9, 12)]),
    // td!(r#"(?<!a.*z|a)def"#, "axxxxxxxzdefxxdef", &[("def", 14, 17)]),
    // td!(r#"(?<!a.*z|a)def"#, "bxxxxxxxadefxxdef", &[("def", 14, 17)]),
    // td!(r#"(?<!a.*z|a)def"#, "bxxxxxxxzdef", &[("def", 9, 12)]),
    // td!(r#"(?<!x+|y+)\d+"#, "xxx572", &[("72", 4, 6)]),
    // td!(r#"(?<!3+|4+)\d+"#, "33334444", &[("33334444", 0, 8)]),
    // td!(r#"(.{,3})..(?<!\1)"#, "abcde", &[("abcde", 0, 5)]),
    // td!(r#"(.{,3})...(?<!\1)"#, "abcde", &[("abcde", 0, 5)]),
    // td!(r#"(a.c)(.{3,}?)(?<!\1)"#, "abcabcd", &[("abcabcd", 0, 7)]),
    // td!(r#"(a*)(.{3,}?)(?<!\1)"#, "abcabcd", &[("abcab", 0, 5)]),
    // td!(r#"(?:(a.*b)|c.*d)(?<!(?(1))azzzb)"#, "azzzzb", &[("azzzzb", 0, 6)]),
    // td!(r#"<(?<!NT{+}abcd)"#, "<(?<!NT{+}abcd)", &[("<", 0, 1)]),
    // td!(r#"(?<!a.*c)def"#, "abbbbdef", &[("def", 5, 8)]),
    // td!(r#"(?<!a.*X\b)def"#, "abbbbbXdef", &[("def", 7, 10)]),
    // td!(r#"(?<!a.*[uvw])def"#, "abbbbbXdef", &[("def", 7, 10)]),
    // td!(r#"(?<!ab*\S+)def"#, "abbbbb   def", &[("def", 9, 12)]),
    // td!(r#"(?<!a.*\S)def"#, "abbbbb def", &[("def", 7, 10)]),
    // td!(r#"(?<!ab*\s+\B)def"#, "abbbbb   def", &[("def", 9, 12)]),
    // td!(r#"(?<!v|t|a+.*[efg])z"#, "abcdfzavzuz", &[("z", 10, 11)]),
    // td!(r#"(?<!v|^t|^a+.*[efg])z"#, "uabcdfz", &[("z", 6, 7)]),
    // td!(r#"(a|\k<2>)|(?<=(\k<1>))"#, "a", &[("a", 0, 1)]),
    // td!(r#"(a|\k<2>)|(?<=b(\k<1>))"#, "ba", &[("a", 1, 2)]),
    // td!(r#"(?<=RMA)X"#, "123RMAX", &[("X", 6, 7)]),
    // td!(r#"(?<=RMA)$"#, "123RMA", &[("", 6, 6)]),
    // td!(r#"(?<=RMA)\Z"#, "123RMA", &[("", 6, 6)]),
    // td!(r#"(?<=RMA)\z"#, "123RMA", &[("", 6, 6)]),
    // td!(r#"((?(a)\g<1>|b))"#, "aab", &[("aab", 0, 3)]),
    // td!(r#"((?(a)\g<1>))"#, "aab", &[("aa", 0, 2)]),
    // td!(r#"((?(a)\g<1>))"#, "", &[]),
    // td!(r#"(b(?(a)|\g<1>))"#, "bba", &[("bba", 0, 3)]),
    // td!(r#"(?(a)(?:b|c))"#, "ac", &[("ac", 0, 2)]),
    // td!(r#"(?(a)(?:b|c))"#, "", &[]),
    // td!(r#"(?(a)b)"#, "", &[]),
    // td!(r#"(?i)a|b"#, "B", &[("B", 0, 1)]),
    // td!(r#"c(?i)a|b"#, "cB", &[("cB", 0, 2)]),
    // td!(r#"c(?i)a.|b."#, "cBb", &[("cBb", 0, 3)]),
    // td!(r#"(?i)st"#, "st", &[("st", 0, 2)]),
    // td!(r#"(?i)st"#, "St", &[("St", 0, 2)]),
    // td!(r#"(?i)st"#, "sT", &[("sT", 0, 2)]),
    // td!(r#"(?i)st"#, "\xC5\xBFt", &[("\xC", 0, 3)]),
    // td!(r#"(?i)st"#, "\xEF\xAC\x85", &[("\xE", 0, 3)]),
    // td!(r#"(?i)st"#, "\xEF\xAC\x86", &[("\xE", 0, 3)]),
    // td!(r#"(?i)ast"#, "Ast", &[("Ast", 0, 3)]),
    // td!(r#"(?i)ast"#, "ASt", &[("ASt", 0, 3)]),
    // td!(r#"(?i)ast"#, "AsT", &[("AsT", 0, 3)]),
    // td!(r#"(?i)ast"#, "A\xC5\xBFt", &[("A\xC", 0, 4)]),
    // td!(r#"(?i)ast"#, "A\xEF\xAC\x85", &[("A\xE", 0, 4)]),
    // td!(r#"(?i)ast"#, "A\xEF\xAC\x86", &[("A\xE", 0, 4)]),
    // td!(r#"(?i)stZ"#, "stz", &[("stz", 0, 3)]),
    // td!(r#"(?i)stZ"#, "Stz", &[("Stz", 0, 3)]),
    // td!(r#"(?i)stZ"#, "sTz", &[("sTz", 0, 3)]),
    // td!(r#"(?i)stZ"#, "\xC5\xBFtz", &[("\xC5", 0, 4)]),
    // td!(r#"(?i)stZ"#, "\xEF\xAC\x85z", &[("\xEF", 0, 4)]),
    // td!(r#"(?i)stZ"#, "\xEF\xAC\x86z", &[("\xEF", 0, 4)]),
    // td!(r#"(?i)BstZ"#, "bstz", &[("bstz", 0, 4)]),
    // td!(r#"(?i)BstZ"#, "bStz", &[("bStz", 0, 4)]),
    // td!(r#"(?i)BstZ"#, "bsTz", &[("bsTz", 0, 4)]),
    // td!(r#"(?i)BstZ"#, "b\xC5\xBFtz", &[("b\xC5", 0, 5)]),
    // td!(r#"(?i)BstZ"#, "b\xEF\xAC\x85z", &[("b\xEF", 0, 5)]),
    // td!(r#"(?i)BstZ"#, "b\xEF\xAC\x86z", &[("b\xEF", 0, 5)]),
    // td!(r#"(?i).*st\z"#, "tttssss\xC5\xBFt", &[("tttssss\xC", 0, 10)]),
    // td!(r#"(?i).*st\z"#, "tttssss\xEF\xAC\x85", &[("tttssss\xE", 0, 10)]),
    // td!(r#"(?i).*st\z"#, "tttssss\xEF\xAC\x86", &[("tttssss\xE", 0, 10)]),
    // td!(r#"(?i).*あstい\z"#, "tttssssあ\xC5\xBFtい", &[("tttssssあ\xC5\xBF", 0, 16)]),
    // td!(r#"(?i).*あstい\z"#, "tttssssあ\xEF\xAC\x85い", &[("tttssssあ\xEF\xAC", 0, 16)]),
    // td!(r#"(?i).*あstい\z"#, "tttssssあ\xEF\xAC\x86い", &[("tttssssあ\xEF\xAC", 0, 16)]),
    // td!(r#"(?i).*\xC5\xBFt\z"#, "tttssssst", &[("tttssssst", 0, 9)]),
    // x2("(?i).*\xEF\xAC\x85\\z", "tttssssあst", 0, 12); // U+FB05
    // x2("(?i).*\xEF\xAC\x86い\\z", "tttssssstい", 0, 12); // U+FB06
    // td!(r#"(?i).*\xEF\xAC\x85\z"#, "tttssssあ\xEF\xAC\x85", &[("tttssssあ\xEF\", 0, 13)]),
    // td!(r#"(?i).*ss"#, "abcdefghijklmnopqrstuvwxyz\xc3\x9f", &[("abcdefghijklmnopqrstuvwxyz\x", 0, 28)]),
    // td!(r#"(?i).*ss.*"#, "abcdefghijklmnopqrstuvwxyz\xc3\x9fxyz", &[("abcdefghijklmnopqrstuvwxyz\xc3\", 0, 31)]),
    // td!(r#"(?i).*\xc3\x9f"#, "abcdefghijklmnopqrstuvwxyzss", &[("abcdefghijklmnopqrstuvwxyzss", 0, 28)]),
    // td!(r#"(?i).*ss.*"#, "abcdefghijklmnopqrstuvwxyzSSxyz", &[("abcdefghijklmnopqrstuvwxyzSSxyz", 0, 31)]),
    // td!(r#"(?i)ssv"#, "\xc3\x9fv", &[("\xc", 0, 3)]),
    // td!(r#"(?i)(?<=ss)v"#, "SSv", &[("v", 2, 3)]),
    // td!(r#"(?i)(?<=\xc3\x9f)v"#, "\xc3\x9fv", &[("c", 2, 3)]),
    // x2("(?i).+Isssǰ", ".+Isssǰ", 0, 8);
    // x2(".+Isssǰ", ".+Isssǰ", 0, 8);
    // x2("(?i)ǰ", "ǰ", 0, 2);
    // td!(r#"(?i)ǰ"#, "j\xcc\x8c", &[("j\x", 0, 3)]),
    // x2("(?i)j\xcc\x8c", "ǰ", 0, 2);
    // x2("(?i)5ǰ", "5ǰ", 0, 3);
    // td!(r#"(?i)5ǰ"#, "5j\xcc\x8c", &[("5j\x", 0, 4)]),
    // x2("(?i)5j\xcc\x8c", "5ǰ", 0, 3);
    // x2("(?i)ǰv", "ǰV", 0, 3);
    // td!(r#"(?i)ǰv"#, "j\xcc\x8cV", &[("j\xc", 0, 4)]),
    // x2("(?i)j\xcc\x8cv", "ǰV", 0, 3);
    // x2("(?i)[ǰ]", "ǰ", 0, 2);
    // td!(r#"(?i)[ǰ]"#, "j\xcc\x8c", &[("j\x", 0, 3)]),
    // td!(r#"(?i)\ufb00a"#, "ffa", &[("ffa", 0, 3)]),
    // td!(r#"(?i)ffz"#, "\xef\xac\x80z", &[("\xef", 0, 4)]),
    // td!(r#"(?i)\u2126"#, "\xcf\x89", &[("\x", 0, 2)]),
    // td!(r#"a(?i)\u2126"#, "a\xcf\x89", &[("a\x", 0, 3)]),
    // td!(r#"(?i)A\u2126"#, "a\xcf\x89", &[("a\x", 0, 3)]),
    // td!(r#"(?i)A\u2126="#, "a\xcf\x89=", &[("a\xc", 0, 4)]),
    // td!(r#"(?i:ss)=1234567890"#, "\xc5\xbf\xc5\xbf=1234567890", &[("\xc5\xbf\xc5\xb", 0, 15)]),
    // td!(r#"\x{000A}"#, "\x0a", &[("\", 0, 1)]),
    // td!(r#"\x{000A 002f}"#, "\x0a\x2f", &[("\x", 0, 2)]),
    // td!(r#"\x{000A 002f }"#, "\x0a\x2f", &[("\x", 0, 2)]),
    // td!(r#"\x{007C     001b}"#, "\x7c\x1b", &[("\x", 0, 2)]),
    // td!(r#"\x{1 2 3 4 5 6 7 8 9 a b c d e f}"#, "\x01\x02\x3\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f", &[("\x01\x02\x3\x04", 0, 15)]),
    // td!(r#"a\x{000A 002f}@"#, "a\x0a\x2f@", &[("a\x0", 0, 4)]),
    // td!(r#"a\x{0060\n0063}@"#, "a\x60\x63@", &[("a\x6", 0, 4)]),
    // td!(r#"\o{102}"#, "B", &[("B", 0, 1)]),
    // td!(r#"\o{102 103}"#, "BC", &[("BC", 0, 2)]),
    // td!(r#"\o{0160 0000161}"#, "pq", &[("pq", 0, 2)]),
    // td!(r#"\o{1 2 3 4 5 6 7 10 11 12 13 14 15 16 17}"#, "\x01\x02\x3\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f", &[("\x01\x02\x3\x04", 0, 15)]),
    // td!(r#"\o{0007 0010 }"#, "\x07\x08", &[("\x", 0, 2)]),
    // td!(r#"[\x{000A}]"#, "\x0a", &[("\", 0, 1)]),
    // td!(r#"[\x{000A 002f}]+"#, "\x0a\x2f\x2e", &[("\x", 0, 2)]),
    // td!(r#"[\x{01 0F 1A 2c 4B}]+"#, "\x20\x01\x0f\x1a\x2c\x4b\x1b", &[("x20\x", 1, 6)]),
    // td!(r#"[\x{0020 0024}-\x{0026}]+"#, "\x25\x24\x26\x23", &[("\x2", 0, 3)]),
    // td!(r#"[\x{0030}-\x{0033 005a}]+"#, "\x30\x31\x32\x33\x5a\34", &[("\x30\", 0, 5)]),
    // td!(r#"[\o{102}]"#, "B", &[("B", 0, 1)]),
    // td!(r#"[\o{102 103}]*"#, "BC", &[("BC", 0, 2)]),
    // td!(r#"[\x{0030-0039}]+"#, "abc0123456789def", &[("0123456789", 3, 13)]),
    // td!(r#"[\x{0030 - 0039 }]+"#, "abc0123456789def", &[("0123456789", 3, 13)]),
    // td!(r#"[\x{0030 - 0039 0063 0064}]+"#, "abc0123456789def", &[("c0123456789d", 2, 14)]),
    // td!(r#"[\x{0030 - 0039 0063-0065}]+"#, "acde019b", &[("cde019", 1, 7)]),
    // td!(r#"[a-\x{0063 0071}]+"#, "dabcqz", &[("abcq", 1, 5)]),
    // td!(r#"[-\x{0063-0065}]+"#, "ace-df", &[("ce-d", 1, 5)]),
    // td!(r#"[\x61-\x{0063 0065}]+"#, "abced", &[("abce", 0, 4)]),
    // td!(r#"[t\x{0063 0071}]+"#, "tcqb", &[("tcq", 0, 3)]),
    // td!(r#"[\W\x{0063 0071}]+"#, "*cqa", &[("*cq", 0, 3)]),
    // td!(r#"(\O|(?=z\g<2>*))(\g<0>){0}"#, "a", &[("a", 0, 1)]),
    // td!(r#"(?Ii)abc"#, "abc", &[("abc", 0, 3)]),
    // td!(r#"(?Ii)abc"#, "ABC", &[("ABC", 0, 3)]),
    // td!(r#"(?Ii:abc)"#, "abc", &[("abc", 0, 3)]),
    // td!(r#"(?Ii)xyz|abc"#, "aBc", &[("aBc", 0, 3)]),
    // td!(r#"(?Ii:zz|abc|AZ)"#, "ABc", &[("ABc", 0, 3)]),
    // td!(r#"(?I-i:abc)"#, "abc", &[("abc", 0, 3)]),
    // td!(r#"(?i)\xe2\x84\xaa"#, "k", &[("k", 0, 1)]),
    // td!(r#"(?:(?Ii)abc)"#, "ABC", &[("ABC", 0, 3)]),
    // td!(r#"(?:(?:(?Ii)abc))"#, "ABC", &[("ABC", 0, 3)]),
    // td!(r#"(?Ii)$"#, "", &[]),
    // td!(r#"(?Ii)|"#, "", &[]),
    td!(r#"a*"#, "aabcaaa", &[("aa", 0, 2), ("aaa", 4, 7)]),
    // td!(r#"(?L)a*"#, "aabcaaa", &[("aaa", 4, 7)]),
    // td!(r#"(?L)a{4}|a{3}|b*"#, "baaaaabbb", &[("aaaa", 1, 5)]),
    // td!(r#"(?L)a{3}|a{4}|b*"#, "baaaaabbb", &[("aaaa", 1, 5)]),
    // td!(r#"(?L)z|a\g<0>a"#, "aazaa", &[("aazaa", 0, 5)]),
    // td!(r#"(?Li)z|a\g<0>a"#, "aazAA", &[("aazAA", 0, 5)]),
    // td!(r#"(?Li:z|a\g<0>a)"#, "aazAA", &[("aazAA", 0, 5)]),
    // td!(r#"(?L)z|a\g<0>a"#, "aazaaaazaaaa", &[("aaaazaaaa", 3, 12)]),
    // td!(r#"(?iI)(?:[[:word:]])"#, "\xc5\xbf", &[("\x", 0, 2)]),
    // td!(r#"(?iW:[[:word:]])"#, "\xc5\xbf", &[("\x", 0, 2)]),
    // td!(r#"(?iW:[\p{Word}])"#, "\xc5\xbf", &[("\x", 0, 2)]),
    // td!(r#"(?iW:[\w])"#, "\xc5\xbf", &[("\x", 0, 2)]),
    // td!(r#"(?i)\p{Word}"#, "\xc5\xbf", &[("\x", 0, 2)]),
    // td!(r#"(?i)\w"#, "\xc5\xbf", &[("\x", 0, 2)]),
    // td!(r#"(?iW:[[:^word:]])"#, "\xc5\xbf", &[("\x", 0, 2)]),
    // td!(r#"(?iW:[\P{Word}])"#, "\xc5\xbf", &[("\x", 0, 2)]),
    // td!(r#"(?iW:[\W])"#, "\xc5\xbf", &[("\x", 0, 2)]),
    // td!(r#"(?iW:\P{Word})"#, "\xc5\xbf", &[("\x", 0, 2)]),
    // td!(r#"(?iW:\W)"#, "\xc5\xbf", &[("\x", 0, 2)]),
    // td!(r#"(?iW:[[:^word:]])"#, "s", &[("s", 0, 1)]),
    // td!(r#"(?iW:[\P{Word}])"#, "s", &[("s", 0, 1)]),
    // td!(r#"(?iW:[\W])"#, "s", &[("s", 0, 1)]),
    td!(r#"[[:punct:]]"#, ":", &[(":", 0, 1)]),
    td!(r#"[[:punct:]]"#, "$", &[("$", 0, 1)]),
    td!(r#"[[:punct:]]+"#, "$+<=>^`|~", &[("$+<=>^`|~", 0, 9)]),
    // x2("\\p{PosixPunct}+", "$¦", 0, 3);
    // td!(r#"\A.*\R"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\A\O*\R"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\A\n*\R"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\A\R*\R"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\At*\R"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\A.{0,99}\R"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\A\O{0,99}\R"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\A\n{0,99}\R"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\A\R{0,99}\R"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\At{0,99}\R"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\A.*\n"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\A.{0,99}\n"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\A.*\O"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\A.{0,99}\O"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\A.*\s"#, "\n", &[("\", 0, 1)]),
    // td!(r#"\A.{0,99}\s"#, "\n", &[("\", 0, 1)]),
    // td!(r#"000||0\xfa"#, "0", &[]),
    // x2("aaaaaaaaaaaaaaaaaaaaaaaあb", "aaaaaaaaaaaaaaaaaaaaaaaあb", 0, 27); /* Issue #221 */
    // td!(r#"\p{Common}"#, "\xe3\x8b\xbf", &[("\xe", 0, 3)]),
    // td!(r#"\p{In_Enclosed_CJK_Letters_and_Months}"#, "\xe3\x8b\xbf", &[("\xe", 0, 3)]),
    // td!(r#"(?:)*"#, "abc", &[]),
];

#[test]
fn match_test() {
    for (test_number, test_data) in TEST_DATA.iter().enumerate() {
        // Create a scanner from the scanner builder with a single pattern.
        let scanner = ScannerBuilder::new()
            .add_patterns(vec![test_data.pattern])
            .build();
        match scanner {
            Ok(scanner) => {
                assert!(
                    test_data.error_msg.is_none(),
                    "#{}: Parsing regex should fail: {:?}",
                    test_number,
                    test_data,
                );
                // Scanner build succeeded. Check if the matches are as expected.
                let matches = scanner.find_iter(test_data.input).collect::<Vec<_>>();
                assert_eq!(
                    matches.len(),
                    test_data.expected.len(),
                    "#{}, Differing matches count {:?}, {:?}",
                    test_number,
                    test_data,
                    matches
                );
                for (matched, (expected_match, expected_start, expected_end)) in
                    matches.iter().zip(test_data.expected.iter())
                {
                    assert_eq!(
                        &test_data.input[matched.span().start..matched.span().end],
                        *expected_match,
                        "#{}: {:?} {:?}",
                        test_number,
                        test_data,
                        matched
                    );
                    assert_eq!(
                        matched.start(),
                        *expected_start,
                        "#{} Match start ",
                        test_number
                    );
                    assert_eq!(matched.end(), *expected_end, "#{} Match end ", test_number);
                }
            }
            Err(e) => {
                // Scanner build failed. Check if the error message is as expected.
                assert!(
                    test_data.error_msg.is_some(),
                    "#{}: Unexpected error: {}, {:?}",
                    test_number,
                    e,
                    test_data
                );
                let msg = e.to_string();
                assert!(
                    msg.contains(test_data.error_msg.unwrap()),
                    "#{}:\n'{}'\ndoes not contain\n'{}'",
                    test_number,
                    e,
                    test_data.error_msg.unwrap()
                );
            }
        }
    }
}
