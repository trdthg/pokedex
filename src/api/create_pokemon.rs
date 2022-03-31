use std::sync::Arc;

use rouille::{self};
use serde::{Deserialize, Serialize};

use crate::{domain, repositories::pokemon::Repository};

use super::Status;

#[derive(Serialize)]
struct Response {
    number: u16,
    name: String,
    types: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Request {
    number: u16,
    name: String,
    types: Vec<String>,
}

pub fn serve(repo: Arc<dyn Repository>, req: &rouille::Request) -> rouille::Response {
    let req: domain::create_pokemon::Request = match rouille::input::json_input::<Request>(req) {
        Ok(req) => domain::create_pokemon::Request {
            number: req.number,
            name: req.name,
            types: req.types,
        },
        _ => return rouille::Response::from(Status::BadRequest),
    };
    // rouille::Response::from(Status::InternalServerError)
    match domain::create_pokemon::execute(repo, req) {
        Ok(domain::create_pokemon::Response {
            number,
            name,
            types,
        }) => rouille::Response::json(&Response {
            number,
            name,
            types,
        }),
        Err(domain::create_pokemon::Error::BadRequest) => {
            rouille::Response::from(Status::BadRequest)
        }
        Err(domain::create_pokemon::Error::Conflict) => rouille::Response::from(Status::Conflict),
        Err(domain::create_pokemon::Error::Unknown) => {
            rouille::Response::from(Status::InternalServerError)
        }
    }
}
