#[macro_use]
extern crate log;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use clap::{App, Arg, ArgMatches};

use parser::errors::ParserError;
use parser::io::Reader;
use parser::parsers::{MosfetFile, ParserContext};

fn main() {
    configure_logger();

    // Start CLI.
    let matches = run_cli();

    // Get input file content.
    let (file_path, content) = match read_input_file(matches.value_of("INPUT").unwrap()) {
        Some(v) => v,
        None => return,
    };

    let file_path = Arc::new(file_path);
    let content = Arc::new(content);
    info!("Parsing {:?}", file_path);

    let mut reader = Reader::new(Some(file_path.clone()), content);
    let _parsed_file = match MosfetFile::parse(&mut reader, &ParserContext::default()) {
        Ok(v) => v,
        Err(e) => {
            error!(
                "The file at {:?} cannot be parsed\n{}",
                file_path,
                e.print_error(&reader)
            );
            return;
        }
    };
}

fn configure_logger() {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info")
    }

    pretty_env_logger::init();
}

fn run_cli() -> ArgMatches {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("INPUT")
                .about("The .mos file to compile")
                .required(true),
        )
        .get_matches()
}

fn read_input_file(path: &str) -> Option<(String, String)> {
    let mut file = match File::open(path) {
        Ok(v) => v,
        Err(e) => {
            error!("Cannot open the file at '{}': {}", path, e);
            return None;
        }
    };
    let mut buffer = String::new();

    if let Err(e) = file.read_to_string(&mut buffer) {
        error!("Cannot read the file at '{}': {}", path, e);
        return None;
    }

    let file_path = PathBuf::from_str(path)
        .unwrap()
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    Some((file_path, buffer))
}
