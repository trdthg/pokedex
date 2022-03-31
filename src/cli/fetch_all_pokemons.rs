use std::sync::Arc;

use crate::{domain, repositories::pokemon::Repository};

#[derive(Debug)]
struct Response {
    number: u16,
    name: String,
    types: Vec<String>,
}

pub fn run(repo: Arc<dyn Repository>) {
    match domain::fetch_all_pokemons::execute(repo) {
        Ok(res) => {
            for res in res {
                println!(
                    "{:?}",
                    Response {
                        number: res.number,
                        name: res.name,
                        types: res.types,
                    }
                )
            }
        }
        Err(domain::fetch_all_pokemons::Error::Unknown) => println!("An unknown error occurred"),
    }
}
