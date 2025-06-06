#![cfg(not(feature = "regex_automata"))]
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
    test_number: usize,
}

// A macros to easily create a TestData struct.

// Valid pattern, input, and expected matches.
macro_rules! td {
    ($pattern:expr, $input:expr, $expected:expr, $num:expr) => {
        TestData {
            pattern: $pattern,
            input: $input,
            expected: $expected,
            error_msg: None,
            test_number: $num,
        }
    };
}

// Invalid pattern and expected error message.
macro_rules! tu {
    ($pattern:expr, $input:expr, $expected:expr, $result:expr, $num:expr) => {
        TestData {
            pattern: $pattern,
            input: "",
            expected: &[],
            error_msg: Some($result),
            test_number: $num,
        }
    };
}

const ERROR_MSG: &str = "regex parse error";

// Pattern that causes a regex parse error
macro_rules! tr {
    ($pattern:expr, $input:expr, $expected:expr, $num:expr) => {
        TestData {
            pattern: $pattern,
            input: "",
            expected: &[],
            error_msg: Some(ERROR_MSG),
            test_number: $num,
        }
    };
}

// -------------------------------------------------------------------------------------------------
// Here is how the tests are categorized:
// * td macros indicate a valid pattern, input, and expected matches.
// * tr macros indicate a pattern that causes a regex parse error from the regex-syntax crate.
// * tu macros indicate a pattern that contains an unsupported feature.
// * Commented out tests are either not yet covered or do not compile with Rust.
// * Commented out tests with name x2 couldn't be converted to Rust automatically.
const TEST_DATA: &[TestData] = &[
    // ---------------------------------------------------------------------------------------------
    // The following tests are extracted from the test_utf8.c file from the Oniguruma project.
    // The tests have been converted to Rust by the extract.ps1 script.
    // ---------------------------------------------------------------------------------------------
    td!(r#""#, "", &[], 0),
    tu!(r#"^"#, "", &[], "StartLine", 1),
    tu!(r#"^a"#, "\na", &[("n", 1, 2)], "StartLine", 2),
    tu!(r#"$"#, "", &[], "EndLine", 3),
    tr!(r#"$\O"#, "bb\n", &[("", 2, 3)], 4),
    tr!(r#"\G"#, "", &[], 5),
    tu!(r#"\A"#, "", &[], "StartText", 6),
    tr!(r#"\Z"#, "", &[], 7),
    tu!(r#"\z"#, "", &[], "EndText", 8),
    tu!(r#"^$"#, "", &[], "StartLine", 9),
    // td!(r#"\ca"#, "\001", &[("\", 0, 1)], 10),
    // td!(r#"\C-b"#, "\002", &[("\", 0, 1)], 11),
    // td!(r#"\c\\"#, "\034", &[("\", 0, 1)], 12),
    // td!(r#"q[\c\\]"#, "q\034", &[("q\", 0, 2)], 13),
    td!(r#""#, "a", &[], 14),
    td!(r#"a"#, "a", &[("a", 0, 1)], 15),
    td!(r#"\x61"#, "a", &[("a", 0, 1)], 16),
    td!(r#"aa"#, "aa", &[("aa", 0, 2)], 17),
    td!(r#"aaa"#, "aaa", &[("aaa", 0, 3)], 18),
    td!(
        r#"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"#,
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        &[("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", 0, 35)],
        19
    ),
    td!(r#"ab"#, "ab", &[("ab", 0, 2)], 20),
    td!(r#"b"#, "ab", &[("b", 1, 2)], 21),
    td!(r#"bc"#, "abc", &[("bc", 1, 3)], 22),
    tu!(
        r#"(?i:#RET#)"#,
        "#INS##RET#",
        &[("#RET#", 5, 10)],
        "CaseInsensitive",
        23
    ),
    // td!(r#"\17"#, "\017", &[("\", 0, 1)], 24),
    td!(r#"\x1f"#, "\x1f", &[("\x1f", 0, 1)], 25),
    tr!(r#"a(?#....\\JJJJ)b"#, "ab", &[("ab", 0, 2)], 26),
    tu!(
        r#"(?x)  G (o O(?-x)oO) g L"#,
        "GoOoOgLe",
        &[("GoOoOgL", 0, 7)],
        "IgnoreWhitespace",
        27
    ),
    td!(r#"."#, "a", &[("a", 0, 1)], 28),
    td!(r#".."#, "ab", &[("ab", 0, 2)], 29),
    td!(r#"\w"#, "e", &[("e", 0, 1)], 30),
    td!(r#"\s"#, " ", &[(" ", 0, 1)], 31),
    td!(r#"\S"#, "b", &[("b", 0, 1)], 32),
    td!(r#"\d"#, "4", &[("4", 0, 1)], 33),
    tu!(r#"\b"#, "z ", &[], "WordBoundary", 34),
    tu!(r#"\b"#, " z", &[("", 1, 1)], "WordBoundary", 35),
    tu!(r#"\b"#, "  z ", &[("", 2, 2)], "WordBoundary", 36),
    tu!(r#"\B"#, "zz ", &[("", 1, 1)], "NotWordBoundary", 37),
    tu!(r#"\B"#, "z ", &[("", 2, 2)], "NotWordBoundary", 38),
    tu!(r#"\B"#, " z", &[], "NotWordBoundary", 39),
    td!(r#"[ab]"#, "b", &[("b", 0, 1)], 40),
    td!(r#"[a-z]"#, "t", &[("t", 0, 1)], 41),
    td!(r#"[^a]"#, "\n", &[("\n", 0, 1)], 42),
    td!(r#"[]]"#, "]", &[("]", 0, 1)], 43),
    td!(r#"[\^]+"#, "0^^1", &[("^^", 1, 3)], 44),
    td!(r#"[b-]"#, "b", &[("b", 0, 1)], 45),
    td!(r#"[b-]"#, "-", &[("-", 0, 1)], 46),
    td!(r#"[\w]"#, "z", &[("z", 0, 1)], 47),
    td!(r#"[\W]"#, "b$", &[("$", 1, 2)], 48),
    td!(r#"[\d]"#, "5", &[("5", 0, 1)], 49),
    td!(r#"[\D]"#, "t", &[("t", 0, 1)], 50),
    td!(r#"[\s]"#, " ", &[(" ", 0, 1)], 51),
    td!(r#"[\S]"#, "b", &[("b", 0, 1)], 52),
    td!(r#"[\w\d]"#, "2", &[("2", 0, 1)], 53),
    td!(r#"[[:upper:]]"#, "B", &[("B", 0, 1)], 54),
    td!(r#"[*[:xdigit:]+]"#, "+", &[("+", 0, 1)], 55),
    td!(
        r#"[*[:xdigit:]+]"#,
        "GHIKK-9+*",
        &[("9", 6, 7), ("+", 7, 8), ("*", 8, 9)], // 56
        57
    ),
    td!(r#"[*[:xdigit:]+]"#, "-@^+", &[("+", 3, 4)], 58),
    td!(r#"[[:upper]]"#, ":", &[(":", 0, 1)], 59),
    td!(r#"[[:^upper:]]"#, "a", &[("a", 0, 1)], 60),
    td!(r#"[[:^lower:]]"#, "A", &[("A", 0, 1)], 61),
    td!(r#"[[:upper\] :]]"#, "]", &[("]", 0, 1)], 62),
    td!(r#"[[::]]"#, ":", &[(":", 0, 1)], 63),
    td!(r#"[[:::]]"#, ":", &[(":", 0, 1)], 64),
    td!(r#"[[:\]:]]*"#, ":]", &[(":]", 0, 2)], 65),
    td!(r#"[[:\[:]]*"#, ":[", &[(":[", 0, 2)], 66),
    td!(r#"[[:\]]]*"#, ":]", &[(":]", 0, 2)], 67),
    td!(r#"[\x24-\x27]"#, "\x26", &[("\x26", 0, 1)], 68),
    td!(r#"[\x5a-\x5c]"#, "\x5b", &[("\x5b", 0, 1)], 69),
    td!(r#"[\x6A-\x6D]"#, "\x6c", &[("\x6c", 0, 1)], 70),
    td!(r#"[\[]"#, "[", &[("[", 0, 1)], 71),
    td!(r#"[\]]"#, "]", &[("]", 0, 1)], 72),
    td!(r#"[&]"#, "&", &[("&", 0, 1)], 73),
    td!(r#"[[ab]]"#, "b", &[("b", 0, 1)], 74),
    td!(r#"[[ab]c]"#, "c", &[("c", 0, 1)], 75),
    td!(r#"[[ab]&&bc]"#, "b", &[("b", 0, 1)], 76),
    td!(r#"[a-z&&b-y&&c-x]"#, "w", &[("w", 0, 1)], 77),
    td!(r#"[[^a&&a]&&a-z]"#, "b", &[("b", 0, 1)], 78),
    td!(r#"[[^a-z&&bcdef]&&[^c-g]]"#, "h", &[("h", 0, 1)], 79),
    td!(r#"[^[^abc]&&[^cde]]"#, "c", &[("c", 0, 1)], 80),
    td!(r#"[^[^abc]&&[^cde]]"#, "e", &[("e", 0, 1)], 81),
    tr!(r#"[a-&&-a]"#, "-", &[("-", 0, 1)], 82),
    td!(r#"a\Wbc"#, "a bc", &[("a bc", 0, 4)], 83),
    td!(r#"a.b.c"#, "aabbc", &[("aabbc", 0, 5)], 84),
    td!(r#".\wb\W..c"#, "abb bcc", &[("abb bcc", 0, 7)], 85),
    td!(r#"\s\wzzz"#, " zzzz", &[(" zzzz", 0, 5)], 86),
    td!(r#"aa.b"#, "aabb", &[("aabb", 0, 4)], 87),
    td!(r#".a"#, "aa", &[("aa", 0, 2)], 88),
    tu!(r#"^a"#, "a", &[("a", 0, 1)], "StartLine", 89),
    tu!(r#"^a$"#, "a", &[("a", 0, 1)], "StartLine", 90),
    tu!(r#"^\w$"#, "a", &[("a", 0, 1)], "StartLine", 91),
    tu!(r#"^\wab$"#, "zab", &[("zab", 0, 3)], "StartLine", 92),
    tu!(
        r#"^\wabcdef$"#,
        "zabcdef",
        &[("zabcdef", 0, 7)],
        "StartLine",
        93
    ),
    tu!(
        r#"^\w...def$"#,
        "zabcdef",
        &[("zabcdef", 0, 7)],
        "StartLine",
        94
    ),
    td!(r#"\w\w\s\Waaa\d"#, "aa  aaa4", &[("aa  aaa4", 0, 8)], 95),
    tr!(r#"\A\Z"#, "", &[], 96),
    tu!(r#"\Axyz"#, "xyz", &[("xyz", 0, 3)], "StartText", 97),
    tr!(r#"xyz\Z"#, "xyz", &[("xyz", 0, 3)], 98),
    tu!(r#"xyz\z"#, "xyz", &[("xyz", 0, 3)], "EndText", 99),
    tr!(r#"a\Z"#, "a", &[("a", 0, 1)], 100),
    tr!(r#"\Gaz"#, "az", &[("az", 0, 2)], 101),
    td!(r#"\^\$"#, "^$", &[("^$", 0, 2)], 102),
    tu!(r#"^x?y"#, "xy", &[("xy", 0, 2)], "StartLine", 103),
    tu!(r#"^(x?y)"#, "xy", &[("xy", 0, 2)], "StartLine", 104),
    td!(r#"\w"#, "_", &[("_", 0, 1)], 105),
    tr!(r#"(?=z)z"#, "z", &[("z", 0, 1)], 106),
    tr!(r#"(?!z)a"#, "a", &[("a", 0, 1)], 107),
    tu!(r#"(?i:a)"#, "a", &[("a", 0, 1)], "CaseInsensitive", 108),
    tu!(r#"(?i:a)"#, "A", &[("A", 0, 1)], "CaseInsensitive", 109),
    tu!(r#"(?i:A)"#, "a", &[("a", 0, 1)], "CaseInsensitive", 110),
    tu!(r#"(?i:i)"#, "I", &[("I", 0, 1)], "CaseInsensitive", 111),
    tu!(r#"(?i:I)"#, "i", &[("i", 0, 1)], "CaseInsensitive", 112),
    tu!(r#"(?i:[A-Z])"#, "i", &[("i", 0, 1)], "CaseInsensitive", 113),
    tu!(r#"(?i:[a-z])"#, "I", &[("I", 0, 1)], "CaseInsensitive", 114),
    tu!(r#"(?i:ss)"#, "ss", &[("ss", 0, 2)], "CaseInsensitive", 115),
    tu!(r#"(?i:ss)"#, "Ss", &[("Ss", 0, 2)], "CaseInsensitive", 116),
    tu!(r#"(?i:ss)"#, "SS", &[("SS", 0, 2)], "CaseInsensitive", 117),
    // tu!(r#"(?i:ss)"#, "\xc5\xbfS", &[("\xc", 0, 3)], "CaseInsensitive", 118),
    // tu!(r#"(?i:ss)"#, "s\xc5\xbf", &[("s\x", 0, 3)], "CaseInsensitive", 119),
    // tu!(r#"(?i:ss)"#, "\xc3\x9f", &[("\x", 0, 2)], "CaseInsensitive", 120),
    // tu!(r#"(?i:ss)"#, "\xe1\xba\x9e", &[("\xe", 0, 3)], "CaseInsensitive", 121),
    tu!(
        r#"(?i:xssy)"#,
        "xssy",
        &[("xssy", 0, 4)],
        "CaseInsensitive",
        122
    ),
    tu!(
        r#"(?i:xssy)"#,
        "xSsy",
        &[("xSsy", 0, 4)],
        "CaseInsensitive",
        123
    ),
    tu!(
        r#"(?i:xssy)"#,
        "xSSy",
        &[("xSSy", 0, 4)],
        "CaseInsensitive",
        124
    ),
    // tu!(r#"(?i:xssy)"#, "x\xc5\xbfSy", &[("x\xc5", 0, 5)], "CaseInsensitive", 125),
    // tu!(r#"(?i:xssy)"#, "xs\xc5\xbfy", &[("xs\xc", 0, 5)], "CaseInsensitive", 126),
    // tu!(r#"(?i:xssy)"#, "x\xc3\x9fy", &[("x\xc", 0, 4)], "CaseInsensitive", 127),
    // tu!(r#"(?i:xssy)"#, "x\xe1\xba\x9ey", &[("x\xe1", 0, 5)], "CaseInsensitive", 128),
    tu!(
        r#"(?i:x\xc3\x9fy)"#,
        "xssy",
        &[("xssy", 0, 4)],
        "CaseInsensitive",
        129
    ),
    tu!(
        r#"(?i:x\xc3\x9fy)"#,
        "xSSy",
        &[("xSSy", 0, 4)],
        "CaseInsensitive",
        130
    ),
    tu!(
        r#"(?i:\xc3\x9f)"#,
        "ss",
        &[("ss", 0, 2)],
        "CaseInsensitive",
        131
    ),
    tu!(
        r#"(?i:\xc3\x9f)"#,
        "SS",
        &[("SS", 0, 2)],
        "CaseInsensitive",
        132
    ),
    tu!(
        r#"(?i:[\xc3\x9f])"#,
        "ss",
        &[("ss", 0, 2)],
        "CaseInsensitive",
        133
    ),
    tu!(
        r#"(?i:[\xc3\x9f])"#,
        "SS",
        &[("SS", 0, 2)],
        "CaseInsensitive",
        134
    ),
    tr!(r#"(?i)(?<!ss)z"#, "qqz", &[("z", 2, 3)], 135),
    tu!(r#"(?i:[A-Z])"#, "a", &[("a", 0, 1)], "CaseInsensitive", 136),
    tu!(r#"(?i:[f-m])"#, "H", &[("H", 0, 1)], "CaseInsensitive", 137),
    tu!(r#"(?i:[f-m])"#, "h", &[("h", 0, 1)], "CaseInsensitive", 138),
    tu!(r#"(?i:[A-c])"#, "D", &[("D", 0, 1)], "CaseInsensitive", 139),
    tu!(r#"(?i:[!-k])"#, "Z", &[("Z", 0, 1)], "CaseInsensitive", 140),
    tu!(r#"(?i:[!-k])"#, "7", &[("7", 0, 1)], "CaseInsensitive", 141),
    tu!(r#"(?i:[T-}])"#, "b", &[("b", 0, 1)], "CaseInsensitive", 142),
    tu!(r#"(?i:[T-}])"#, "{", &[("{", 0, 1)], "CaseInsensitive", 143),
    tu!(r#"(?i:\?a)"#, "?A", &[("?A", 0, 2)], "CaseInsensitive", 144),
    tu!(r#"(?i:\*A)"#, "*a", &[("*a", 0, 2)], "CaseInsensitive", 145),
    tu!(r#"(?m:.)"#, "\n", &[("\n", 0, 1)], "MultiLine", 146),
    tu!(r#"(?m:a.)"#, "a\n", &[("a\n", 0, 2)], "MultiLine", 147),
    tu!(r#"(?m:.b)"#, "a\nb", &[("\n", 1, 3)], "MultiLine", 148),
    td!(r#".*abc"#, "dddabdd\nddabc", &[("ddabc", 8, 13)], 149),
    td!(
        r#".+abc"#,
        "dddabdd\nddabcaa\naaaabc",
        &[("ddabc", 8, 13), ("aaaabc", 16, 22)], // 150
        151
    ),
    tu!(
        r#"(?m:.*abc)"#,
        "dddabddabc",
        &[("dddabddabc", 0, 10)],
        "MultiLine",
        152
    ),
    td!(r#"a?"#, "", &[], 153),
    td!(r#"a?"#, "b", &[], 154),
    td!(r#"a?"#, "a", &[("a", 0, 1)], 155),
    td!(r#"a*"#, "", &[], 156),
    td!(r#"a*"#, "a", &[("a", 0, 1)], 157),
    td!(r#"a*"#, "aaa", &[("aaa", 0, 3)], 158),
    td!(r#"a*"#, "baaaa", &[("aaaa", 1, 5)], 159),
    td!(r#"a+"#, "a", &[("a", 0, 1)], 160),
    td!(r#"a+"#, "aaaa", &[("aaaa", 0, 4)], 161),
    td!(r#"a+"#, "aabbb", &[("aa", 0, 2)], 162),
    td!(r#"a+"#, "baaaa", &[("aaaa", 1, 5)], 163),
    td!(r#".?"#, "", &[], 164),
    td!(r#".?"#, "f", &[("f", 0, 1)], 165),
    td!(r#".?"#, "\n", &[], 166),
    td!(r#".*"#, "", &[], 167),
    td!(r#".*"#, "abcde", &[("abcde", 0, 5)], 168),
    td!(r#".+"#, "z", &[("z", 0, 1)], 169),
    td!(r#".+"#, "zdswer\n", &[("zdswer", 0, 6)], 170),
    tr!(r#"(.*)a\1f"#, "babfbac", &[("babf", 0, 4)], 171),
    tr!(r#"(.*)a\1f"#, "bacbabf", &[("babf", 3, 7)], 172),
    tr!(r#"((.*)a\2f)"#, "bacbabf", &[("babf", 3, 7)], 173),
    tr!(
        r#"(.*)a\1f"#,
        "baczzzzzz\nbazz\nzzzzbabf",
        &[("zzba", 19, 23)],
        174
    ),
    td!(r#"(?:x?)?"#, "", &[], 175),
    td!(r#"(?:x?)?"#, "x", &[("x", 0, 1)], 176),
    td!(r#"(?:x?)?"#, "xx", &[("x", 0, 1), ("x", 1, 2)], 177),
    td!(r#"(?:x?)*"#, "", &[], 178),
    td!(r#"(?:x?)*"#, "x", &[("x", 0, 1)], 179),
    td!(r#"(?:x?)*"#, "xx", &[("xx", 0, 2)], 180),
    td!(r#"(?:x?)+"#, "", &[], 181),
    td!(r#"(?:x?)+"#, "x", &[("x", 0, 1)], 182),
    td!(r#"(?:x?)+"#, "xx", &[("xx", 0, 2)], 183),
    td!(r#"(?:x?)\?\?"#, "", &[], 184),
    td!(r#"(?:x?)\?\?"#, "x", &[], 185),
    td!(r#"(?:x?)\?\?"#, "xx", &[], 186),
    tu!(r#"(?:x?)*?"#, "", &[], "Non-greedy", 187),
    tu!(r#"(?:x?)*?"#, "x", &[], "Non-greedy", 188),
    tu!(r#"(?:x?)*?"#, "xx", &[], "Non-greedy", 189),
    tu!(r#"(?:x?)+?"#, "", &[], "Non-greedy", 190),
    tu!(r#"(?:x?)+?"#, "x", &[("x", 0, 1)], "Non-greedy", 191),
    tu!(r#"(?:x?)+?"#, "xx", &[("x", 0, 1)], "Non-greedy", 192),
    td!(r#"(?:x*)?"#, "", &[], 193),
    td!(r#"(?:x*)?"#, "x", &[("x", 0, 1)], 194),
    td!(r#"(?:x*)?"#, "xx", &[("xx", 0, 2)], 195),
    td!(r#"(?:x*)*"#, "", &[], 196),
    td!(r#"(?:x*)*"#, "x", &[("x", 0, 1)], 197),
    td!(r#"(?:x*)*"#, "xx", &[("xx", 0, 2)], 198),
    td!(r#"(?:x*)+"#, "", &[], 199),
    td!(r#"(?:x*)+"#, "x", &[("x", 0, 1)], 200),
    td!(r#"(?:x*)+"#, "xx", &[("xx", 0, 2)], 201),
    td!(r#"(?:x*)\?\?"#, "", &[], 202),
    td!(r#"(?:x*)\?\?"#, "x", &[], 203),
    td!(r#"(?:x*)\?\?"#, "xx", &[], 204),
    tu!(r#"(?:x*)*?"#, "", &[], "Non-greedy", 205),
    tu!(r#"(?:x*)*?"#, "x", &[], "Non-greedy", 206),
    tu!(r#"(?:x*)*?"#, "xx", &[], "Non-greedy", 207),
    tu!(r#"(?:x*)+?"#, "", &[], "Non-greedy", 208),
    tu!(r#"(?:x*)+?"#, "x", &[("x", 0, 1)], "Non-greedy", 209),
    tu!(r#"(?:x*)+?"#, "xx", &[("xx", 0, 2)], "Non-greedy", 210),
    td!(r#"(?:x+)?"#, "", &[], 211),
    td!(r#"(?:x+)?"#, "x", &[("x", 0, 1)], 212),
    td!(r#"(?:x+)?"#, "xx", &[("xx", 0, 2)], 213),
    td!(r#"(?:x+)*"#, "", &[], 214),
    td!(r#"(?:x+)*"#, "x", &[("x", 0, 1)], 215),
    td!(r#"(?:x+)*"#, "xx", &[("xx", 0, 2)], 216),
    td!(r#"(?:x+)+"#, "x", &[("x", 0, 1)], 217),
    td!(r#"(?:x+)+"#, "xx", &[("xx", 0, 2)], 218),
    td!(r#"(?:x+)\?\?"#, "", &[], 219),
    td!(r#"(?:x+)\?\?"#, "x", &[], 220),
    td!(r#"(?:x+)\?\?"#, "xx", &[], 221),
    tu!(r#"(?:x+)*?"#, "", &[], "Non-greedy", 222),
    tu!(r#"(?:x+)*?"#, "x", &[], "Non-greedy", 223),
    tu!(r#"(?:x+)*?"#, "xx", &[], "Non-greedy", 224),
    tu!(r#"(?:x+)+?"#, "x", &[("x", 0, 1)], "Non-greedy", 225),
    tu!(r#"(?:x+)+?"#, "xx", &[("xx", 0, 2)], "Non-greedy", 226),
    td!(r#"(?:x\?\?)?"#, "", &[], 227),
    td!(r#"(?:x\?\?)?"#, "x", &[], 228),
    td!(r#"(?:x\?\?)?"#, "xx", &[], 229),
    td!(r#"(?:x\?\?)*"#, "", &[], 230),
    td!(r#"(?:x\?\?)*"#, "x", &[], 231),
    td!(r#"(?:x\?\?)*"#, "xx", &[], 232),
    td!(r#"(?:x\?\?)+"#, "", &[], 233),
    td!(r#"(?:x\?\?)+"#, "x", &[], 234),
    td!(r#"(?:x\?\?)+"#, "xx", &[], 235),
    td!(r#"(?:x\?\?)\?\?"#, "", &[], 236),
    td!(r#"(?:x\?\?)\?\?"#, "x", &[], 237),
    td!(r#"(?:x\?\?)\?\?"#, "xx", &[], 238),
    tu!(r#"(?:x\?\?)*?"#, "", &[], "Non-greedy", 239),
    tu!(r#"(?:x\?\?)*?"#, "x", &[], "Non-greedy", 240),
    tu!(r#"(?:x\?\?)*?"#, "xx", &[], "Non-greedy", 241),
    tu!(r#"(?:x\?\?)+?"#, "", &[], "Non-greedy", 242),
    tu!(r#"(?:x\?\?)+?"#, "x", &[], "Non-greedy", 243),
    tu!(r#"(?:x\?\?)+?"#, "xx", &[], "Non-greedy", 244),
    tu!(r#"(?:x*?)?"#, "", &[], "Non-greedy", 245),
    tu!(r#"(?:x*?)?"#, "x", &[], "Non-greedy", 246),
    tu!(r#"(?:x*?)?"#, "xx", &[], "Non-greedy", 247),
    tu!(r#"(?:x*?)*"#, "", &[], "Non-greedy", 248),
    tu!(r#"(?:x*?)*"#, "x", &[], "Non-greedy", 249),
    tu!(r#"(?:x*?)*"#, "xx", &[], "Non-greedy", 250),
    tu!(r#"(?:x*?)+"#, "", &[], "Non-greedy", 251),
    tu!(r#"(?:x*?)+"#, "x", &[], "Non-greedy", 252),
    tu!(r#"(?:x*?)+"#, "xx", &[], "Non-greedy", 253),
    tu!(r#"(?:x*?)\?\?"#, "", &[], "Non-greedy", 254),
    tu!(r#"(?:x*?)\?\?"#, "x", &[], "Non-greedy", 255),
    tu!(r#"(?:x*?)\?\?"#, "xx", &[], "Non-greedy", 256),
    tu!(r#"(?:x*?)*?"#, "", &[], "Non-greedy", 257),
    tu!(r#"(?:x*?)*?"#, "x", &[], "Non-greedy", 258),
    tu!(r#"(?:x*?)*?"#, "xx", &[], "Non-greedy", 259),
    tu!(r#"(?:x*?)+?"#, "", &[], "Non-greedy", 260),
    tu!(r#"(?:x*?)+?"#, "x", &[], "Non-greedy", 261),
    tu!(r#"(?:x*?)+?"#, "xx", &[], "Non-greedy", 262),
    tu!(r#"(?:x+?)?"#, "", &[], "Non-greedy", 263),
    tu!(r#"(?:x+?)?"#, "x", &[("x", 0, 1)], "Non-greedy", 264),
    tu!(r#"(?:x+?)?"#, "xx", &[("x", 0, 1)], "Non-greedy", 265),
    tu!(r#"(?:x+?)*"#, "", &[], "Non-greedy", 266),
    tu!(r#"(?:x+?)*"#, "x", &[("x", 0, 1)], "Non-greedy", 267),
    tu!(r#"(?:x+?)*"#, "xx", &[("xx", 0, 2)], "Non-greedy", 268),
    tu!(r#"(?:x+?)+"#, "x", &[("x", 0, 1)], "Non-greedy", 269),
    tu!(r#"(?:x+?)+"#, "xx", &[("xx", 0, 2)], "Non-greedy", 270),
    tu!(r#"(?:x+?)\?\?"#, "", &[], "Non-greedy", 271),
    tu!(r#"(?:x+?)\?\?"#, "x", &[], "Non-greedy", 272),
    tu!(r#"(?:x+?)\?\?"#, "xx", &[], "Non-greedy", 273),
    tu!(r#"(?:x+?)*?"#, "", &[], "Non-greedy", 274),
    tu!(r#"(?:x+?)*?"#, "x", &[], "Non-greedy", 275),
    tu!(r#"(?:x+?)*?"#, "xx", &[], "Non-greedy", 276),
    tu!(r#"(?:x+?)+?"#, "x", &[("x", 0, 1)], "Non-greedy", 277),
    tu!(r#"(?:x+?)+?"#, "xx", &[("x", 0, 1)], "Non-greedy", 278),
    td!(r#"a|b"#, "a", &[("a", 0, 1)], 279),
    td!(r#"a|b"#, "b", &[("b", 0, 1)], 280),
    td!(r#"|a"#, "a", &[("a", 0, 1)], 281),
    td!(r#"(|a)"#, "a", &[("a", 0, 1)], 282),
    td!(r#"ab|bc"#, "ab", &[("ab", 0, 2)], 283),
    td!(r#"ab|bc"#, "bc", &[("bc", 0, 2)], 284),
    td!(r#"z(?:ab|bc)"#, "zbc", &[("zbc", 0, 3)], 285),
    td!(r#"a(?:ab|bc)c"#, "aabc", &[("aabc", 0, 4)], 286),
    td!(r#"ab|(?:ac|az)"#, "az", &[("az", 0, 2)], 287),
    td!(r#"a|b|c"#, "dc", &[("c", 1, 2)], 288),
    td!(
        r#"a|b|cd|efg|h|ijk|lmn|o|pq|rstuvwx|yz"#,
        "pqr",
        &[("pq", 0, 2)],
        289
    ),
    tu!(r#"a|^z"#, "ba", &[("a", 1, 2)], "StartLine", 290),
    tu!(r#"a|^z"#, "za", &[("z", 0, 1)], "StartLine", 291),
    tr!(r#"a|\Gz"#, "bza", &[("a", 2, 3)], 292),
    tr!(r#"a|\Gz"#, "za", &[("z", 0, 1)], 293),
    tu!(r#"a|\Az"#, "bza", &[("a", 2, 3)], "StartText", 294),
    tu!(r#"a|\Az"#, "za", &[("z", 0, 1)], "StartText", 295),
    tr!(r#"a|b\Z"#, "ba", &[("a", 1, 2)], 296),
    tr!(r#"a|b\Z"#, "b", &[("b", 0, 1)], 297),
    tu!(r#"a|b\z"#, "ba", &[("a", 1, 2)], "EndText", 298),
    tu!(r#"a|b\z"#, "b", &[("b", 0, 1)], "EndText", 299),
    td!(r#"\w|\s"#, " ", &[(" ", 0, 1)], 300),
    td!(r#"\w|%"#, "%", &[("%", 0, 1)], 301),
    td!(r#"\w|[&$]"#, "&", &[("&", 0, 1)], 302),
    td!(r#"[b-d]|[^e-z]"#, "a", &[("a", 0, 1)], 303),
    td!(r#"(?:a|[c-f])|bz"#, "dz", &[("d", 0, 1)], 304),
    td!(r#"(?:a|[c-f])|bz"#, "bz", &[("bz", 0, 2)], 305),
    tr!(r#"abc|(?=zz)..f"#, "zzf", &[("zzf", 0, 3)], 306),
    tr!(r#"abc|(?!zz)..f"#, "abf", &[("abf", 0, 3)], 307),
    tr!(r#"(?=za)..a|(?=zz)..a"#, "zza", &[("zza", 0, 3)], 308),
    tr!(r#"(?>abd|a)c"#, "abdc", &[("abdc", 0, 4)], 309),
    td!(r#"a?|b"#, "a", &[("a", 0, 1)], 310),
    td!(r#"a?|b"#, "b", &[("b", 0, 1)], 311),
    td!(r#"a?|b"#, "", &[], 312),
    td!(r#"a*|b"#, "aa", &[("aa", 0, 2)], 313),
    td!(r#"a*|b*"#, "ba", &[("b", 0, 1), ("a", 1, 2)], 314),
    td!(r#"a*|b*"#, "ab", &[("a", 0, 1), ("b", 1, 2)], 315),
    td!(r#"a+|b*"#, "", &[], 316),
    td!(r#"a+|b*"#, "bbb", &[("bbb", 0, 3)], 317),
    td!(r#"a+|b*"#, "abbb", &[("a", 0, 1), ("bbb", 1, 4)], 318),
    td!(r#"(a|b)?"#, "b", &[("b", 0, 1)], 319),
    td!(r#"(a|b)*"#, "ba", &[("ba", 0, 2)], 320),
    td!(r#"(a|b)+"#, "bab", &[("bab", 0, 3)], 321),
    td!(r#"(ab|ca)+"#, "caabbc", &[("caab", 0, 4)], 322),
    td!(r#"(ab|ca)+"#, "aabca", &[("abca", 1, 5)], 323),
    td!(r#"(ab|ca)+"#, "abzca", &[("ab", 0, 2), ("ca", 3, 5)], 324),
    td!(r#"(a|bab)+"#, "ababa", &[("ababa", 0, 5)], 325),
    td!(r#"(a|bab)+"#, "ba", &[("a", 1, 2)], 326),
    td!(r#"(a|bab)+"#, "baaaba", &[("aaa", 1, 4), ("a", 5, 6)], 327),
    td!(r#"(?:a|b)(?:a|b)"#, "ab", &[("ab", 0, 2)], 328),
    td!(r#"(?:a*|b*)(?:a*|b*)"#, "aaabbb", &[("aaabbb", 0, 6)], 329),
    td!(r#"(?:a*|b*)(?:a+|b+)"#, "aaabbb", &[("aaabbb", 0, 6)], 330),
    td!(r#"(?:a+|b+){2}"#, "aaabbb", &[("aaabbb", 0, 6)], 331),
    td!(r#"h{0,}"#, "hhhh", &[("hhhh", 0, 4)], 332),
    td!(r#"(?:a+|b+){1,2}"#, "aaabbb", &[("aaabbb", 0, 6)], 333),
    tu!(r#"^a{2,}?a$"#, "aaa", &[("aaa", 0, 3)], "StartLine", 334),
    tu!(r#"^[a-z]{2,}?$"#, "aaa", &[("aaa", 0, 3)], "StartLine", 335),
    tu!(r#"(?:a+|\Ab*)cc"#, "cc", &[("cc", 0, 2)], "StartText", 336),
    tu!(
        r#"(?:^a+|b+)*c"#,
        "aabbbabc",
        &[("bc", 6, 8)],
        "StartLine",
        337
    ),
    tu!(
        r#"(?:^a+|b+)*c"#,
        "aabbbbc",
        &[("aabbbbc", 0, 7)],
        "StartLine",
        338
    ),
    tu!(r#"a|(?i)c"#, "C", &[("C", 0, 1)], "CaseInsensitive", 339),
    tu!(r#"(?i)c|a"#, "C", &[("C", 0, 1)], "CaseInsensitive", 340),
    tu!(r#"(?i)c|a"#, "A", &[("A", 0, 1)], "CaseInsensitive", 341),
    tu!(r#"a(?i)b|c"#, "aB", &[("aB", 0, 2)], "CaseInsensitive", 342),
    tu!(r#"a(?i)b|c"#, "aC", &[("aC", 0, 2)], "CaseInsensitive", 343),
    tu!(r#"(?i:c)|a"#, "C", &[("C", 0, 1)], "CaseInsensitive", 344),
    td!(
        r#"[abc]?"#,
        "abc",
        &[("a", 0, 1), ("b", 1, 2), ("c", 2, 3)],
        345
    ),
    td!(r#"[abc]*"#, "abc", &[("abc", 0, 3)], 346),
    td!(r#"[^abc]*"#, "abc", &[], 347),
    td!(r#"a?\?"#, "aaa", &[], 348),
    td!(
        // Oniguruma: ("bab", 0, 3)
        r#"ba?\?b"#,
        "bab",
        &[],
        349
    ),
    tu!(r#"a*?"#, "aaa", &[], "Non-greedy", 350),
    tu!(r#"ba*?"#, "baa", &[("b", 0, 1)], "Non-greedy", 351),
    tu!(r#"ba*?b"#, "baab", &[("baab", 0, 4)], "Non-greedy", 352),
    tu!(r#"a+?"#, "aaa", &[("a", 0, 1)], "Non-greedy", 353),
    tu!(r#"ba+?"#, "baa", &[("ba", 0, 2)], "Non-greedy", 354),
    tu!(r#"ba+?b"#, "baab", &[("baab", 0, 4)], "Non-greedy", 355),
    td!(r#"(?:a?)?\?"#, "a", &[], 356),
    td!(r#"(?:a?\?)?"#, "a", &[], 357),
    tu!(r#"(?:a?)+?"#, "aaa", &[("a", 0, 1)], "Non-greedy", 358),
    td!(r#"(?:a+)?\?"#, "aaa", &[], 359),
    td!(
        // Oniguruma: ("aaab", 0, 4)
        r#"(?:a+)?\?b"#,
        "aaab",
        &[],
        360
    ),
    td!(r#"(?:ab)?{2}"#, "", &[], 361),
    td!(r#"(?:ab)?{2}"#, "ababa", &[("abab", 0, 4)], 362),
    td!(r#"(?:ab)*{0}"#, "ababa", &[], 363),
    td!(r#"(?:ab){3,}"#, "abababab", &[("abababab", 0, 8)], 364),
    td!(r#"(?:ab){2,4}"#, "ababab", &[("ababab", 0, 6)], 365),
    td!(r#"(?:ab){2,4}"#, "ababababab", &[("abababab", 0, 8)], 366),
    tu!(
        r#"(?:ab){2,4}?"#,
        "ababababab",
        &[("abab", 0, 4)],
        "Non-greedy",
        367
    ),
    tr!(r#"(?:ab){,}"#, "ab{,}", &[("ab{,}", 0, 5)], 368),
    tu!(
        r#"(?:abc)+?{2}"#,
        "abcabcabc",
        &[("abcabc", 0, 6)],
        "Non-greedy",
        369
    ),
    tu!(
        r#"(?:X*)(?i:xa)"#,
        "XXXa",
        &[("XXXa", 0, 4)],
        "CaseInsensitive",
        370
    ),
    td!(r#"(d+)([^abc]z)"#, "dddz", &[("dddz", 0, 4)], 371),
    td!(r#"([^abc]*)([^abc]z)"#, "dddz", &[("dddz", 0, 4)], 372),
    td!(r#"(\w+)(\wz)"#, "dddz", &[("dddz", 0, 4)], 373),
    td!(r#"((ab))"#, "ab", &[("ab", 0, 2)], 374),
    tu!(r#"(^a)"#, "a", &[("a", 0, 1)], "StartLine", 375),
    tr!(r#"(abc)(?i:\1)"#, "abcABC", &[("abcABC", 0, 6)], 376),
    td!(r#"(?:abc)|(ABC)"#, "abc", &[("abc", 0, 3)], 377),
    tr!(r#"(?:(?:\1|z)(a))+$"#, "zaaa", &[("zaaa", 0, 4)], 378),
    tr!(r#"(a)(?=\1)"#, "aa", &[("a", 0, 1)], 379),
    tr!(r#"(a)\1"#, "aa", &[("aa", 0, 2)], 380),
    tr!(r#"(a?)\1"#, "aa", &[("aa", 0, 2)], 381),
    tr!(r#"(a?\?)\1"#, "aa", &[], 382),
    tr!(r#"(a*)\1"#, "aaaaa", &[("aaaa", 0, 4)], 383),
    tr!(r#"a(b*)\1"#, "abbbb", &[("abbbb", 0, 5)], 384),
    tr!(r#"a(b*)\1"#, "ab", &[("a", 0, 1)], 385),
    tr!(
        r#"(a*)(b*)\1\2"#,
        "aaabbaaabb",
        &[("aaabbaaabb", 0, 10)],
        386
    ),
    tr!(r#"(a*)(b*)\2"#, "aaabbbb", &[("aaabbbb", 0, 7)], 387),
    tr!(
        r#"(((((((a*)b))))))c\7"#,
        "aaabcaaa",
        &[("aaabcaaa", 0, 8)],
        388
    ),
    tr!(r#"(a)(b)(c)\2\1\3"#, "abcbac", &[("abcbac", 0, 6)], 389),
    tr!(r#"([a-d])\1"#, "cc", &[("cc", 0, 2)], 390),
    tr!(r#"(\w\d\s)\1"#, "f5 f5 ", &[("f5 f5 ", 0, 6)], 391),
    tr!(r#"(who|[a-c]{3})\1"#, "whowho", &[("whowho", 0, 6)], 392),
    tr!(
        r#"...(who|[a-c]{3})\1"#,
        "abcwhowho",
        &[("abcwhowho", 0, 9)],
        393
    ),
    tr!(r#"(who|[a-c]{3})\1"#, "cbccbc", &[("cbccbc", 0, 6)], 394),
    tr!(r#"(^a)\1"#, "aa", &[("aa", 0, 2)], 395),
    tr!(r#"(a*\Z)\1"#, "a", &[("", 1, 1)], 396),
    tr!(r#".(a*\Z)\1"#, "ba", &[("a", 1, 2)], 397),
    tr!(r#"((?i:az))\1"#, "AzAz", &[("AzAz", 0, 4)], 398),
    tr!(r#"(?<=a)b"#, "ab", &[("b", 1, 2)], 399),
    tr!(r#"(?<=a|b)b"#, "bb", &[("b", 1, 2)], 400),
    tr!(r#"(?<=a|bc)b"#, "bcb", &[("b", 2, 3)], 401),
    tr!(r#"(?<=a|bc)b"#, "ab", &[("b", 1, 2)], 402),
    tr!(
        r#"(?<=a|bc||defghij|klmnopq|r)z"#,
        "rz",
        &[("z", 1, 2)],
        403
    ),
    tr!(r#"(?<=(?i:abc))d"#, "ABCd", &[("d", 3, 4)], 404),
    tr!(r#"(?<=^|b)c"#, " cbc", &[("c", 3, 4)], 405),
    tr!(r#"(?<=a|^|b)c"#, " cbc", &[("c", 3, 4)], 406),
    tr!(r#"(?<=a|(^)|b)c"#, " cbc", &[("c", 3, 4)], 407),
    tr!(r#"(?<=a|(^)|b)c"#, "cbc", &[("c", 0, 1)], 408),
    tr!(r#"(Q)(?<=a|(?(1))|b)c"#, "cQc", &[("Qc", 1, 3)], 409),
    tr!(r#"(?<=a|(?~END)|b)c"#, "ENDc", &[("c", 3, 4)], 410),
    tr!(r#"(?<!a|(?:^)|b)c"#, " cbc", &[("c", 1, 2)], 411),
    tr!(r#"(a)\g<1>"#, "aa", &[("aa", 0, 2)], 412),
    tr!(r#"(?<!a)b"#, "cb", &[("b", 1, 2)], 413),
    tr!(r#"(?<!a|bc)b"#, "bbb", &[("b", 0, 1)], 414),
    td!(r#"(?<name1>a)"#, "a", &[("a", 0, 1)], 415),
    tr!(r#"(?<name_2>ab)\g<name_2>"#, "abab", &[("abab", 0, 4)], 416),
    tr!(
        r#"(?<name_3>.zv.)\k<name_3>"#,
        "azvbazvb",
        &[("azvbazvb", 0, 8)],
        417
    ),
    tr!(
        r#"(?<=\g<ab>)|-\zEND (?<ab>XyZ)"#,
        "XyZ",
        &[("", 3, 3)],
        418
    ),
    tr!(r#"(?<n>|a\g<n>)+"#, "", &[], 419),
    tr!(r#"(?<n>|\(\g<n>\))+$"#, "()(())", &[("()(())", 0, 6)], 420),
    tr!(
        r#"\g<n>(abc|df(?<n>.YZ){2,8}){0}"#,
        "XYZ",
        &[("XYZ", 0, 3)],
        421
    ),
    tr!(r#"\A(?<n>(a\g<n>)|)\z"#, "aaaa", &[("aaaa", 0, 4)], 422),
    tr!(
        r#"(?<n>|\g<m>\g<n>)\z|\zEND (?<m>a|(b)\g<m>)"#,
        "bbbbabba",
        &[("bbbbabba", 0, 8)],
        423
    ),
    tr!(
        r#"(?<name1240>\w+\sx)a+\k<name1240>"#,
        "  fg xaaaaaaaafg x",
        &[("fg xaaaaaaaafg x", 2, 18)],
        424
    ),
    tr!(r#"(.)(((?<_>a)))\k<_>"#, "zaa", &[("zaa", 0, 3)], 425),
    tr!(
        r#"((?<name1>\d)|(?<name2>\w))(\k<name1>|\k<name2>)"#,
        "ff",
        &[("ff", 0, 2)],
        426
    ),
    tr!(r#"(?:(?<x>)|(?<x>efg))\k<x>"#, "", &[], 427),
    tr!(
        r#"(?:(?<x>abc)|(?<x>efg))\k<x>"#,
        "abcefgefg",
        &[("efgefg", 3, 9)],
        428
    ),
    tr!(r#"(?<x>x)(?<x>xx)\k<x>"#, "xxxx", &[("xxxx", 0, 4)], 429),
    tr!(r#"(?<x>x)(?<x>xx)\k<x>"#, "xxxxz", &[("xxxx", 0, 4)], 430),
    tr!(
        r#"(?:(?<n1>.)|(?<n1>..)|(?<n1>...)|(?<n1>....)|(?<n1>.....)|(?<n1>......)|(?<n1>.......)|(?<n1>........)|(?<n1>.........)|(?<n1>..........)|(?<n1>...........)|(?<n1>............)|(?<n1>.............)|(?<n1>..............))\k<n1>$"#,
        "a-pyumpyum",
        &[("pyumpyum", 2, 10)],
        431
    ),
    tr!(r#"(?<foo>a|\(\g<foo>\))"#, "a", &[("a", 0, 1)], 432),
    tr!(
        r#"(?<foo>a|\(\g<foo>\))"#,
        "((((((a))))))",
        &[("((((((a))))))", 0, 13)],
        433
    ),
    tr!(
        r#"\g<bar>|\zEND(?<bar>.*abc$)"#,
        "abcxxxabc",
        &[("abcxxxabc", 0, 9)],
        434
    ),
    tr!(r#"\g<1>|\zEND(.a.)"#, "bac", &[("bac", 0, 3)], 435),
    tr!(
        r#"\A(?:\g<pon>|\g<pan>|\zEND  (?<pan>a|c\g<pon>c)(?<pon>b|d\g<pan>d))$"#,
        "cdcbcdc",
        &[("cdcbcdc", 0, 7)],
        436
    ),
    tr!(
        r#"\A(?<n>|a\g<m>)\z|\zEND (?<m>\g<n>)"#,
        "aaaa",
        &[("aaaa", 0, 4)],
        437
    ),
    tr!(
        r#"(?<n>(a|b\g<n>c){3,5})"#,
        "baaaaca",
        &[("aaaa", 1, 5)],
        438
    ),
    tr!(
        r#"(?<n>(a|b\g<n>c){3,5})"#,
        "baaaacaaaaa",
        &[("baaaacaaaa", 0, 10)],
        439
    ),
    tr!(
        r#"(?<pare>\(([^\(\)]++|\g<pare>)*+\))"#,
        "((a))",
        &[("((a))", 0, 5)],
        440
    ),
    tr!(r#"()*\1"#, "", &[], 441),
    tr!(r#"(?:()|())*\1\2"#, "", &[], 442),
    td!(r#"(?:a*|b*)*c"#, "abadc", &[("c", 4, 5)], 443),
    td!(r#"x((.)*)*x"#, "0x1x2x3", &[("x1x2x", 1, 6)], 444),
    tr!(
        r#"x((.)*)*x(?i:\1)\Z"#,
        "0x1x2x1X2",
        &[("x1x2x1X2", 1, 9)],
        445
    ),
    tr!(r#"(?:()|()|()|()|()|())*\2\5"#, "", &[], 446),
    tr!(r#"(?:()|()|()|(x)|()|())*\2b\5"#, "b", &[("b", 0, 1)], 447),
    td!(r#"[0-9-a]"#, "-", &[("-", 0, 1)], 448),
    tr!(r#"\o{101}"#, "A", &[("A", 0, 1)], 449),
    tr!(
        r#"\A(a|b\g<1>c)\k<1+3>\z"#,
        "bbacca",
        &[("bbacca", 0, 6)],
        450
    ),
    tr!(
        r#"(?i)\A(a|b\g<1>c)\k<1+2>\z"#,
        "bBACcbac",
        &[("bBACcbac", 0, 8)],
        451
    ),
    tr!(
        r#"(?i)(?<X>aa)|(?<X>bb)\k<X>"#,
        "BBbb",
        &[("BBbb", 0, 4)],
        452
    ),
    tr!(r#"(?:\k'+1'B|(A)C)*"#, "ACAB", &[("ACAB", 0, 4)], 453),
    tr!(r#"\g<+2>(abc)(ABC){0}"#, "ABCabc", &[("ABCabc", 0, 6)], 454),
    tr!(r#"A\g'0'|B()"#, "AAAAB", &[("AAAAB", 0, 5)], 455),
    tr!(r#"(a*)(?(1))aa"#, "aaaaa", &[("aaaaa", 0, 5)], 456),
    tr!(r#"(a*)(?(-1))aa"#, "aaaaa", &[("aaaaa", 0, 5)], 457),
    tr!(
        r#"(?<name>aaa)(?('name'))aa"#,
        "aaaaa",
        &[("aaaaa", 0, 5)],
        458
    ),
    tr!(r#"(a)(?(1)aa|bb)a"#, "aaaaa", &[("aaaa", 0, 4)], 459),
    tr!(
        r#"(?:aa|())(?(<1>)aa|bb)a"#,
        "aabba",
        &[("aabba", 0, 5)],
        460
    ),
    tr!(
        r#"(?:aa|())(?('1')aa|bb|cc)a"#,
        "aacca",
        &[("aacca", 0, 5)],
        461
    ),
    tr!(r#"(a)(?(1)|)c"#, "ac", &[("ac", 0, 2)], 462),
    tr!(r#"(a)(?(1+0)b|c)d"#, "abd", &[("abd", 0, 3)], 463),
    tr!(
        r#"(?:(?'name'a)|(?'name'b))(?('name')c|d)e"#,
        "ace",
        &[("ace", 0, 3)],
        464
    ),
    tr!(
        r#"(?:(?'name'a)|(?'name'b))(?('name')c|d)e"#,
        "bce",
        &[("bce", 0, 3)],
        465
    ),
    tr!(r#"\R"#, "\r\n", &[("\r", 0, 2)], 466),
    tr!(r#"\R"#, "\r", &[("\r", 0, 1)], 467),
    tr!(r#"\R"#, "\n", &[("\n", 0, 1)], 468),
    tr!(r#"\R"#, "\x0b", &[("\x0b", 0, 1)], 469),
    // tr!(r#"\R"#, "\xc2\x85", &[("\xc2", 0, 2)], 470),
    tr!(r#"\N"#, "a", &[("a", 0, 1)], 471),
    tr!(r#"\O"#, "a", &[("a", 0, 1)], 472),
    tr!(r#"\O"#, "\n", &[("\n", 0, 1)], 473),
    tr!(r#"(?m:\O)"#, "\n", &[("\n", 0, 1)], 474),
    tr!(r#"(?-m:\O)"#, "\n", &[("\n", 0, 1)], 475),
    tr!(r#"\K"#, "a", &[], 476),
    tr!(r#"a\K"#, "a", &[("", 1, 1)], 477),
    tr!(r#"a\Kb"#, "ab", &[("b", 1, 2)], 478),
    tr!(r#"(a\Kb|ac\Kd)"#, "acd", &[("d", 2, 3)], 479),
    tr!(r#"(a\Kb|\Kac\K)*"#, "acababacab", &[("b", 9, 10)], 480),
    tr!(r#"(?:()|())*\1"#, "abc", &[], 481),
    tr!(r#"(?:()|())*\2"#, "abc", &[], 482),
    tr!(r#"(?:()|()|())*\3\1"#, "abc", &[], 483),
    tr!(r#"(|(?:a(?:\g'1')*))b|"#, "abc", &[("ab", 0, 2)], 484),
    tr!(r#"^(\"|)(.*)\1$"#, "XX", &[("XX", 0, 2)], 485),
    tu!(
        r#"(abc|def|ghi|jkl|mno|pqr|stu){0,10}?\z"#,
        "admno",
        &[("mno", 2, 5)],
        "Non-greedy",
        486
    ),
    tu!(
        r#"(abc|(def|ghi|jkl|mno|pqr){0,7}?){5}\z"#,
        "adpqrpqrpqr",
        &[("pqrpqrpqr", 2, 11)],
        "Non-greedy",
        487
    ),
    tr!(r#"(?!abc).*\z"#, "abcde", &[("bcde", 1, 5)], 488),
    td!(r#"(.{2,})?"#, "abcde", &[("abcde", 0, 5)], 489),
    td!(
        r#"((a|b|c|d|e|f|g|h|i|j|k|l|m|n)+)?"#,
        "abcde",
        &[("abcde", 0, 5)],
        490
    ),
    td!(
        r#"((a|b|c|d|e|f|g|h|i|j|k|l|m|n){3,})?"#,
        "abcde",
        &[("abcde", 0, 5)],
        491
    ),
    td!(
        r#"((?:a(?:b|c|d|e|f|g|h|i|j|k|l|m|n))+)?"#,
        "abacadae",
        &[("abacadae", 0, 8)],
        492
    ),
    tu!(
        r#"((?:a(?:b|c|d|e|f|g|h|i|j|k|l|m|n))+?)?z"#,
        "abacadaez",
        &[("abacadaez", 0, 9)],
        "Non-greedy",
        493
    ),
    tu!(
        r#"\A((a|b)\?\?)?z"#,
        "bz",
        &[("bz", 0, 2)],
        "StartText",
        494
    ),
    tr!(
        r#"((?<x>abc){0}a\g<x>d)+"#,
        "aabcd",
        &[("aabcd", 0, 5)],
        495
    ),
    tr!(r#"((?(abc)true|false))+"#, "false", &[("false", 0, 5)], 496),
    tu!(
        r#"((?i:abc)d)+"#,
        "abcdABCd",
        &[("abcdABCd", 0, 8)],
        "CaseInsensitive",
        497
    ),
    tr!(r#"((?<!abc)def)+"#, "bcdef", &[("def", 2, 5)], 498),
    tu!(r#"(\ba)+"#, "aaa", &[("a", 0, 1)], "WordBoundary", 499),
    tr!(r#"()(?<x>ab)(?(<x>)a|b)"#, "aba", &[("aba", 0, 3)], 500),
    tr!(r#"(?<=a.b)c"#, "azbc", &[("c", 3, 4)], 501),
    tr!(r#"(?<=(?(a)a|bb))z"#, "aaz", &[("z", 2, 3)], 502),
    td!(r#"[a]*\W"#, "aa@", &[("aa@", 0, 3)], 503),
    td!(r#"[a]*[b]"#, "aab", &[("aab", 0, 3)], 504),
    tr!(r#"(?<=ab(?<=ab))"#, "ab", &[("", 2, 2)], 505),
    tr!(
        r#"(?<x>a)(?<x>b)(\k<x>)+"#,
        "abbaab",
        &[("abbaab", 0, 6)],
        506
    ),
    tr!(r#"()(\1)(\2)"#, "abc", &[], 507),
    tr!(r#"((?(a)b|c))(\1)"#, "abab", &[("abab", 0, 4)], 508),
    tr!(r#"(?<x>$|b\g<x>)"#, "bbb", &[("bbb", 0, 3)], 509),
    tr!(r#"(?<x>(?(a)a|b)|c\g<x>)"#, "cccb", &[("cccb", 0, 4)], 510),
    tr!(r#"(a)(?(1)a*|b*)+"#, "aaaa", &[("aaaa", 0, 4)], 511),
    td!(r#"[[^abc]&&cde]*"#, "de", &[("de", 0, 2)], 512),
    td!(r#"(?:a?)+"#, "aa", &[("aa", 0, 2)], 513),
    tu!(r#"(?:a?)*?"#, "a", &[], "Non-greedy", 514),
    tu!(r#"(?:a*)*?"#, "a", &[], "Non-greedy", 515),
    tu!(r#"(?:a+?)*"#, "a", &[("a", 0, 1)], "Non-greedy", 516),
    tr!(r#"\h"#, "5", &[("5", 0, 1)], 517),
    tr!(r#"\H"#, "z", &[("z", 0, 1)], 518),
    tr!(r#"[\h]"#, "5", &[("5", 0, 1)], 519),
    tr!(r#"[\H]"#, "z", &[("z", 0, 1)], 520),
    tr!(r#"[\o{101}]"#, "A", &[("A", 0, 1)], 521),
    td!(r#"[\u0041]"#, "A", &[("A", 0, 1)], 522),
    tr!(r#"(?~)"#, "", &[], 523),
    tr!(r#"(?~)"#, "A", &[], 524),
    tr!(r#"(?~ab)"#, "abc", &[], 525),
    tr!(r#"(?~abc)"#, "abc", &[], 526),
    tr!(r#"(?~abc|ab)"#, "abc", &[], 527),
    tr!(r#"(?~ab|abc)"#, "abc", &[], 528),
    tr!(r#"(?~a.c)"#, "abc", &[], 529),
    tr!(r#"(?~a.c|ab)"#, "abc", &[], 530),
    tr!(r#"(?~ab|a.c)"#, "abc", &[], 531),
    tr!(r#"aaaaa(?~)"#, "aaaaaaaaaa", &[("aaaaa", 0, 5)], 532),
    tr!(r#"(?~(?:|aaa))"#, "aaa", &[], 533),
    tr!(r#"(?~aaa|)"#, "aaa", &[], 534),
    tr!(
        r#"a(?~(?~))."#,
        "abcdefghijklmnopqrstuvwxyz",
        &[("abcdefghijklmnopqrstuvwxyz", 0, 26)],
        535
    ),
    tr!(r#"/\*(?~\*/)\*/"#, "/* */ */", &[("/* */", 0, 5)], 536),
    tr!(r#"(?~\w+)zzzzz"#, "zzzzz", &[("zzzzz", 0, 5)], 537),
    tr!(r#"(?~\w*)zzzzz"#, "zzzzz", &[("zzzzz", 0, 5)], 538),
    tr!(r#"(?~A.C|B)"#, "ABC", &[], 539),
    tr!(r#"(?~XYZ|ABC)a"#, "ABCa", &[("BCa", 1, 4)], 540),
    tr!(r#"(?~XYZ|ABC)a"#, "aABCa", &[("a", 0, 1)], 541),
    tr!(
        r#"<[^>]*>(?~[<>])</[^>]*>"#,
        "<a>vvv</a>   <b>  </b>",
        &[("<a>vvv</a>", 0, 10)],
        542
    ),
    tr!(r#"(?~ab)"#, "ccc\ndab", &[("ccc\n", 0, 5)], 543),
    tr!(r#"(?m:(?~ab))"#, "ccc\ndab", &[("ccc\n", 0, 5)], 544),
    tr!(r#"(?-m:(?~ab))"#, "ccc\ndab", &[("ccc\n", 0, 5)], 545),
    tr!(
        r#"(?~abc)xyz"#,
        "xyz012345678901234567890123456789abc",
        &[("xyz", 0, 3)],
        546
    ),
    tr!(r#"(?~|78|\d*)"#, "123456789", &[("123456", 0, 6)], 547),
    tr!(
        r#"(?~|def|(?:abc|de|f){0,100})"#,
        "abcdedeabcfdefabc",
        &[("abcdedeabcf", 0, 11)],
        548
    ),
    tr!(r#"(?~|ab|.*)"#, "ccc\nddd", &[("ccc", 0, 3)], 549),
    tr!(r#"(?~|ab|\O*)"#, "ccc\ndab", &[("ccc\n", 0, 5)], 550),
    tr!(r#"(?~|ab|\O{2,10})"#, "ccc\ndab", &[("ccc\n", 0, 5)], 551),
    tr!(r#"(?~|ab|\O{1,10})"#, "ab", &[("b", 1, 2)], 552),
    tr!(r#"(?~|abc|\O{1,10})"#, "abc", &[("bc", 1, 3)], 553),
    tr!(r#"(?~|ab|\O{5,10})|abc"#, "abc", &[("abc", 0, 3)], 554),
    tr!(
        r#"(?~|ab|\O{1,10})"#,
        "cccccccccccab",
        &[("cccccccccc", 0, 10)],
        555
    ),
    tr!(r#"(?~|aaa|)"#, "aaa", &[], 556),
    tr!(r#"(?~||a*)"#, "aaaaaa", &[], 557),
    tr!(r#"(?~||a*?)"#, "aaaaaa", &[], 558),
    tr!(r#"(a)(?~|b|\1)"#, "aaaaaa", &[("aa", 0, 2)], 559),
    tr!(r#"(a)(?~|bb|(?:a\1)*)"#, "aaaaaa", &[("aaaaa", 0, 5)], 560),
    tr!(
        r#"(b|c)(?~|abac|(?:a\1)*)"#,
        "abababacabab",
        &[("bab", 1, 4)],
        561
    ),
    tr!(r#"(?~|aaaaa|a*+)"#, "aaaaa", &[], 562),
    tr!(r#"(?~|aaaaaa|a*+)b"#, "aaaaaab", &[("aaaaab", 1, 7)], 563),
    tr!(r#"(?~|abcd|(?>))"#, "zzzabcd", &[], 564),
    tr!(r#"(?~|abc|a*?)"#, "aaaabc", &[], 565),
    tr!(r#"(?~|abc)a*"#, "aaaaaabc", &[("aaaaa", 0, 5)], 566),
    tr!(
        r#"(?~|abc)a*z|aaaaaabc"#,
        "aaaaaabc",
        &[("aaaaaabc", 0, 8)],
        567
    ),
    tr!(r#"(?~|aaaaaa)a*"#, "aaaaaa", &[], 568),
    tr!(r#"(?~|abc)aaaa|aaaabc"#, "aaaabc", &[("aaaabc", 0, 6)], 569),
    tr!(
        r#"(?>(?~|abc))aaaa|aaaabc"#,
        "aaaabc",
        &[("aaaabc", 0, 6)],
        570
    ),
    tr!(r#"(?~|)a"#, "a", &[("a", 0, 1)], 571),
    tr!(r#"(?~|a)(?~|)a"#, "a", &[("a", 0, 1)], 572),
    tr!(
        r#"(?~|a).*(?~|)a"#,
        "bbbbbbbbbbbbbbbbbbbba",
        &[("bbbbbbbbbbbbbbbbbbbba", 0, 21)],
        573
    ),
    tr!(
        r#"(?~|abc).*(xyz|pqr)(?~|)abc"#,
        "aaaaxyzaaapqrabc",
        &[("aaaaxyzaaapqrabc", 0, 16)],
        574
    ),
    tr!(
        r#"(?~|abc).*(xyz|pqr)(?~|)abc"#,
        "aaaaxyzaaaabcpqrabc",
        &[("bcpqrabc", 11, 19)],
        575
    ),
    td!(r#""#, "あ", &[], 576),
    td!("あ", "あ", &[("あ", 0, 3)], 577),
    td!("うう", "うう", &[("うう", 0, 6)], 578),
    td!("あいう", "あいう", &[("あいう", 0, 9)], 579),
    td!(
        "こここここここここここここここここここここここここここここここここここ",
        "こここここここここここここここここここここここここここここここここここ",
        &[(
            "こここここここここここここここここここここここここここここここここここ",
            0,
            105
        )],
        580
    ),
    td!("あ", "いあ", &[("あ", 3, 6)], 581),
    td!("いう", "あいう", &[("いう", 3, 9)], 582),
    // td!(r#"\xca\xb8"#, "\xca\xb8", &[("\xca\xb8"#, 0, 2)], 583),
    td!(".", "あ", &[("あ", 0, 3)], 584),
    td!("..", "かき", &[("かき", 0, 6)], 585),
    // x2("\\w", "お", 0, 3); // 586
    // x2("[\\W]", "う$", 3, 4); // 587
    // x2("\\S", "そ", 0, 3); // 588
    // x2("\\S", "漢", 0, 3); // 589
    tu!(r#"\b"#, "気 ", &[], "WordBoundary", 590),
    tu!(r#"\b"#, " ほ", &[("", 1, 1)], "WordBoundary", 591),
    tu!(r#"\B"#, "せそ ", &[("", 3, 3)], "NotWordBoundary", 592),
    // x2("\\B", "う ", 4, 4); // 593
    tu!(r#"\B"#, " い", &[], "NotWordBoundary", 594),
    // x2("[たち]", "ち", 0, 3); // 595
    // x2("[う-お]", "え", 0, 3); // 596
    // x2("[\\w]", "ね", 0, 3); // 597
    // x2("[\\D]", "は", 0, 3); // 598
    // x2("[\\S]", "へ", 0, 3); // 599
    // x2("[\\w\\d]", "よ", 0, 3); // 600
    // x2("[\\w\\d]", "   よ", 3, 6); // 601
    // x2("鬼\\W車", "鬼 車", 0, 7); // 602
    // x2("あ.い.う", "ああいいう", 0, 15); // 603
    // x2(".\\wう\\W..ぞ", "えうう うぞぞ", 0, 19); // 604
    // x2("\\s\\wこここ", " ここここ", 0, 13); // 605
    td!("ああ.け", "ああけけ", &[("ああけけ", 0, 12)], 606),
    // x2(".お", "おお", 0, 6); // 607
    // x2("^あ", "あ", 0, 3); // 608
    // x2("^む$", "む", 0, 3); // 609
    // x2("^\\w$", "に", 0, 3); // 610
    // x2("^\\wかきくけこ$", "zかきくけこ", 0, 16); // 611
    // x2("^\\w...うえお$", "zあいううえお", 0, 19); // 612
    // x2("\\w\\w\\s\\Wおおお\\d", "aお  おおお4", 0, 16); // 613
    // x2("\\Aたちつ", "たちつ", 0, 9); // 614
    // x2("むめも\\Z", "むめも", 0, 9); // 615
    // x2("かきく\\z", "かきく", 0, 9); // 616
    // x2("かきく\\Z", "かきく\n", 0, 9); // 617
    // x2("\\Gぽぴ", "ぽぴ", 0, 6); // 618
    // x2("(?=せ)せ", "せ", 0, 3); // 619
    // x2("(?!う)か", "か", 0, 3); // 620
    // x2("(?i:あ)", "あ", 0, 3); // 621
    // x2("(?i:ぶべ)", "ぶべ", 0, 6); // 622
    // x2("(?m:よ.)", "よ\n", 0, 4); // 623
    // x2("(?m:.め)", "ま\nめ", 3, 7); // 624
    td!(r#"あ?"#, "", &[], 625),
    td!(r#"変?"#, "化", &[], 626),
    // x2("変?", "変", 0, 3); // 627
    td!(r#"量*"#, "", &[], 628),
    // x2("量*", "量", 0, 3); // 629
    // x2("子*", "子子子", 0, 9); // 630
    td!(
        // Oniguruma []
        r#"馬*"#,
        "鹿馬馬馬馬",
        &[("馬馬馬馬", 3, 15)],
        631
    ),
    // x2("河+", "河", 0, 3); // 632
    // x2("時+", "時時時時", 0, 12); // 633
    // x2("え+", "ええううう", 0, 6); // 634
    // x2("う+", "おうううう", 3, 15); // 635
    // x2(".?", "た", 0, 3); // 636
    // x2(".*", "ぱぴぷぺ", 0, 12); // 637
    // x2(".+", "ろ", 0, 3); // 638
    // x2(".+", "いうえか\n", 0, 12); // 639
    // x2("あ|い", "あ", 0, 3); // 640
    // x2("あ|い", "い", 0, 3); // 641
    // x2("あい|いう", "あい", 0, 6); // 642
    // x2("あい|いう", "いう", 0, 6); // 643
    // x2("を(?:かき|きく)", "をかき", 0, 9); // 644
    // x2("を(?:かき|きく)け", "をきくけ", 0, 12); // 645
    // x2("あい|(?:あう|あを)", "あを", 0, 6); // 646
    // x2("あ|い|う", "えう", 3, 6); // 647
    // x2("あ|い|うえ|おかき|く|けこさ|しすせ|そ|たち|つてとなに|ぬね", "しすせ", 0, 9); // 648
    // x2("あ|^わ", "ぶあ", 3, 6); // 649
    // x2("あ|^を", "をあ", 0, 3); // 650
    // x2("鬼|\\G車", "け車鬼", 6, 9); // 651
    // x2("鬼|\\G車", "車鬼", 0, 3); // 652
    // x2("鬼|\\A車", "b車鬼", 4, 7); // 653
    // x2("鬼|\\A車", "車", 0, 3); // 654
    // x2("鬼|車\\Z", "車鬼", 3, 6); // 655
    // x2("鬼|車\\Z", "車", 0, 3); // 656
    tr!(r#"鬼|車\Z"#, "車\n", &[("車\n", 0, 3)], 657),
    // x2("鬼|車\\z", "車鬼", 3, 6); // 658
    // x2("鬼|車\\z", "車", 0, 3); // 659
    // x2("\\w|\\s", "お", 0, 3); // 660
    td!(r#"\w|%"#, "%お", &[("%", 0, 1), ("お", 1, 4)], 661),
    // x2("\\w|[&$]", "う&", 0, 3); // 662
    // x2("[い-け]", "う", 0, 3); // 663
    // x2("[い-け]|[^か-こ]", "あ", 0, 3); // 664
    // x2("[い-け]|[^か-こ]", "か", 0, 3); // 665
    td!(r#"[^あ]"#, "\n", &[("\n", 0, 1)], 666),
    // x2("(?:あ|[う-き])|いを", "うを", 0, 3); // 667
    // x2("(?:あ|[う-き])|いを", "いを", 0, 6); // 668
    // x2("あいう|(?=けけ)..ほ", "けけほ", 0, 9); // 669
    // x2("あいう|(?!けけ)..ほ", "あいほ", 0, 9); // 670
    // x2("(?=をあ)..あ|(?=をを)..あ", "ををあ", 0, 9); // 671
    // x2("(?<=あ|いう)い", "いうい", 6, 9); // 672
    // x2("(?>あいえ|あ)う", "あいえう", 0, 12); // 673
    // x2("あ?|い", "あ", 0, 3); // 674
    td!(
        // Oniguruma []
        r#"あ?|い"#,
        "い",
        &[("い", 0, 3)],
        675
    ),
    td!(r#"あ?|い"#, "", &[], 676),
    // x2("あ*|い", "ああ", 0, 6); // 677
    td!(
        // Oniguruma []
        r#"あ*|い*"#,
        "いあ",
        &[(r#"い"#, 0, 3), (r#"あ"#, 3, 6)], // 678
        679
    ),
    // x2("あ*|い*", "あい", 0, 3); // 680
    td!(
        // Oniguruma [("aあいい", 0, 4)]
        r#"[aあ]*|い*"#,
        "aあいいい",
        &[("aあ", 0, 4), ("いいい", 4, 13)], // 681
        682
    ),
    td!(r#"あ+|い*"#, "", &[], 683),
    // x2("あ+|い*", "いいい", 0, 9); // 684
    td!(
        // Oniguruma [("あいい", 0, 3)]
        r#"あ+|い*"#,
        "あいいい",
        &[("あ", 0, 3), ("いいい", 3, 12)], // 685
        686
    ),
    td!(
        r#"あ+|い*"#,
        "aあいいい",
        &[(r#"あ"#, 1, 4), (r#"いいい"#, 4, 13)], // 687
        688
    ),
    // x2("(あ|い)?", "い", 0, 3); // 689
    // x2("(あ|い)*", "いあ", 0, 6); // 690
    // x2("(あ|い)+", "いあい", 0, 9); // 691
    // x2("(あい|うあ)+", "うああいうえ", 0, 12); // 692
    // x2("(あい|うえ)+", "うああいうえ", 6, 18); // 693
    // x2("(あい|うあ)+", "ああいうあ", 3, 15); // 694
    // x2("(あい|うあ)+", "あいをうあ", 0, 6); // 695
    // x2("(あい|うあ)+", "$$zzzzあいをうあ", 6, 12); // 696
    // x2("(あ|いあい)+", "あいあいあ", 0, 15); // 697
    // x2("(あ|いあい)+", "いあ", 3, 6); // 698
    // x2("(あ|いあい)+", "いあああいあ", 3, 12); // 699
    // x2("(?:あ|い)(?:あ|い)", "あい", 0, 6); // 700
    // x2("(?:あ*|い*)(?:あ*|い*)", "あああいいい", 0, 9); // 701
    // x2("(?:あ*|い*)(?:あ+|い+)", "あああいいい", 0, 18); // 702
    // x2("(?:あ+|い+){2}", "あああいいい", 0, 18); // 703
    // x2("(?:あ+|い+){1,2}", "あああいいい", 0, 18); // 704
    // x2("(?:あ+|\\Aい*)うう", "うう", 0, 6); // 705
    // x2("(?:^あ+|い+)*う", "ああいいいあいう", 18, 24); // 706
    // x2("(?:^あ+|い+)*う", "ああいいいいう", 0, 21); // 707
    // x2("う{0,}", "うううう", 0, 12); // 708
    // td!(r#"あ|(?i)c"#, "C", &[("C", 0, 1)], 709),
    // td!(r#"(?i)c|あ"#, "C", &[("C", 0, 1)], 710),
    // td!(r#"(?i:あ)|a"#, "a", &[("a", 0, 1)], 711),
    // td!(r#"[あいう]?"#, "あいう", &[("あいう", 0, 3)], 712),
    // x2("[あいう]*", "あいう", 0, 9); // 713
    // td!(r#"[^あいう]*"#, "あいう", &[], 714),
    // td!(r#"あ?\?"#, "あああ", &[], 715),
    // x2("いあ?\?い", "いあい", 0, 9); // 716
    // td!(r#"あ*?"#, "あああ", &[], 717),
    // td!(r#"いあ*?"#, "いああ", &[("いああ", 0, 3)], 718),
    // x2("いあ*?い", "いああい", 0, 12); // 719
    // td!(r#"あ+?"#, "あああ", &[("あああ", 0, 3)], 720),
    // x2("いあ+?", "いああ", 0, 6); // 721
    // x2("いあ+?い", "いああい", 0, 12); // 722
    // td!(r#"(?:天?)?\?"#, "天", &[], 723),
    // td!(r#"(?:天?\?)?"#, "天", &[], 724),
    // td!(r#"(?:夢?)+?"#, "夢夢夢", &[("夢夢夢", 0, 3)], 725),
    // td!(r#"(?:風+)?\?"#, "風風風", &[], 726),
    // x2("(?:雪+)?\?霜", "雪雪雪霜", 0, 12); // 727
    // td!(r#"(?:あい)?{2}"#, "", &[], 728),
    // x2("(?:鬼車)?{2}", "鬼車鬼車鬼", 0, 12); // 729
    // td!(r#"(?:鬼車)*{0}"#, "鬼車鬼車鬼", &[], 730),
    // x2("(?:鬼車){3,}", "鬼車鬼車鬼車鬼車", 0, 24); // 731
    // x2("(?:鬼車){2,4}", "鬼車鬼車鬼車", 0, 18); // 732
    // x2("(?:鬼車){2,4}", "鬼車鬼車鬼車鬼車鬼車", 0, 24); // 733
    // x2("(?:鬼車){2,4}?", "鬼車鬼車鬼車鬼車鬼車", 0, 12); // 734
    // x2("(?:鬼車){,}", "鬼車{,}", 0, 9); // 735
    // x2("(?:かきく)+?{2}", "かきくかきくかきく", 0, 18); // 736
    // x2("((時間))", "時間", 0, 6); // 737
    // x2("(^あ)", "あ", 0, 3); // 738
    // x2("(無)\\1", "無無", 0, 6); // 739
    // x2("(空?)\\1", "空空", 0, 6); // 740
    // td!(r#"(空?\?)\1"#, "空空", &[], 741),
    // x2("(空*)\\1", "空空空空空", 0, 12); // 742
    // x2("あ(い*)\\1", "あいいいい", 0, 15); // 743
    // x2("あ(い*)\\1", "あい", 0, 3); // 744
    // x2("(あ*)(い*)\\1\\2", "あああいいあああいい", 0, 30); // 745
    // x2("(あ*)(い*)\\2", "あああいいいい", 0, 21); // 746
    // x2("(((((((ぽ*)ぺ))))))ぴ\\7", "ぽぽぽぺぴぽぽぽ", 0, 24); // 747
    // x2("(は)(ひ)(ふ)\\2\\1\\3", "はひふひはふ", 0, 18); // 748
    // x2("([き-け])\\1", "くく", 0, 6); // 749
    // x2("(\\w\\d\\s)\\1", "あ5 あ5 ", 0, 10); // 750
    // x2("(誰？|[あ-う]{3})\\1", "誰？誰？", 0, 12); // 751
    // x2("...(誰？|[あ-う]{3})\\1", "あaあ誰？誰？", 0, 19); // 752
    // x2("(誰？|[あ-う]{3})\\1", "ういうういう", 0, 18); // 753
    // x2("(^こ)\\1", "ここ", 0, 6); // 754
    // x2("(あ*\\Z)\\1", "あ", 3, 3); // 755
    // x2(".(あ*\\Z)\\1", "いあ", 3, 6); // 756
    // x2("((?i:あvず))\\1", "あvずあvず", 0, 14); // 757
    // x2("(?<愚か>変|\\(\\g<愚か>\\))", "((((((変))))))", 0, 15); // 758
    // x2("\\A(?:\\g<阿_1>|\\g<云_2>|\\z終了  (?<阿_1>観|自\\g<云_2>自)(?<云_2>在|菩薩\\g<阿_1>菩薩))$", "菩薩自菩薩自在自菩薩自菩薩", 0, 39); // 759
    // x2("[[ひふ]]", "ふ", 0, 3); // 760
    // x2("[[いおう]か]", "か", 0, 3); // 761
    // x2("[^[^あ]]", "あ", 0, 3); // 762
    // x2("[[かきく]&&きく]", "く", 0, 3); // 763
    // x2("[あ-ん&&い-を&&う-ゑ]", "ゑ", 0, 3); // 764
    // x2("[[^あ&&あ]&&あ-ん]", "い", 0, 3); // 765
    // x2("[[^あ-ん&&いうえお]&&[^う-か]]", "き", 0, 3); // 766
    // x2("[^[^あいう]&&[^うえお]]", "う", 0, 3); // 767
    // x2("[^[^あいう]&&[^うえお]]", "え", 0, 3); // 768
    // td!(r#"[あ-&&-あ]"#, "-", &[("-", 0, 1)], 769),
    // x2("[^[^a-zあいう]&&[^bcdefgうえお]q-w]", "え", 0, 3); // 770
    // td!(r#"[^[^a-zあいう]&&[^bcdefgうえお]g-w]"#, "f", &[("f", 0, 1)], 771),
    // td!(r#"[^[^a-zあいう]&&[^bcdefgうえお]g-w]"#, "g", &[("g", 0, 1)], 772),
    // x2("a<b>バージョンのダウンロード<\\/b>", "a<b>バージョンのダウンロード</b>", 0, 44); // 773
    // x2(".<b>バージョンのダウンロード<\\/b>", "a<b>バージョンのダウンロード</b>", 0, 44); // 774
    // x2("\\n?\\z", "こんにちは", 15, 15); // 775
    // x2("(?m).*", "青赤黄", 0, 9); // 776
    // x2("(?m).*a", "青赤黄a", 0, 10); // 777
    // x2("\\p{Hiragana}", "ぴ", 0, 3); // 778
    // td!(r#"\p{Emoji}"#, "\xE2\xAD\x90", &[("\xE", 0, 3)], 779),
    // td!(r#"\p{^Emoji}"#, "\xEF\xBC\x93", &[("\xE", 0, 3)], 780),
    // td!(r#"\p{Extended_Pictographic}"#, "\xE2\x9A\xA1", &[("\xE", 0, 3)], 781),
    // x2("\\p{Word}", "こ", 0, 3); // 782
    // x2("[\\p{Word}]", "こ", 0, 3); // 783
    // x2("[^\\p{^Word}]", "こ", 0, 3); // 784
    // x2("[^\\p{^Word}&&\\p{ASCII}]", "こ", 0, 3); // 785
    tu!(
        r#"[^\p{^Word}&&\p{ASCII}]"#,
        "a",
        &[("a", 0, 1)],
        "named class",
        786
    ),
    // x2("[^[\\p{^Word}]&&[\\p{ASCII}]]", "こ", 0, 3); // 787
    // x2("[^[\\p{ASCII}]&&[^\\p{Word}]]", "こ", 0, 3); // 788
    // x2("[^[\\p{^Word}]&&[^\\p{ASCII}]]", "こ", 0, 3); // 789
    // x2("[^\\x{104a}]", "こ", 0, 3); // 790
    // x2("[^\\p{^Word}&&[^\\x{104a}]]", "こ", 0, 3); // 791
    // x2("[^[\\p{^Word}]&&[^\\x{104a}]]", "こ", 0, 3); // 792
    // x2("\\p{^Cntrl}", "こ", 0, 3); // 793
    // x2("[\\p{^Cntrl}]", "こ", 0, 3); // 794
    // x2("[^\\p{Cntrl}]", "こ", 0, 3); // 795
    // x2("[^\\p{Cntrl}&&\\p{ASCII}]", "こ", 0, 3); // 796
    tu!(
        r#"[^\p{Cntrl}&&\p{ASCII}]"#,
        "a",
        &[("a", 0, 1)],
        "named class",
        797
    ),
    // x2("[^[\\p{^Cntrl}]&&[\\p{ASCII}]]", "こ", 0, 3); // 798
    // x2("[^[\\p{ASCII}]&&[^\\p{Cntrl}]]", "こ", 0, 3); // 799
    // x2("(?-W:\\p{Word})", "こ", 0, 3); // 800
    tr!(r#"(?W:\p{Word})"#, "k", &[("k", 0, 1)], 801),
    // x2("(?-W:[[:word:]])", "こ", 0, 3); // 802
    // x2("(?-D:\\p{Digit})", "３", 0, 3); // 803
    // td!(r#"(?-S:\p{Space})"#, "\xc2\x85", &[("\x", 0, 2)], 804),
    // x2("(?-P:\\p{Word})", "こ", 0, 3); // 805
    // x2("(?-W:\\w)", "こ", 0, 3); // 806
    tr!(r#"(?-W:\w)"#, "k", &[("k", 0, 1)], 807),
    tr!(r#"(?W:\w)"#, "k", &[("k", 0, 1)], 808),
    // x2("(?W:\\W)", "こ", 0, 3); // 809
    tr!(r#"(?-W:\b)"#, "こ", &[], 810),
    tr!(r#"(?-W:\b)"#, "h", &[], 811),
    tr!(r#"(?W:\b)"#, "h", &[], 812),
    tr!(r#"(?W:\B)"#, "こ", &[], 813),
    tr!(r#"(?-P:\b)"#, "こ", &[], 814),
    tr!(r#"(?-P:\b)"#, "h", &[], 815),
    tr!(r#"(?P:\b)"#, "h", &[], 816),
    tr!(r#"(?P:\B)"#, "こ", &[], 817),
    tu!(
        r#"\p{InBasicLatin}"#,
        "\x41",
        &[("\x41", 0, 1)],
        "named class",
        818
    ),
    // td!(r#".\Y\O"#, "\x0d\x0a", &[("\x", 0, 2)], 819),
    // td!(r#".\Y."#, "\x67\xCC\x88", &[("\x6", 0, 3)], 820),
    // td!(r#"\y.\Y.\y"#, "\x67\xCC\x88", &[("\x6", 0, 3)], 821),
    // td!(r#"\y.\y"#, "\xEA\xB0\x81", &[("\xE", 0, 3)], 822),
    // td!(r#"^.\Y.\Y.$"#, "\xE1\x84\x80\xE1\x85\xA1\xE1\x86\xA8", &[("\xE1\x84\", 0, 9)], 823),
    // td!(r#".\Y."#, "\xE0\xAE\xA8\xE0\xAE\xBF", &[("\xE0\x", 0, 6)], 824),
    // td!(r#".\Y."#, "\xE0\xB8\x81\xE0\xB8\xB3", &[("\xE0\x", 0, 6)], 825),
    // td!(r#".\Y."#, "\xE0\xA4\xB7\xE0\xA4\xBF", &[("\xE0\x", 0, 6)], 826),
    // td!(r#"..\Y."#, "\xE3\x80\xB0\xE2\x80\x8D\xE2\xAD\x95", &[("\xE3\x80\", 0, 9)], 827),
    // td!(r#"...\Y."#, "\xE3\x80\xB0\xCC\x82\xE2\x80\x8D\xE2\xAD\x95", &[("\xE3\x80\xB", 0, 11)], 828),
    // td!(r#"^\X$"#, "\x0d\x0a", &[("\x", 0, 2)], 829),
    // td!(r#"^\X$"#, "\x67\xCC\x88", &[("\x6", 0, 3)], 830),
    // td!(r#"^\X$"#, "\xE1\x84\x80\xE1\x85\xA1\xE1\x86\xA8", &[("\xE1\x84\", 0, 9)], 831),
    // td!(r#"^\X$"#, "\xE0\xAE\xA8\xE0\xAE\xBF", &[("\xE0\x", 0, 6)], 832),
    // td!(r#"^\X$"#, "\xE0\xB8\x81\xE0\xB8\xB3", &[("\xE0\x", 0, 6)], 833),
    // td!(r#"^\X$"#, "\xE0\xA4\xB7\xE0\xA4\xBF", &[("\xE0\x", 0, 6)], 834),
    // td!(r#"h\Xllo"#, "ha\xCC\x80llo", &[("ha\xCC\", 0, 7)], 835),
    // td!(r#"(?y{g})\yabc\y"#, "abc", &[("abc", 0, 3)], 836),
    // td!(r#"(?y{g})\y\X\y"#, "abc", &[("a", 0, 1)], 837),
    // td!(r#"(?y{w})\yabc\y"#, "abc", &[("abc", 0, 3)], 838),
    // td!(r#"(?y{w})\X"#, "\r\n", &[("\r", 0, 2)], 839),
    // td!(r#"(?y{w})\X"#, "\x0cz", &[("\", 0, 1)], 840),
    // td!(r#"(?y{w})\X"#, "q\x0c", &[("q", 0, 1)], 841),
    // td!(r#"(?y{w})\X"#, "\xE2\x80\x8D\xE2\x9D\x87", &[("\xE2\x", 0, 6)], 842),
    // td!(r#"(?y{w})\X"#, "\x20\x20", &[("\x", 0, 2)], 843),
    // td!(r#"(?y{w})\X"#, "a\xE2\x80\x8D", &[("a\xE", 0, 4)], 844),
    // td!(r#"(?y{w})\y\X\y"#, "abc", &[("abc", 0, 3)], 845),
    // td!(r#"(?y{w})\y\X\y"#, "v\xCE\x87w", &[("v\xC", 0, 4)], 846),
    // td!(r#"(?y{w})\y\X\y"#, "\xD7\x93\x27", &[("\xD", 0, 3)], 847),
    // td!(r#"(?y{w})\y\X\y"#, "\xD7\x93\x22\xD7\x93", &[("\xD7\", 0, 5)], 848),
    // td!(r#"(?y{w})\X"#, "14 45", &[("14", 0, 2)], 849),
    // td!(r#"(?y{w})\X"#, "a14", &[("a14", 0, 3)], 850),
    // td!(r#"(?y{w})\X"#, "832e", &[("832e", 0, 4)], 851),
    // td!(r#"(?y{w})\X"#, "8\xEF\xBC\x8C\xDB\xB0", &[("8\xEF\", 0, 6)], 852),
    // x2("(?y{w})\\y\\X\\y", "ケン", 0, 6); // WB13 // 853
    // td!(r#"(?y{w})\y\X\y"#, "ケン\xE2\x80\xAFタ", &[("ケン\xE2\x80\x", 0, 12)], 854),
    // td!(r#"(?y{w})\y\X\y"#, "\x21\x23", &[("\", 0, 1)], 855),
    // x2("(?y{w})\\y\\X\\y", "山ア", 0, 3); // 856
    // td!(r#"(?y{w})\X"#, "3.14", &[("3.14", 0, 4)], 857),
    // td!(r#"(?y{w})\X"#, "3 14", &[("3", 0, 1)], 858),
    // td!(r#"\x40"#, "@", &[("@", 0, 1)], 859),
    // td!(r#"\x1"#, "\x01", &[("\", 0, 1)], 860),
    // td!(r#"\x{1}"#, "\x01", &[("\", 0, 1)], 861),
    // td!(r#"\x{4E38}"#, "\xE4\xB8\xB8", &[("\xE", 0, 3)], 862),
    // td!(r#"\u4E38"#, "\xE4\xB8\xB8", &[("\xE", 0, 3)], 863),
    td!(r#"\u0040"#, "@", &[("@", 0, 1)], 864),
    tu!(r#"c.*\b"#, "abc", &[("c", 2, 3)], "WordBoundary", 865),
    tu!(
        r#"\b.*abc.*\b"#,
        "abc",
        &[("abc", 0, 3)],
        "WordBoundary",
        866
    ),
    tr!(
        r#"((?()0+)+++(((0\g<0>)0)|())++++((?(1)(0\g<0>))++++++0*())++++((?(1)(0\g<1>)+)++++++++++*())++++((?(1)((0)\g<0>)+)++())+0++*+++(((0\g<0>))*())++++((?(1)(0\g<0>)+)++++++++++*|)++++*+++((?(1)((0)\g<0>)+)+++++++++())++*|)++++((?()0))|"#,
        "abcde",
        &[],
        867
    ),
    tr!(
        r#"(?:[ab]|(*MAX{2}).)*"#,
        "abcbaaccaaa",
        &[("abcbaac", 0, 7)],
        868
    ),
    tr!(r#"(?(?{....})123|456)"#, "123", &[("123", 0, 3)], 869),
    tr!(r#"(?(*FAIL)123|456)"#, "456", &[("456", 0, 3)], 870),
    tr!(r#"\g'0'++{,0}"#, "abcdefgh", &[], 871),
    tr!(r#"\g'0'++{,0}?"#, "abcdefgh", &[], 872),
    tr!(r#"\g'0'++{,0}b"#, "abcdefgh", &[("b", 1, 2)], 873),
    tr!(r#"\g'0'++{,0}?def"#, "abcdefgh", &[("def", 3, 6)], 874),
    tu!(r#"a{1,3}?"#, "aaa", &[("a", 0, 1)], "Non-greedy", 875),
    td!(r#"a{3}"#, "aaa", &[("aaa", 0, 3)], 876),
    tu!(r#"a{3}?"#, "aaa", &[("aaa", 0, 3)], "Non-greedy", 877),
    tu!(r#"a{3}?"#, "aa", &[], "Non-greedy", 878),
    tu!(r#"a{3,3}?"#, "aaa", &[("aaa", 0, 3)], "Non-greedy", 879),
    td!(r#"a{1,3}+"#, "aaaaaa", &[("aaaaaa", 0, 6)], 880),
    td!(r#"a{3}+"#, "aaaaaa", &[("aaaaaa", 0, 6)], 881),
    td!(r#"a{3,3}+"#, "aaaaaa", &[("aaaaaa", 0, 6)], 882),
    tr!(r#"a{3,2}b"#, "aaab", &[("aaab", 0, 4)], 883),
    tr!(r#"a{3,2}b"#, "aaaab", &[("aaab", 1, 5)], 884),
    tr!(r#"a{3,2}b"#, "aab", &[("aab", 0, 3)], 885),
    tr!(r#"a{3,2}?"#, "", &[], 886),
    td!(r#"a{2,3}+a"#, "aaa", &[("aaa", 0, 3)], 887),
    // td!(r#"[\x{0}-\x{7fffffff}]"#, "a", &[("a", 0, 1)], 888),
    // td!(r#"[\x{7f}-\x{7fffffff}]"#, "\xe5\xae\xb6", &[("\xe", 0, 3)], 889),
    // td!(r#"[a[cdef]]"#, "a", &[("a", 0, 1)], 890),
    // td!(r#"[a[xyz]-c]"#, "a", &[("a", 0, 1)], 891),
    // td!(r#"[a[xyz]-c]"#, "-", &[("-", 0, 1)], 892),
    // td!(r#"[a[xyz]-c]"#, "c", &[("c", 0, 1)], 893),
    // td!(r#"(a.c|def)(.{4})(?<=\1)"#, "abcdabc", &[("abcdabc", 0, 7)], 894),
    // td!(r#"(a.c|de)(.{4})(?<=\1)"#, "abcdabc", &[("abcdabc", 0, 7)], 895),
    // td!(r#"(a.c|def)(.{5})(?<=d\1e)"#, "abcdabce", &[("abcdabce", 0, 8)], 896),
    // td!(r#"(a.c|.)d(?<=\k<1>d)"#, "zzzzzabcdabc", &[("abcd", 5, 9)], 897),
    // td!(r#"(?<=az*)abc"#, "azzzzzzzzzzabcdabcabc", &[("abc", 11, 14)], 898),
    // td!(r#"(?<=ab|abc|abcd)ef"#, "abcdef", &[("ef", 4, 6)], 899),
    // td!(r#"(?<=ta+|tb+|tc+|td+)zz"#, "tcccccccccczz", &[("zz", 11, 13)], 900),
    // td!(r#"(?<=t.{7}|t.{5}|t.{2}|t.)zz"#, "tczz", &[("zz", 2, 4)], 901),
    // td!(r#"(?<=t.{7}|t.{5}|t.{2})zz"#, "tczzzz", &[("zz", 3, 5)], 902),
    // td!(r#"(?<=t.{7}|t.{5}|t.{3})zz"#, "tczzazzbzz", &[("zz", 8, 10)], 903),
    // td!(r#"(?<=(ab|abc|abcd))ef"#, "abcdef", &[("ef", 4, 6)], 904),
    // td!(r#"(?<=(ta+|tb+|tc+|td+))zz"#, "tcccccccccczz", &[("zz", 11, 13)], 905),
    // td!(r#"(?<=(t.{7}|t.{5}|t.{2}|t.))zz"#, "tczz", &[("zz", 2, 4)], 906),
    // td!(r#"(?<=(t.{7}|t.{5}|t.{2}))zz"#, "tczzzz", &[("zz", 3, 5)], 907),
    // td!(r#"(?<=(t.{7}|t.{5}|t.{3}))zz"#, "tczzazzbzz", &[("zz", 8, 10)], 908),
    // td!(r#"(.{1,4})(.{1,4})(?<=\2\1)"#, "abaaba", &[("abaaba", 0, 6)], 909),
    // td!(r#"(.{1,4})(.{1,4})(?<=\2\1)"#, "ababab", &[("ababab", 0, 6)], 910),
    // td!(r#"(.{1,4})(.{1,4})(?<=\2\1)"#, "abcdabceabce", &[("abceabce", 4, 12)], 911),
    // td!(r#"(?<=a)"#, "a", &[("", 1, 1)], 912),
    // td!(r#"(?<=a.*\w)z"#, "abbbz", &[("z", 4, 5)], 913),
    // td!(r#"(?<=a.*\W)z"#, "abb z", &[("z", 4, 5)], 914),
    // td!(r#"(?<=a.*\b)z"#, "abb z", &[("z", 4, 5)], 915),
    // td!(r#"(?<=(?>abc))"#, "abc", &[("", 3, 3)], 916),
    // td!(r#"(?<=a\Xz)"#, "abz", &[("", 3, 3)], 917),
    // td!(r#"(?<=a+.*[efg])z"#, "abcdfz", &[("z", 5, 6)], 918),
    // td!(r#"(?<=a+.*[efg])z"#, "abcdfgz", &[("z", 6, 7)], 919),
    // td!(r#"(?<=a*.*[efg])z"#, "bcdfz", &[("z", 4, 5)], 920),
    // td!(r#"(?<=v|t|a+.*[efg])z"#, "abcdfz", &[("z", 5, 6)], 921),
    // td!(r#"(?<=v|t|^a+.*[efg])z"#, "abcdfz", &[("z", 5, 6)], 922),
    // td!(r#"(?<=^(?:v|t|a+.*[efg]))z"#, "abcdfz", &[("z", 5, 6)], 923),
    // td!(r#"(?<=v|^t|a+.*[efg])z"#, "uabcdfz", &[("z", 6, 7)], 924),
    // td!(r#"^..(?<=(a{,2}))\1z"#, "aaz", &[("aaz", 0, 3)], 925),
    // td!(r#"(?<=(?<= )| )"#, "abcde fg", &[("", 6, 6)], 926),
    // td!(r#"(?<=D|)(?<=@!nnnnnnnnnIIIIn;{1}D?()|<x@x*xxxD|)(?<=@xxx|xxxxx\g<1>;{1}x)"#, "(?<=D|)(?<=@!nnnnnnnnnIIIIn;{1}D?()|<x@x*xxxD|)(?<=@xxx|xxxxx\\g<1>;{1}x)", &[("", 55, 55)], 927),
    // td!(r#"(?<=;()|)\g<1>"#, "", &[], 928),
    // td!(r#"(?<=;()|)\k<1>"#, ";", &[("", 1, 1)], 929),
    // td!(r#"(())\g<3>{0}(?<=|())"#, "abc", &[], 930),
    // td!(r#"(?<=()|)\1{0}"#, "abc", &[], 931),
    // td!(r#"(?<=(?<=abc))def"#, "abcdef", &[("def", 3, 6)], 932),
    // td!(r#"(?<=ab(?<=.+b)c)def"#, "abcdef", &[("def", 3, 6)], 933),
    // td!(r#"(?<!ab.)(?<=.bc)def"#, "abcdefcbcdef", &[("def", 9, 12)], 934),
    // td!(r#"(?<!x+|abc)def"#, "xxxxxxxxzdef", &[("def", 9, 12)], 935),
    // td!(r#"(?<!a.*z|a)def"#, "axxxxxxxzdefxxdef", &[("def", 14, 17)], 936),
    // td!(r#"(?<!a.*z|a)def"#, "bxxxxxxxadefxxdef", &[("def", 14, 17)], 937),
    // td!(r#"(?<!a.*z|a)def"#, "bxxxxxxxzdef", &[("def", 9, 12)], 938),
    // td!(r#"(?<!x+|y+)\d+"#, "xxx572", &[("72", 4, 6)], 939),
    // td!(r#"(?<!3+|4+)\d+"#, "33334444", &[("33334444", 0, 8)], 940),
    // td!(r#"(.{,3})..(?<!\1)"#, "abcde", &[("abcde", 0, 5)], 941),
    // td!(r#"(.{,3})...(?<!\1)"#, "abcde", &[("abcde", 0, 5)], 942),
    // td!(r#"(a.c)(.{3,}?)(?<!\1)"#, "abcabcd", &[("abcabcd", 0, 7)], 943),
    // td!(r#"(a*)(.{3,}?)(?<!\1)"#, "abcabcd", &[("abcab", 0, 5)], 944),
    // td!(r#"(?:(a.*b)|c.*d)(?<!(?(1))azzzb)"#, "azzzzb", &[("azzzzb", 0, 6)], 945),
    // td!(r#"<(?<!NT{+}abcd)"#, "<(?<!NT{+}abcd)", &[("<", 0, 1)], 946),
    // td!(r#"(?<!a.*c)def"#, "abbbbdef", &[("def", 5, 8)], 947),
    // td!(r#"(?<!a.*X\b)def"#, "abbbbbXdef", &[("def", 7, 10)], 948),
    // td!(r#"(?<!a.*[uvw])def"#, "abbbbbXdef", &[("def", 7, 10)], 949),
    // td!(r#"(?<!ab*\S+)def"#, "abbbbb   def", &[("def", 9, 12)], 950),
    // td!(r#"(?<!a.*\S)def"#, "abbbbb def", &[("def", 7, 10)], 951),
    // td!(r#"(?<!ab*\s+\B)def"#, "abbbbb   def", &[("def", 9, 12)], 952),
    // td!(r#"(?<!v|t|a+.*[efg])z"#, "abcdfzavzuz", &[("z", 10, 11)], 953),
    // td!(r#"(?<!v|^t|^a+.*[efg])z"#, "uabcdfz", &[("z", 6, 7)], 954),
    // td!(r#"(a|\k<2>)|(?<=(\k<1>))"#, "a", &[("a", 0, 1)], 955),
    // td!(r#"(a|\k<2>)|(?<=b(\k<1>))"#, "ba", &[("a", 1, 2)], 956),
    // td!(r#"(?<=RMA)X"#, "123RMAX", &[("X", 6, 7)], 957),
    // td!(r#"(?<=RMA)$"#, "123RMA", &[("", 6, 6)], 958),
    // td!(r#"(?<=RMA)\Z"#, "123RMA", &[("", 6, 6)], 959),
    // td!(r#"(?<=RMA)\z"#, "123RMA", &[("", 6, 6)], 960),
    // td!(r#"((?(a)\g<1>|b))"#, "aab", &[("aab", 0, 3)], 961),
    // td!(r#"((?(a)\g<1>))"#, "aab", &[("aa", 0, 2)], 962),
    // td!(r#"((?(a)\g<1>))"#, "", &[], 963),
    // td!(r#"(b(?(a)|\g<1>))"#, "bba", &[("bba", 0, 3)], 964),
    // td!(r#"(?(a)(?:b|c))"#, "ac", &[("ac", 0, 2)], 965),
    // td!(r#"(?(a)(?:b|c))"#, "", &[], 966),
    // td!(r#"(?(a)b)"#, "", &[], 967),
    // td!(r#"(?i)a|b"#, "B", &[("B", 0, 1)], 968),
    // td!(r#"c(?i)a|b"#, "cB", &[("cB", 0, 2)], 969),
    // td!(r#"c(?i)a.|b."#, "cBb", &[("cBb", 0, 3)], 970),
    // td!(r#"(?i)st"#, "st", &[("st", 0, 2)], 971),
    // td!(r#"(?i)st"#, "St", &[("St", 0, 2)], 972),
    // td!(r#"(?i)st"#, "sT", &[("sT", 0, 2)], 973),
    // td!(r#"(?i)st"#, "\xC5\xBFt", &[("\xC", 0, 3)], 974),
    // td!(r#"(?i)st"#, "\xEF\xAC\x85", &[("\xE", 0, 3)], 975),
    // td!(r#"(?i)st"#, "\xEF\xAC\x86", &[("\xE", 0, 3)], 976),
    // td!(r#"(?i)ast"#, "Ast", &[("Ast", 0, 3)], 977),
    // td!(r#"(?i)ast"#, "ASt", &[("ASt", 0, 3)], 978),
    // td!(r#"(?i)ast"#, "AsT", &[("AsT", 0, 3)], 979),
    // td!(r#"(?i)ast"#, "A\xC5\xBFt", &[("A\xC", 0, 4)], 980),
    // td!(r#"(?i)ast"#, "A\xEF\xAC\x85", &[("A\xE", 0, 4)], 981),
    // td!(r#"(?i)ast"#, "A\xEF\xAC\x86", &[("A\xE", 0, 4)], 982),
    // td!(r#"(?i)stZ"#, "stz", &[("stz", 0, 3)], 983),
    // td!(r#"(?i)stZ"#, "Stz", &[("Stz", 0, 3)], 984),
    // td!(r#"(?i)stZ"#, "sTz", &[("sTz", 0, 3)], 985),
    // td!(r#"(?i)stZ"#, "\xC5\xBFtz", &[("\xC5", 0, 4)], 986),
    // td!(r#"(?i)stZ"#, "\xEF\xAC\x85z", &[("\xEF", 0, 4)], 987),
    // td!(r#"(?i)stZ"#, "\xEF\xAC\x86z", &[("\xEF", 0, 4)], 988),
    // td!(r#"(?i)BstZ"#, "bstz", &[("bstz", 0, 4)], 989),
    // td!(r#"(?i)BstZ"#, "bStz", &[("bStz", 0, 4)], 990),
    // td!(r#"(?i)BstZ"#, "bsTz", &[("bsTz", 0, 4)], 991),
    // td!(r#"(?i)BstZ"#, "b\xC5\xBFtz", &[("b\xC5", 0, 5)], 992),
    // td!(r#"(?i)BstZ"#, "b\xEF\xAC\x85z", &[("b\xEF", 0, 5)], 993),
    // td!(r#"(?i)BstZ"#, "b\xEF\xAC\x86z", &[("b\xEF", 0, 5)], 994),
    // td!(r#"(?i).*st\z"#, "tttssss\xC5\xBFt", &[("tttssss\xC", 0, 10)], 995),
    // td!(r#"(?i).*st\z"#, "tttssss\xEF\xAC\x85", &[("tttssss\xE", 0, 10)], 996),
    // td!(r#"(?i).*st\z"#, "tttssss\xEF\xAC\x86", &[("tttssss\xE", 0, 10)], 997),
    // td!(r#"(?i).*あstい\z"#, "tttssssあ\xC5\xBFtい", &[("tttssssあ\xC5\xBF", 0, 16)], 998),
    // td!(r#"(?i).*あstい\z"#, "tttssssあ\xEF\xAC\x85い", &[("tttssssあ\xEF\xAC", 0, 16)], 999),
    // td!(r#"(?i).*あstい\z"#, "tttssssあ\xEF\xAC\x86い", &[("tttssssあ\xEF\xAC", 0, 16)], 1000),
    // td!(r#"(?i).*\xC5\xBFt\z"#, "tttssssst", &[("tttssssst", 0, 9)], 1001),
    // x2("(?i).*\xEF\xAC\x85\\z", "tttssssあst", 0, 12); // U+FB05 // 1002
    // x2("(?i).*\xEF\xAC\x86い\\z", "tttssssstい", 0, 12); // U+FB06 // 1003
    // td!(r#"(?i).*\xEF\xAC\x85\z"#, "tttssssあ\xEF\xAC\x85", &[("tttssssあ\xEF\", 0, 13)], 1004),
    // td!(r#"(?i).*ss"#, "abcdefghijklmnopqrstuvwxyz\xc3\x9f", &[("abcdefghijklmnopqrstuvwxyz\x", 0, 28)], 1005),
    // td!(r#"(?i).*ss.*"#, "abcdefghijklmnopqrstuvwxyz\xc3\x9fxyz", &[("abcdefghijklmnopqrstuvwxyz\xc3\", 0, 31)], 1006),
    // td!(r#"(?i).*\xc3\x9f"#, "abcdefghijklmnopqrstuvwxyzss", &[("abcdefghijklmnopqrstuvwxyzss", 0, 28)], 1007),
    // td!(r#"(?i).*ss.*"#, "abcdefghijklmnopqrstuvwxyzSSxyz", &[("abcdefghijklmnopqrstuvwxyzSSxyz", 0, 31)], 1008),
    // td!(r#"(?i)ssv"#, "\xc3\x9fv", &[("\xc", 0, 3)], 1009),
    // td!(r#"(?i)(?<=ss)v"#, "SSv", &[("v", 2, 3)], 1010),
    // td!(r#"(?i)(?<=\xc3\x9f)v"#, "\xc3\x9fv", &[("c", 2, 3)], 1011),
    // x2("(?i).+Isssǰ", ".+Isssǰ", 0, 8); // 1012
    // x2(".+Isssǰ", ".+Isssǰ", 0, 8); // 1013
    // x2("(?i)ǰ", "ǰ", 0, 2); // 1014
    // td!(r#"(?i)ǰ"#, "j\xcc\x8c", &[("j\x", 0, 3)], 1015),
    // x2("(?i)j\xcc\x8c", "ǰ", 0, 2); // 1016
    // x2("(?i)5ǰ", "5ǰ", 0, 3); // 1017
    // td!(r#"(?i)5ǰ"#, "5j\xcc\x8c", &[("5j\x", 0, 4)], 1018),
    // x2("(?i)5j\xcc\x8c", "5ǰ", 0, 3); // 1019
    // x2("(?i)ǰv", "ǰV", 0, 3); // 1020
    // td!(r#"(?i)ǰv"#, "j\xcc\x8cV", &[("j\xc", 0, 4)], 1021),
    // x2("(?i)j\xcc\x8cv", "ǰV", 0, 3); // 1022
    // x2("(?i)[ǰ]", "ǰ", 0, 2); // 1023
    // td!(r#"(?i)[ǰ]"#, "j\xcc\x8c", &[("j\x", 0, 3)], 1024),
    // td!(r#"(?i)\ufb00a"#, "ffa", &[("ffa", 0, 3)], 1025),
    // td!(r#"(?i)ffz"#, "\xef\xac\x80z", &[("\xef", 0, 4)], 1026),
    // td!(r#"(?i)\u2126"#, "\xcf\x89", &[("\x", 0, 2)], 1027),
    // td!(r#"a(?i)\u2126"#, "a\xcf\x89", &[("a\x", 0, 3)], 1028),
    // td!(r#"(?i)A\u2126"#, "a\xcf\x89", &[("a\x", 0, 3)], 1029),
    // td!(r#"(?i)A\u2126="#, "a\xcf\x89=", &[("a\xc", 0, 4)], 1030),
    // td!(r#"(?i:ss)=1234567890"#, "\xc5\xbf\xc5\xbf=1234567890", &[("\xc5\xbf\xc5\xb", 0, 15)], 1031),
    // td!(r#"\x{000A}"#, "\x0a", &[("\", 0, 1)], 1032),
    // td!(r#"\x{000A 002f}"#, "\x0a\x2f", &[("\x", 0, 2)], 1033),
    // td!(r#"\x{000A 002f }"#, "\x0a\x2f", &[("\x", 0, 2)], 1034),
    // td!(r#"\x{007C     001b}"#, "\x7c\x1b", &[("\x", 0, 2)], 1035),
    // td!(r#"\x{1 2 3 4 5 6 7 8 9 a b c d e f}"#, "\x01\x02\x3\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f", &[("\x01\x02\x3\x04", 0, 15)], 1036),
    // td!(r#"a\x{000A 002f}@"#, "a\x0a\x2f@", &[("a\x0", 0, 4)], 1037),
    // td!(r#"a\x{0060\n0063}@"#, "a\x60\x63@", &[("a\x6", 0, 4)], 1038),
    // td!(r#"\o{102}"#, "B", &[("B", 0, 1)], 1039),
    // td!(r#"\o{102 103}"#, "BC", &[("BC", 0, 2)], 1040),
    // td!(r#"\o{0160 0000161}"#, "pq", &[("pq", 0, 2)], 1041),
    // td!(r#"\o{1 2 3 4 5 6 7 10 11 12 13 14 15 16 17}"#, "\x01\x02\x3\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f", &[("\x01\x02\x3\x04", 0, 15)], 1042),
    // td!(r#"\o{0007 0010 }"#, "\x07\x08", &[("\x", 0, 2)], 1043),
    // td!(r#"[\x{000A}]"#, "\x0a", &[("\", 0, 1)], 1044),
    // td!(r#"[\x{000A 002f}]+"#, "\x0a\x2f\x2e", &[("\x", 0, 2)], 1045),
    // td!(r#"[\x{01 0F 1A 2c 4B}]+"#, "\x20\x01\x0f\x1a\x2c\x4b\x1b", &[("x20\x", 1, 6)], 1046),
    // td!(r#"[\x{0020 0024}-\x{0026}]+"#, "\x25\x24\x26\x23", &[("\x2", 0, 3)], 1047),
    // td!(r#"[\x{0030}-\x{0033 005a}]+"#, "\x30\x31\x32\x33\x5a\34", &[("\x30\", 0, 5)], 1048),
    // td!(r#"[\o{102}]"#, "B", &[("B", 0, 1)], 1049),
    // td!(r#"[\o{102 103}]*"#, "BC", &[("BC", 0, 2)], 1050),
    // td!(r#"[\x{0030-0039}]+"#, "abc0123456789def", &[("0123456789", 3, 13)], 1051),
    // td!(r#"[\x{0030 - 0039 }]+"#, "abc0123456789def", &[("0123456789", 3, 13)], 1052),
    // td!(r#"[\x{0030 - 0039 0063 0064}]+"#, "abc0123456789def", &[("c0123456789d", 2, 14)], 1053),
    // td!(r#"[\x{0030 - 0039 0063-0065}]+"#, "acde019b", &[("cde019", 1, 7)], 1054),
    // td!(r#"[a-\x{0063 0071}]+"#, "dabcqz", &[("abcq", 1, 5)], 1055),
    // td!(r#"[-\x{0063-0065}]+"#, "ace-df", &[("ce-d", 1, 5)], 1056),
    // td!(r#"[\x61-\x{0063 0065}]+"#, "abced", &[("abce", 0, 4)], 1057),
    // td!(r#"[t\x{0063 0071}]+"#, "tcqb", &[("tcq", 0, 3)], 1058),
    // td!(r#"[\W\x{0063 0071}]+"#, "*cqa", &[("*cq", 0, 3)], 1059),
    // td!(r#"(\O|(?=z\g<2>*))(\g<0>){0}"#, "a", &[("a", 0, 1)], 1060),
    // td!(r#"(?Ii)abc"#, "abc", &[("abc", 0, 3)], 1061),
    // td!(r#"(?Ii)abc"#, "ABC", &[("ABC", 0, 3)], 1062),
    // td!(r#"(?Ii:abc)"#, "abc", &[("abc", 0, 3)], 1063),
    // td!(r#"(?Ii)xyz|abc"#, "aBc", &[("aBc", 0, 3)], 1064),
    // td!(r#"(?Ii:zz|abc|AZ)"#, "ABc", &[("ABc", 0, 3)], 1065),
    // td!(r#"(?I-i:abc)"#, "abc", &[("abc", 0, 3)], 1066),
    // td!(r#"(?i)\xe2\x84\xaa"#, "k", &[("k", 0, 1)], 1067),
    // td!(r#"(?:(?Ii)abc)"#, "ABC", &[("ABC", 0, 3)], 1068),
    // td!(r#"(?:(?:(?Ii)abc))"#, "ABC", &[("ABC", 0, 3)], 1069),
    // td!(r#"(?Ii)$"#, "", &[], 1070),
    // td!(r#"(?Ii)|"#, "", &[], 1071),
    td!(r#"a*"#, "aabcaaa", &[("aa", 0, 2), ("aaa", 4, 7)], 1072),
    // td!(r#"(?L)a*"#, "aabcaaa", &[("aaa", 4, 7)], 1073),
    // td!(r#"(?L)a{4}|a{3}|b*"#, "baaaaabbb", &[("aaaa", 1, 5)], 1074),
    // td!(r#"(?L)a{3}|a{4}|b*"#, "baaaaabbb", &[("aaaa", 1, 5)], 1075),
    // td!(r#"(?L)z|a\g<0>a"#, "aazaa", &[("aazaa", 0, 5)], 1076),
    // td!(r#"(?Li)z|a\g<0>a"#, "aazAA", &[("aazAA", 0, 5)], 1077),
    // td!(r#"(?Li:z|a\g<0>a)"#, "aazAA", &[("aazAA", 0, 5)], 1078),
    // td!(r#"(?L)z|a\g<0>a"#, "aazaaaazaaaa", &[("aaaazaaaa", 3, 12)], 1079),
    // td!(r#"(?iI)(?:[[:word:]])"#, "\xc5\xbf", &[("\x", 0, 2)], 1080),
    // td!(r#"(?iW:[[:word:]])"#, "\xc5\xbf", &[("\x", 0, 2)], 1081),
    // td!(r#"(?iW:[\p{Word}])"#, "\xc5\xbf", &[("\x", 0, 2)], 1082),
    // td!(r#"(?iW:[\w])"#, "\xc5\xbf", &[("\x", 0, 2)], 1083),
    // td!(r#"(?i)\p{Word}"#, "\xc5\xbf", &[("\x", 0, 2)], 1084),
    // td!(r#"(?i)\w"#, "\xc5\xbf", &[("\x", 0, 2)], 1085),
    // td!(r#"(?iW:[[:^word:]])"#, "\xc5\xbf", &[("\x", 0, 2)], 1086),
    // td!(r#"(?iW:[\P{Word}])"#, "\xc5\xbf", &[("\x", 0, 2)], 1087),
    // td!(r#"(?iW:[\W])"#, "\xc5\xbf", &[("\x", 0, 2)], 1088),
    // td!(r#"(?iW:\P{Word})"#, "\xc5\xbf", &[("\x", 0, 2)], 1089),
    // td!(r#"(?iW:\W)"#, "\xc5\xbf", &[("\x", 0, 2)], 1090),
    // td!(r#"(?iW:[[:^word:]])"#, "s", &[("s", 0, 1)], 1091),
    // td!(r#"(?iW:[\P{Word}])"#, "s", &[("s", 0, 1)], 1092),
    // td!(r#"(?iW:[\W])"#, "s", &[("s", 0, 1)], 1093),
    td!(r#"[[:punct:]]"#, ":", &[(":", 0, 1)], 1094),
    td!(r#"[[:punct:]]"#, "$", &[("$", 0, 1)], 1095),
    td!(r#"[[:punct:]]+"#, "$+<=>^`|~", &[("$+<=>^`|~", 0, 9)], 1096),
    // x2("\\p{PosixPunct}+", "$¦", 0, 3); // 1097
    // td!(r#"\A.*\R"#, "\n", &[("\", 0, 1)], 1098),
    // td!(r#"\A\O*\R"#, "\n", &[("\", 0, 1)], 1099),
    // td!(r#"\A\n*\R"#, "\n", &[("\", 0, 1)], 1100),
    // td!(r#"\A\R*\R"#, "\n", &[("\", 0, 1)], 1101),
    // td!(r#"\At*\R"#, "\n", &[("\", 0, 1)], 1102),
    // td!(r#"\A.{0,99}\R"#, "\n", &[("\", 0, 1)], 1103),
    // td!(r#"\A\O{0,99}\R"#, "\n", &[("\", 0, 1)], 1104),
    // td!(r#"\A\n{0,99}\R"#, "\n", &[("\", 0, 1)], 1105),
    // td!(r#"\A\R{0,99}\R"#, "\n", &[("\", 0, 1)], 1106),
    // td!(r#"\At{0,99}\R"#, "\n", &[("\", 0, 1)], 1107),
    // td!(r#"\A.*\n"#, "\n", &[("\", 0, 1)], 1108),
    // td!(r#"\A.{0,99}\n"#, "\n", &[("\", 0, 1)], 1109),
    // td!(r#"\A.*\O"#, "\n", &[("\", 0, 1)], 1110),
    // td!(r#"\A.{0,99}\O"#, "\n", &[("\", 0, 1)], 1111),
    // td!(r#"\A.*\s"#, "\n", &[("\", 0, 1)], 1112),
    // td!(r#"\A.{0,99}\s"#, "\n", &[("\", 0, 1)], 1113),
    // td!(r#"000||0\xfa"#, "0", &[], 1114),
    td!(
        /* Issue #221 */
        "aaaaaaaaaaaaaaaaaaaaaaaあb",
        "aaaaaaaaaaaaaaaaaaaaaaaあb",
        &[("aaaaaaaaaaaaaaaaaaaaaaaあb", 0, 27)],
        1115
    ),
    // td!(r#"\p{Common}"#, "\xe3\x8b\xbf", &[("\xe", 0, 3)], 1116),
    // td!(r#"\p{In_Enclosed_CJK_Letters_and_Months}"#, "\xe3\x8b\xbf", &[("\xe", 0, 3)], 1117),
    // td!(r#"(?:)*"#, "abc", &[], 1118),
];

#[test]
fn match_test() {
    for test_data in TEST_DATA.iter() {
        // Create a scanner from the scanner builder with a single pattern.
        let scanner = ScannerBuilder::new()
            .add_patterns(vec![test_data.pattern])
            .build();
        match scanner {
            Ok(scanner) => {
                assert!(
                    test_data.error_msg.is_none(),
                    "#{}: Parsing regex should fail",
                    test_data.test_number,
                );
                // Scanner build succeeded. Check if the matches are as expected.
                let matches = scanner.find_iter(test_data.input).collect::<Vec<_>>();
                assert_eq!(
                    matches.len(),
                    test_data.expected.len(),
                    "#{}, Differing matches count {:?}",
                    test_data.test_number,
                    matches
                );
                for (matched, (expected_match, expected_start, expected_end)) in
                    matches.iter().zip(test_data.expected.iter())
                {
                    assert_eq!(
                        &test_data.input[matched.span().start..matched.span().end],
                        *expected_match,
                        "#{}: {:?}",
                        test_data.test_number,
                        matched
                    );
                    assert_eq!(
                        matched.start(),
                        *expected_start,
                        "#{} Match start ",
                        test_data.test_number
                    );
                    assert_eq!(
                        matched.end(),
                        *expected_end,
                        "#{} Match end ",
                        test_data.test_number
                    );
                }
            }
            Err(e) => {
                // Scanner build failed. Check if the error message is as expected.
                assert!(
                    test_data.error_msg.is_some(),
                    "#{}: Unexpected error: {}",
                    test_data.test_number,
                    e
                );
                let msg = e.to_string();
                assert!(
                    msg.contains(test_data.error_msg.unwrap()),
                    "#{}:\n'{}'\ndoes not contain\n'{}'",
                    test_data.test_number,
                    e,
                    test_data.error_msg.unwrap()
                );
            }
        }
    }
    println!("{} Match tests passed", TEST_DATA.len());
}
