use serde::Deserialize;

use crate::domain::entities::{Pokemon, PokemonName, PokemonNumber, PokemonTypes};

use super::{DeleteError, FetchAllError, FetchOneError, InsertError, Repository};

pub struct AirtableRepository {
    url: String,
    auth_header: String,
}
#[derive(Deserialize)]
struct AirtableJson {
    records: Vec<AirtableRecord>,
}

#[derive(Deserialize)]
struct AirtableRecord {
    id: String,
    createdTime: String,
    fields: AirtableFields,
}

#[derive(Deserialize)]
struct AirtableFields {
    number: u16,
    name: String,
    types: Vec<String>,
}

impl AirtableRepository {
    pub fn try_new(api_key: &str, workspace_id: &str) -> Result<Self, ()> {
        let url = format!("https://api.airtable.com/v0/app4rbiOzPiOCE20j/pokemons");
        let auth_header = format!("Bearer {}", api_key);

        let res = ureq::get(&url).set("Authorization", &auth_header).call();
        if let Err(_) = res {
            return Err(());
        }
        Ok(Self { url, auth_header })
    }

    fn fetch_pokemon_rows(&self, number: Option<u16>) -> Result<AirtableJson, ()> {
        // code will go here
        let url = match number {
            Some(number) => format!("{}?filterByFormula=number%3D{}", self.url, number),
            None => format!("{}?sort%5B0%5D%5Bfield%5D=number", self.url),
        };
        let res = match ureq::get(&url)
            .set("Authorization", &self.auth_header)
            .call()
        {
            Ok(res) => res,
            _ => return Err(()),
        };
        match res.into_json::<AirtableJson>() {
            Ok(json) => Ok(json),
            _ => Err(()),
        }
    }
}
impl Repository for AirtableRepository {
    fn insert(
        &self,
        number: crate::domain::entities::PokemonNumber,
        name: crate::domain::entities::PokemonName,
        types: crate::domain::entities::PokemonTypes,
    ) -> Result<crate::domain::entities::Pokemon, super::InsertError> {
        let json = match self.fetch_pokemon_rows(Some(u16::from(number.clone()))) {
            Ok(json) => json,
            _ => return Err(InsertError::Unknown),
        };

        if !json.records.is_empty() {
            return Err(InsertError::Conflict);
        }
        let body = ureq::json!({
            "records": [{
                "fields": {
                    "number": u16::from(number.clone()),
                    "name": String::from(name.clone()),
                    "types": Vec::<String>::from(types.clone()),
                },
            }],
        });
        if let Err(_) = ureq::post(&self.url)
            .set("Authorization", &self.auth_header)
            .send_json(body)
        {
            return Err(InsertError::Unknown);
        }

        Ok(Pokemon::new(number, name, types))
    }

    fn fetch_all(&self) -> Result<Vec<crate::domain::entities::Pokemon>, super::FetchAllError> {
        let json = match self.fetch_pokemon_rows(None) {
            Ok(json) => json,
            _ => return Err(FetchAllError::Unknown),
        };
        let mut pokemons = vec![];

        for record in json.records.into_iter() {
            match (
                PokemonNumber::try_from(record.fields.number),
                PokemonName::try_from(record.fields.name),
                PokemonTypes::try_from(record.fields.types),
            ) {
                (Ok(number), Ok(name), Ok(types)) => {
                    pokemons.push(Pokemon::new(number, name, types))
                }
                _ => return Err(FetchAllError::Unknown),
            }
        }

        Ok(pokemons)
    }

    fn fetch_one(
        &self,
        number: crate::domain::entities::PokemonNumber,
    ) -> Result<crate::domain::entities::Pokemon, super::FetchOneError> {
        let mut json = match self.fetch_pokemon_rows(Some(u16::from(number.clone()))) {
            Ok(json) => json,
            _ => return Err(FetchOneError::Unknown),
        };
        if json.records.is_empty() {
            return Err(FetchOneError::NotFound);
        }

        let record = json.records.remove(0);
        match (
            PokemonNumber::try_from(record.fields.number),
            PokemonName::try_from(record.fields.name),
            PokemonTypes::try_from(record.fields.types),
        ) {
            (Ok(number), Ok(name), Ok(types)) => Ok(Pokemon::new(number, name, types)),
            _ => Err(FetchOneError::Unknown),
        }
    }

    fn delete(
        &self,
        number: crate::domain::entities::PokemonNumber,
    ) -> Result<(), super::DeleteError> {
        let mut json = match self.fetch_pokemon_rows(Some(u16::from(number.clone()))) {
            Ok(json) => json,
            _ => return Err(DeleteError::Unknown),
        };

        if json.records.is_empty() {
            return Err(DeleteError::NotFound);
        }

        let record = json.records.remove(0);
        match ureq::delete(&format!("{}/{}", self.url, record.id))
            .set("Authorization", &self.auth_header)
            .call()
        {
            Ok(_) => Ok(()),
            _ => Err(DeleteError::Unknown),
        }
    }
}

#[test]
fn test() {
    let api_key = "key3qwcREeUm8u8QE";
    let workspace_id = "wsp7OQ4vUAFY6v9M5";
    let url = format!("https://api.airtable.com/v0/app4rbiOzPiOCE20j/pokemons");
    let auth_header = format!("Bearer {}", api_key);

    let res = ureq::get(&url).set("Authorization", &auth_header).call();
    println!("{:#?}", res);
}
