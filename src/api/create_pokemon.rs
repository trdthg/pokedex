use std::sync::Arc;

use rouille::{self};
use serde::{Deserialize, Serialize};

use crate::{domain, repositories::pokemon::Repository};

use super::Status;

#[derive(Serialize)]
struct Response {
    number: u16,
}

#[derive(Serialize, Deserialize)]
struct Request {
    number: u16,
    name: String,
    types: Vec<String>,
}

pub fn serve(repo: Arc<dyn Repository>, req: &rouille::Request) -> rouille::Response {
    println!("{req:?}");
    let req: domain::create_pokemon::Request = match rouille::input::json_input::<Request>(req) {
        Ok(req) => domain::create_pokemon::Request {
            number: req.number,
            name: req.name,
            types: req.types,
        },
        _ => return rouille::Response::from(Status::BadRequest),
    };
    match domain::create_pokemon::execute(repo, req) {
        domain::create_pokemon::Response::Ok(number) => {
            rouille::Response::json(&Response { number })
        }
        domain::create_pokemon::Response::BadRequest => rouille::Response::from(Status::BadRequest),
        domain::create_pokemon::Response::Conflict => rouille::Response::from(Status::Conflict),
        domain::create_pokemon::Response::Error => {
            rouille::Response::from(Status::InternalServerError)
        }
    }
}
