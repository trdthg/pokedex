mod memory;
mod sqlite;
pub use memory::InMemoryRepository;
pub use sqlite::SqliteRepository;
use std::sync::Mutex;

use crate::domain::entities::{Pokemon, PokemonName, PokemonNumber, PokemonTypes};
pub trait Repository: Send + Sync {
    fn insert(
        &self,
        number: PokemonNumber,
        name: PokemonName,
        types: PokemonTypes,
    ) -> Result<Pokemon, InsertError>;
    fn fetch_all(&self) -> Result<Vec<Pokemon>, FetchAllError>;
    fn fetch_one(&self, number: PokemonNumber) -> Result<Pokemon, FetchOneError>;
    fn delete(&self, number: PokemonNumber) -> Result<(), DeleteError>;
}
pub enum InsertError {
    Conflict,
    Unknown,
}

pub enum FetchAllError {
    Unknown,
}
pub enum FetchOneError {
    Unknown,
    NotFound,
}
pub enum DeleteError {
    NotFound,
    Unknown,
}
