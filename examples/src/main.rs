use log_surgeon::error_handling::Result;
use log_surgeon::log_parser::LogParser;
use log_surgeon::parser::SchemaConfig;

use std::rc::Rc;

fn main() -> Result<()> {
    let project_root = env!("CARGO_MANIFEST_DIR");
    let schema_path = std::path::Path::new(project_root).join("schema.yaml");
    let log_path = std::path::Path::new(project_root)
        .join("logs")
        .join("hive-24h.log");

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
