use log_surgeon::error_handling::Result;
use log_surgeon::lexer::BufferedFileStream;
use log_surgeon::lexer::Lexer;
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
    let mut lexer = Lexer::new(schema_config)?;

    for path in &log_paths {
        let log_path = path.to_str().unwrap();
        let buffered_file_stream = Box::new(BufferedFileStream::new(log_path)?);
        lexer.set_input_stream(buffered_file_stream);

        let mut tokens = Vec::new();
        while let Some(token) = lexer.get_next_token()? {
            tokens.push(token);
        }
        assert_eq!(false, tokens.is_empty());

        let mut parsed_lines = Vec::new();
        let mut parsed_line = String::new();
        let mut curr_line_num = 1usize;
        for token in &tokens {
            if curr_line_num != token.get_line_num() {
                parsed_lines.push(parsed_line.clone());
                parsed_line.clear();
                curr_line_num += 1;
            }
            parsed_line += &token.get_val().to_string();
            println!("{:?}", token);
        }
        parsed_lines.push(parsed_line.clone());
        println!("{:?}", parsed_lines);

        let mut expected_lines = Vec::new();
        let reader = io::BufReader::new(File::open(log_path).expect("failed to open log file"));
        for line in reader.lines() {
            let line = line.expect("failed to read line");
            expected_lines.push(line.clone() + "\n");
        }

        assert_eq!(parsed_lines.len(), expected_lines.len());
        assert_eq!(false, parsed_line.is_empty());
        assert_eq!(parsed_lines, expected_lines);
    }

    Ok(())
}
