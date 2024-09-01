// Test complete flow of the application
// Run with `cargo test --test e2e_test`

use std::fs;

use regex::Regex;
use scnr::{Match, ScannerBuilder, ScannerMode};

#[test]
fn e2e_test() {
    // Initialize the logger
    let _ = env_logger::builder().is_test(true).try_init();

    // Initialize the regex for newlines. It is used to make the tests platform independent.
    let rx_newline: Regex = Regex::new(r"\r?\n|\r").unwrap();

    // Iterate over all json files in the data directory that contain scanner modes
    for entry in fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap() != "json" {
            continue;
        }

        println!("--------------------------------------------------");
        println!("Entry: {:?}", entry.file_name());
        println!("--------------------------------------------------");

        // Read the json file
        let file = fs::File::open(&path).unwrap();
        let scanner_modes: Vec<ScannerMode> = serde_json::from_reader(file)
            .unwrap_or_else(|e| panic!("**** Failed to read json file {}: {}", path.display(), e));

        // Create a scanner from the scanner builder
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&scanner_modes)
            .build()
            .unwrap();

        // Open the input file which has the same base name as the json file but with a .input
        // extension.
        let input_path = path.with_extension("input");
        let input = fs::read_to_string(&input_path).unwrap();
        let input = rx_newline.replace_all(&input, "\n");

        // Find all matches in the input file
        let find_iter = scanner.find_iter(&input);

        // Collect all matches
        let matches: Vec<Match> = find_iter.collect();

        println!("Matches:\n{}\n", serde_json::to_string(&matches).unwrap());
        for ma in &matches {
            // println!("Match: {:?}", ma);
            println!("{}, Ty: {} ", &input[ma.range()], ma.token_type());
        }
        println!("Matches count: {}", matches.len());

        // Open the expected output file which has the same base name as the json file but with a
        // .tokens extension.
        let token_file_path = path.with_extension("tokens");
        let token_file = fs::File::open(&token_file_path).unwrap();
        let expected_matches: Vec<Match> = serde_json::from_reader(&token_file).unwrap();

        // Compare the matches
        assert_eq!(matches, expected_matches);
    }
}
