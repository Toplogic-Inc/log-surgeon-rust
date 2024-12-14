use log_surgeon::error_handling::Result;
use log_surgeon::lexer::BufferedFileStream;
use log_surgeon::lexer::Lexer;
use log_surgeon::parser::ParsedSchema;

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

    let parsed_schema = ParsedSchema::parse_from_file(schema_path.to_str().unwrap())?;
    let mut lexer = Lexer::new(&parsed_schema)?;
    let buffered_file_stream = Box::new(BufferedFileStream::new(log_path.to_str().unwrap())?);
    lexer.set_input_stream(buffered_file_stream);

    let mut tokens = Vec::new();
    while let Some(token) = lexer.get_next_token()? {
        tokens.push(token);
    }
    assert_eq!(false, tokens.is_empty());

    for token in &tokens {
        // TODO: Add meaningful assertion when DFA bug is fixed
        println!("{:?}", token);
    }

    Ok(())
}
