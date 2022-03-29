mod api;
mod domain;
mod repositories;
use std::sync::Arc;

use repositories::pokemon::InMemoryRepository;

fn main() {
    let repo = Arc::new(InMemoryRepository::new());
    api::serve("localhost:8000", repo);
}
