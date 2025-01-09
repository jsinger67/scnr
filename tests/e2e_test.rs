// Test complete flow of the application
// Run with `cargo test --test e2e_test`

use std::{fs, path::Path};

use regex::Regex;
use scnr::{MatchExt, MatchExtIterator, ScannerBuilder, ScannerMode};

#[test]
fn e2e_test() {
    // Initialize the logger
    let _ = env_logger::builder().is_test(true).try_init();

    // Initialize the regex for newlines. It is used to make the tests platform independent.
    let rx_newline: Regex = Regex::new(r"\r?\n|\r").unwrap();

    // Define the target folder for the generated dot files.
    let target_folder = concat!(env!("CARGO_MANIFEST_DIR"), "/target/testout/e2e_test");

    // Delete all previously generated dot files.
    let _ = fs::remove_dir_all(target_folder);
    // Create the target folder.
    fs::create_dir_all(target_folder).unwrap();

    // Iterate over all json files in the data directory that contain scanner modes
    for entry in fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap() != "json"
            || path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .ends_with("_tokens")
        {
            continue;
        }

        // Select only special files
        // if entry.file_name() != "positive_lookahead_n.json" {
        //     continue;
        // }

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

        scanner
            .generate_compiled_automata_as_dot(
                path.file_stem().unwrap().to_str().unwrap(),
                Path::new(target_folder),
            )
            .expect("Failed to generate compiled automata as dot");

        // Open the input file which has the same base name as the json file but with a .input
        // extension.
        let input_path = path.with_extension("input");
        let input = fs::read_to_string(&input_path)
            .unwrap_or_else(|_| panic!("Failed to open token file {}", input_path.display()));

        let input = rx_newline.replace_all(&input, "\n");

        // Find all matches in the input file
        let find_iter = scanner.find_iter(&input).with_positions();

        // Collect all matches
        let matches: Vec<MatchExt> = find_iter.collect();

        println!("Matches:\n{}\n", serde_json::to_string(&matches).unwrap());
        for ma in &matches {
            // println!("Match: {:?}", ma);
            println!("{}, Ty: {} ", &input[ma.range()], ma.token_type());
        }
        println!("Matches count: {}", matches.len());

        // Open the expected output file which has the same base name as the json file but with a
        // _tokens.json suffix.
        let mut token_file_path = path.clone();
        token_file_path.set_file_name(format!(
            "{}_{}",
            path.file_stem().unwrap().to_str().unwrap(),
            "tokens.json"
        ));
        let token_file = fs::File::open(&token_file_path).unwrap_or_else(|e| {
            panic!(
                "Failed to open token file {} ({}): {}",
                token_file_path.display(),
                path.display(),
                e
            )
        });
        let expected_matches: Vec<MatchExt> = serde_json::from_reader(&token_file).unwrap();

        // Compare the matches
        assert_eq!(matches, expected_matches, "Failed for {}", path.display());
    }
}
