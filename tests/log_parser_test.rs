use log_surgeon::error_handling::Result;
use log_surgeon::log_parser::LogParser;
use log_surgeon::parser::SchemaConfig;

use std::fs::File;
use std::io::{self, BufRead};

#[test]
fn test_lexer_simple() -> Result<()> {
    let project_root = env!("CARGO_MANIFEST_DIR");
    let schema_path = std::path::Path::new(project_root)
        .join("examples")
        .join("schema.yaml");
    let log_path_dir = std::path::Path::new(project_root)
        .join("examples")
        .join("logs");
    let log_paths = vec![
        log_path_dir.clone().join("hive-24h.log"),
        log_path_dir.clone().join("hive-24h_large.log"),
    ];

    let schema_config = SchemaConfig::parse_from_file(schema_path.to_str().unwrap())?;
    let mut log_parser = LogParser::new(schema_config)?;

    for path in log_paths {
        let log_path = path.to_str().unwrap();
        log_parser.set_input_file(log_path)?;

        let mut actual = String::new();
        let mut last_log_event_line_end = 0;
        while let Some(log_event) = log_parser.parse_next_log_event()? {
            let (start_line, end_line) = log_event.get_line_range();
            assert_eq!(last_log_event_line_end + 1, start_line);
            assert_eq!(false, log_event.get_log_message_tokens().is_empty());
            last_log_event_line_end = end_line;
            actual += log_event.to_string().as_str();
        }

        let mut expected = String::new();
        let reader = io::BufReader::new(File::open(log_path).expect("failed to open log file"));
        for line in reader.lines() {
            let line = line.expect("failed to read line");
            expected += line.as_str();
            expected += "\n";
        }

        assert_eq!(false, expected.is_empty());
        assert_eq!(actual, expected);
    }

    Ok(())
}
