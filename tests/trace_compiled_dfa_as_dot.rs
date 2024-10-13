// Outputs the compiled DFA as in dot format for all the modes files in the data directory.
// Run with `cargo test -- --nocapture trace_compiled_dfa_as_dot`

use std::fs;

use scnr::{ScannerBuilder, ScannerMode};

#[test]
fn trace_compiled_dfa_as_dot() {
    // Initialize the logger
    let _ = env_logger::builder()
        .is_test(true)
        .parse_env(
            env_logger::Env::default().default_filter_or("scnr::internal::scanner_impl=trace"),
        )
        .try_init();

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

        // if entry.file_name() != "greedy.modes" {
        //     continue;
        // }

        println!("--------------------------------------------------");
        println!("Entry: {:?}", entry.file_name());
        println!("--------------------------------------------------");

        // Read the json file
        let file = fs::File::open(&path).unwrap();
        let scanner_modes: Vec<ScannerMode> = serde_json::from_reader(file).unwrap();

        // Create a scanner from the scanner builder
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&scanner_modes)
            .build()
            .unwrap();

        scanner
            .log_compiled_dfas_as_dot(&scanner_modes)
            .expect("Failed to trace compiled DFA as dot");
    }
}
