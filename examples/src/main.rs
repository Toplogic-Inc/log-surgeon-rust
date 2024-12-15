use log_surgeon::error_handling::Result;
use log_surgeon::log_parser::LogEvent;
use log_surgeon::log_parser::LogParser;
use log_surgeon::parser::SchemaConfig;

use std::rc::Rc;

fn main() -> Result<()> {
    let project_root = env!("CARGO_MANIFEST_DIR");
    let schema_path = std::path::Path::new(project_root).join("schema_simple.yaml");
    let log_path = std::path::Path::new(project_root)
        .join("logs")
        .join("simple.log");

    let parsed_schema = Rc::new(SchemaConfig::parse_from_file(
        schema_path.to_str().unwrap(),
    )?);
    let mut log_parser = LogParser::new(parsed_schema.clone())?;
    log_parser.set_input_file(log_path.to_str().unwrap())?;

    while let Some(log_event) = log_parser.parse_next_log_event()? {
        println!("{:?}", log_event);
    }

    Ok(())
}
