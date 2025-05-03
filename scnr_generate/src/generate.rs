use proc_macro2::TokenStream;
use syn::parse2;

use crate::scanner_data::ScannerData;

/// This function generates the scanner code from the input token stream.
/// It parses the input token stream into a `ScannerData` struct and then generates the scanner
/// code.
/// It returns a `TokenStream` containing the generated code.
/// The input token stream is expected to contain the scanner definition, including the regex
/// patterns and actions.
/// The macro syntax is expected to be used in the following way:
/// ```ignore
/// use scnr_macro::scanner;
///
/// scanner! {
///     ExampleScanner {
///         mode INITIAL {
///             token r"\r\n|\r|\n" => 1;
///             token r"[\s--\r\n]+" => 2;
///             token r"//.*(\r\n|\r|\n)?" => 3;
///             token r"/\*([^*]|\*[^/])*\*/" => 4;
///             token r#"""# => 8;
///             token r"Hello" => 9;
///             token r"World" => 10;
///             token r"World" => 11 followed by r"!";
///             token r"!" => 12 not followed by r"!";
///             token r"[a-zA-Z_]\w*" => 13;
///             token r"." => 14;
///
///             transition 8 => STRING;
///         }
///         mode STRING {
///             token r#"\\[\"\\bfnt]"# => 5;
///             token r"\\[\s--\n\r]*\r?\n" => 6;
///             token r#"[^\"\]+"# => 7;
///             token r#"""# => 8;
///             token r"." => 14;
///
///             transition 8 => INITIAL;
//          }
///     }
/// }
/// ```
/// where there must be at least one scanner mode with at least one `token` entry.
/// A `token` entry is a regex pattern followed by an arrow and a token type number.
/// Optional `not` and `followed by` modifiers can be used to specify positive and negative
/// lookaheads.
/// Zero or more `transition` entries can exist.
/// The `transition` entries are tuples of the token type numbers and the new scanner mode name.
/// The scanner mode name is later converted to the scanner mode ID and the transitions are sorted
/// by token type number.
///
/// The generated code will include the scanner implementation.
/// The generated scanner in this example will be a struct named `ExampleScanner` which implements
/// the `ScannerTrait`.
pub fn generate(input: TokenStream) -> TokenStream {
    let output = TokenStream::new();
    let _scanner_data: ScannerData = parse2(input).expect("Failed to parse input");
    output
}
