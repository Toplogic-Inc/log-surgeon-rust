use log_surgeon::error_handling::Result;
use log_surgeon::lexer::BufferedFileStream;
use log_surgeon::lexer::Lexer;
use log_surgeon::parser::SchemaConfig;

use clap::{Arg, Command};

fn main() -> Result<()> {
    let matches = Command::new("log-surgeon-example")
        .version("1.0")
        .arg(
            Arg::new("schema")
                .help("Path to the schema file")
                .required(true)
                .value_name("SCHEMA_FILE"),
        )
        .arg(
            Arg::new("input")
                .help("Paths to the input file")
                .required(true)
                .value_name("INPUT_FILE"),
        )
        .get_matches();

    let schema_path: &String = matches.get_one("schema").expect("no schema found");
    let input_file: &String = matches.get_one("input").expect("no input file found");

    let schema_path = std::path::Path::new(schema_path.as_str());
    let log_path = std::path::Path::new(input_file.as_str());

    let parsed_schema = SchemaConfig::parse_from_file(schema_path.to_str().unwrap())?;
    let mut lexer = Lexer::new(parsed_schema)?;
    let buffered_file_stream = Box::new(BufferedFileStream::new(log_path.to_str().unwrap())?);
    lexer.set_input_stream(buffered_file_stream);

    while let Some(token) = lexer.get_next_token()? {
        println!("{:?}", token);
    }

    Ok(())
}
