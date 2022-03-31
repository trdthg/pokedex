mod api;
mod cli;
mod domain;
mod repositories;
use std::sync::Arc;

use clap::Parser;
use repositories::pokemon::InMemoryRepository;

#[derive(Parser)]
struct Opt {
    #[clap(long)]
    cli: bool,
}

fn main() {
    let repo = Arc::new(InMemoryRepository::new());
    let opt = Opt::parse();
    if opt.cli {
        cli::run(repo);
    } else {
        api::serve("localhost:8000", repo);
    }
}
