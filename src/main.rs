use clap::Parser as clapParser;

use env_logger::Builder;
use log::LevelFilter;
use std::error::Error;
use std::process::ExitCode;
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

mod db;
mod file;

#[derive(clapParser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Sets the output db file defaults to results.db
    #[clap(short, long, value_parser, required(false))]
    output: Option<String>,
    /// Sets the source directory to parse
    #[clap(short, long, value_parser, required(true))]
    input: String,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let db_path: String = match &args.output {
        Some(output) => output.clone(),
        None => "results.db".to_string(),
    };
    let scan_path = args.input.clone();

    Builder::new().filter_level(LevelFilter::Info).init();

    match db::setup_database(&db_path) {
        Ok(_) => {
            log::info!("Database setup complete");
        }
        Err(_) => {
            log::info!("Database setup failed");
            return ExitCode::FAILURE;
        }
    }

    match file::search_source(scan_path, db_path) {
        Ok(_) => {
            log::info!("Search complete");
        }
        Err(_) => {
            log::info!("Search failed");
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
