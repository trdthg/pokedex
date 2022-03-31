mod api;
mod cli;
mod domain;
mod repositories;
use std::sync::Arc;

use clap::Parser;
use repositories::pokemon::{InMemoryRepository, Repository, SqliteRepository};

#[derive(Parser, Debug)]
struct Opt {
    #[clap(long, help = "Runs in CLI mode")]
    cli: bool,
    #[clap(long, name = "PATH", help = "Where the database file exists")]
    sqlite: Option<String>,
}

fn main() {
    let opt = Opt::parse();
    println!("{:?}", opt);
    let repo = build_repo(opt.sqlite);
    if opt.cli {
        cli::run(repo);
    } else {
        api::serve("localhost:8000", repo);
    }
}

fn build_repo(sqlite_path: Option<String>) -> Arc<dyn Repository> {
    if sqlite_path.is_none() {
        return Arc::new(InMemoryRepository::new());
    }
    match SqliteRepository::try_new(sqlite_path.unwrap().as_str()) {
        Ok(repo) => return Arc::new(repo),
        _ => panic!("Error while creating sqlite repo"),
    }
}
