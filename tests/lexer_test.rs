use log_surgeon::error_handling::Result;
use log_surgeon::lexer::BufferedFileStream;
use log_surgeon::lexer::Lexer;
use log_surgeon::parser::SchemaConfig;

use std::fs::File;
use std::io::{self, BufRead};
use std::rc::Rc;

#[test]
fn test_lexer_simple() -> Result<()> {
    let project_root = env!("CARGO_MANIFEST_DIR");
    let schema_path = std::path::Path::new(project_root)
        .join("examples")
        .join("schema_simple.yaml");
    let log_path = std::path::Path::new(project_root)
        .join("examples")
        .join("logs")
        .join("simple.log");

    let parsed_schema = Rc::new(SchemaConfig::parse_from_file(
        schema_path.to_str().unwrap(),
    )?);
    let mut lexer = Lexer::new(parsed_schema)?;
    let buffered_file_stream = Box::new(BufferedFileStream::new(log_path.to_str().unwrap())?);
    lexer.set_input_stream(buffered_file_stream);

    let mut tokens = Vec::new();
    while let Some(token) = lexer.get_next_token()? {
        tokens.push(token);
    }
    assert_eq!(false, tokens.is_empty());

    let mut parsed_lines = Vec::new();
    let mut parsed_line = String::new();
    let mut curr_line_num = 0usize;
    for token in &tokens {
        if curr_line_num != token.get_line_num() {
            parsed_lines.push(parsed_line.clone());
            parsed_line.clear();
            curr_line_num += 1;
        }
        parsed_line += &token.get_val().to_string();
    }
    parsed_lines.push(parsed_line.clone());

    let mut expected_lines = Vec::new();
    let reader = io::BufReader::new(File::open(log_path).expect("failed to open log file"));
    for line in reader.lines() {
        let line = line.expect("failed to read line");
        expected_lines.push(line + "\n");
    }

    assert_eq!(parsed_lines.len(), expected_lines.len());
    assert_eq!(false, parsed_line.is_empty());
    assert_eq!(parsed_lines, expected_lines);

    Ok(())
}
