use std::sync::Arc;

use crate::{domain, repositories::pokemon::Repository};

use super::{prompt_name, prompt_number, prompt_types};
#[derive(Debug)]
struct Response {
    number: u16,
    name: String,
    types: Vec<String>,
}

pub fn run(repo: Arc<dyn Repository>) {
    let number = prompt_number();
    let name = prompt_name();
    let types = prompt_types();
    let req = match (number, name, types) {
        (Ok(number), Ok(name), Ok(types)) => domain::create_pokemon::Request {
            number,
            name,
            types,
        },
        _ => {
            println!("An error occurred during the prompt");
            return;
        }
    };
    println!("{:?}", req.types);
    match domain::create_pokemon::execute(repo, req) {
        Ok(res) => println!(
            "{:?}",
            Response {
                number: res.number,
                name: res.name,
                types: res.types,
            }
        ),
        Err(domain::create_pokemon::Error::BadRequest) => println!("The request is invalid"),
        Err(domain::create_pokemon::Error::Conflict) => println!("The Pokemon already exists"),
        Err(domain::create_pokemon::Error::Unknown) => println!("An unknown error occurred"),
    }
}
