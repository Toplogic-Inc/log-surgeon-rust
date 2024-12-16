use log_surgeon::lexer::Lexer;
use log_surgeon::lexer::{BufferedFileStream, LexerStream};
use log_surgeon::log_parser::LogParser;
use log_surgeon::parser::SchemaConfig;

use clap::{Arg, Command};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use std::time::{Duration, Instant};

fn find_files<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut result = Vec::new();
    let entries = fs::read_dir(&path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // If the entry is a directory, recursively search inside it.
            result.extend(find_files(&path)?);
        } else {
            // If it's not a directory, add it to the result.
            result.push(path);
        }
    }

    Ok(result)
}

fn benchmark_log_parser(
    schema_config: std::rc::Rc<SchemaConfig>,
    input_log_paths: Vec<PathBuf>,
) -> log_surgeon::error_handling::Result<()> {
    let mut log_parser = LogParser::new(schema_config.clone())?;

    let mut total_duration = Duration::new(0, 0);
    let mut total_size: u64 = 0;
    let mut total_tokens: usize = 0;

    for log_path in input_log_paths {
        println!("Parsing file: {}", log_path.to_str().unwrap());
        total_size += log_path
            .metadata()
            .expect("Failed to get file metadata")
            .len();
        log_parser.set_input_file(log_path.to_str().unwrap())?;
        let mut log_event_idx = 0;
        let mut num_tokens = 0;
        let start = Instant::now();
        while let Some(log_event) = log_parser.parse_next_log_event()? {
            log_event_idx += 1;
            num_tokens += log_event.get_num_tokens();
        }
        total_duration += start.elapsed();
        total_tokens += num_tokens;
        println!(
            "Num log events: {}; Num tokens: {}",
            log_event_idx, num_tokens
        );
    }

    println!("\nBenchmark log parser:");
    println!(
        "Total size: {}GB",
        total_size as f64 / (1024 * 1024 * 1024) as f64
    );
    println!("Total number of tokens: {}", total_tokens);
    println!(
        "Total duration: {}s",
        total_duration.as_millis() as f64 / 1000 as f64
    );
    println!(
        "Token throughput: {} per second",
        (total_tokens * 1000) as f64 / total_duration.as_millis() as f64
    );
    println!(
        "Parsing throughput: {}MB per second",
        (total_size * 1000) as f64 / total_duration.as_millis() as f64 / (1024 * 1024) as f64
    );

    Ok(())
}

fn benchmark_lexer(
    schema_config: std::rc::Rc<SchemaConfig>,
    input_log_paths: Vec<PathBuf>,
) -> log_surgeon::error_handling::Result<()> {
    let mut lexer = Lexer::new(schema_config.clone())?;

    let mut total_duration = Duration::new(0, 0);
    let mut total_size: u64 = 0;
    let mut total_tokens: usize = 0;

    for log_path in input_log_paths {
        println!("Parsing file: {}", log_path.to_str().unwrap());
        total_size += log_path
            .metadata()
            .expect("Failed to get file metadata")
            .len();
        let buffered_file = Box::new(BufferedFileStream::new(log_path.to_str().unwrap())?);
        lexer.set_input_stream(buffered_file);
        let mut num_tokens = 0;
        let start = Instant::now();
        while let Some(log_surgeon) = lexer.get_next_token()? {
            num_tokens += 1;
        }
        total_duration += start.elapsed();
        total_tokens += num_tokens;
        println!("Num tokens: {}", num_tokens);
    }

    println!("\nBenchmark lexer:");
    println!(
        "Total size: {}GB",
        total_size as f64 / (1024 * 1024 * 1024) as f64
    );
    println!("Total number of tokens: {}", total_tokens);
    println!(
        "Total duration: {}s",
        total_duration.as_millis() as f64 / 1000 as f64
    );
    println!(
        "Token throughput: {} per second",
        (total_tokens * 1000) as f64 / total_duration.as_millis() as f64
    );
    println!(
        "Parsing throughput: {}MB per second",
        (total_size * 1000) as f64 / total_duration.as_millis() as f64 / (1024 * 1024) as f64
    );

    Ok(())
}

fn main() -> log_surgeon::error_handling::Result<()> {
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
                .help("Directory to the input files")
                .required(true)
                .value_name("INPUT_DIR"),
        )
        .arg(
            Arg::new("lexer")
                .long("lexer")
                .help("Benchmark lexer; otherwise benchmark parser")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let schema_path: &String = matches.get_one("schema").expect("no schema found");
    let input_dir: &String = matches.get_one("input").expect("no input file found");

    let schema_path = Path::new(schema_path.as_str());
    let parsed_schema = SchemaConfig::parse_from_file(schema_path.to_str().unwrap())?;
    let log_dir = Path::new(input_dir.as_str());
    let input_log_paths = find_files(log_dir).unwrap();

    if matches.get_flag("lexer") {
        benchmark_lexer(parsed_schema, input_log_paths)
    } else {
        benchmark_log_parser(parsed_schema, input_log_paths)
    }
}
