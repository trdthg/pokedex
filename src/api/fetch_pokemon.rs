use std::sync::Arc;

use serde::Serialize;

use crate::{domain, repositories::pokemon::Repository};

use super::Status;

#[derive(Serialize)]
struct Request {
    number: u16,
}

#[derive(Serialize)]
struct Response {
    number: u16,
    name: String,
    types: Vec<String>,
}

pub fn serve(repo: Arc<dyn Repository>, number: u16) -> rouille::Response {
    let req = domain::fetch_pokemon::Request { number };
    match domain::fetch_pokemon::execute(repo, req) {
        Ok(domain::fetch_pokemon::Response {
            number,
            name,
            types,
        }) => rouille::Response::json(&Response {
            number,
            name,
            types,
        }),
        Err(domain::fetch_pokemon::Error::BadRequest) => {
            rouille::Response::from(Status::BadRequest)
        }
        Err(domain::fetch_pokemon::Error::NotFound) => rouille::Response::from(Status::NotFound),
        Err(domain::fetch_pokemon::Error::Unknown) => {
            rouille::Response::from(Status::InternalServerError)
        }
    }
}
