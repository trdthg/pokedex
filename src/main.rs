mod api;
mod cli;
mod domain;
mod repositories;
use std::sync::Arc;

use clap::{Args, Parser};
use repositories::pokemon::{AirtableRepository, InMemoryRepository, Repository, SqliteRepository};

#[derive(Parser, Debug)]
struct Opt {
    #[clap(long, help = "Runs in CLI mode")]
    cli: bool,
    #[clap(long, name = "PATH", help = "Where the database file exists")]
    sqlite: Option<String>,
    #[clap(long, value_names = &["API_KEY", "WORKSPACE_ID"], help = "Use airtable as repository")]
    airtable: Vec<String>,
}

fn main() {
    let opt = Opt::parse();
    let repo = build_repo(opt.sqlite, opt.airtable);
    if opt.cli {
        cli::run(repo);
    } else {
        api::serve("localhost:8000", repo);
    }
}

fn build_repo(sqlite_path: Option<String>, airtable_value: Vec<String>) -> Arc<dyn Repository> {
    if let Some(sqlite_path) = sqlite_path {
        match SqliteRepository::try_new(sqlite_path.as_str()) {
            Ok(repo) => return Arc::new(repo),
            _ => panic!("Error while creating sqlite repo"),
        }
    }
    if let [api_key, workspace_id] = &airtable_value[..] {
        match AirtableRepository::try_new(api_key, workspace_id) {
            Ok(repo) => return Arc::new(repo),
            _ => panic!("Error while creating airtable repo"),
        }
    }
    return Arc::new(InMemoryRepository::new());
}
