use log_surgeon::error_handling::Result;
use log_surgeon::log_parser::LogParser;
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
    let mut log_parser = LogParser::new(parsed_schema.clone())?;
    log_parser.set_input_file(log_path.to_str().unwrap())?;

    let mut log_event_idx = 0;
    while let Some(log_event) = log_parser.parse_next_log_event()? {
        println!("Log Event #{}", log_event_idx);
        println!("{:?}", log_event);
        log_event_idx += 1;
    }

    Ok(())
}
