use std::sync::Arc;

use crate::{domain, repositories::pokemon::Repository};

use super::prompt_number;
#[derive(Debug)]
struct Response {
    number: u16,
    name: String,
    types: Vec<String>,
}

pub fn run(repo: Arc<dyn Repository>) {
    let number = prompt_number();
    let req = match number {
        Ok(number) => domain::delete_pokemon::Request { number },
        _ => {
            println!("An error occurred during the prompt");
            return;
        }
    };
    match domain::delete_pokemon::execute(repo, req) {
        Ok(_) => {}
        Err(domain::delete_pokemon::Error::BadRequest) => println!("The request is invalid"),
        Err(domain::delete_pokemon::Error::NotFound) => println!("The Pokemon doexn;t exists"),
        Err(domain::delete_pokemon::Error::Unknown) => println!("An unknown error occurred"),
    }
}
