use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, params_from_iter, Connection};

use crate::domain::entities::{Pokemon, PokemonName, PokemonNumber, PokemonTypes};

use super::{DeleteError, FetchAllError, FetchOneError, InsertError, Repository};

pub struct SqliteRepository {
    connection: Mutex<Connection>,
}

impl SqliteRepository {
    pub fn try_new(path: &str) -> Result<Self, ()> {
        let connection = match rusqlite::Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
        ) {
            Ok(connection) => connection,
            Err(_) => return Err(()),
        };
        match connection.execute("pragma foreign_key = 1", []) {
            Ok(_) => Ok(Self {
                connection: Mutex::new(connection),
            }),
            Err(_) => return Err(()),
        }
    }

    fn fetch_pokemon_rows(
        lock: &MutexGuard<'_, Connection>,
        number: Option<u16>,
    ) -> Result<Vec<(u16, String)>, ()> {
        let (query, params) = match number {
            Some(n) => (
                "select number, name from pokemons where number = ?",
                vec![n],
            ),
            _ => ("select number, name from pokemons", vec![]),
        };
        let mut stat = match lock.prepare(query) {
            Ok(stat) => stat,
            _ => return Err(()),
        };
        let rows = stat.query(params_from_iter(params));
        if rows.is_err() {
            return Err(());
        }
        let mut rows = rows.unwrap();
        let mut pokemon_rows = vec![];
        while let Ok(Some(row)) = rows.next() {
            match (row.get::<usize, u16>(0), row.get::<usize, String>(1)) {
                (Ok(number), Ok(name)) => pokemon_rows.push((number, name)),
                _ => return Err(()),
            };
        }
        Ok(pokemon_rows)
    }

    fn fetch_type_rows(lock: &MutexGuard<'_, Connection>, number: u16) -> Result<Vec<String>, ()> {
        let mut stat = match lock.prepare("select * from types where pokemon_number = ?") {
            Ok(stat) => stat,
            _ => return Err(()),
        };
        let mut rows = match stat.query([number]) {
            Ok(rows) => rows,
            _ => return Err(()),
        };
        let mut type_rows = vec![];
        while let Ok(Some(row)) = rows.next() {
            match row.get::<usize, String>(1) {
                Ok(r#type) => type_rows.push(r#type),
                _ => return Err(()),
            }
        }
        Ok(type_rows)
    }
}

impl Repository for SqliteRepository {
    fn insert(
        &self,
        number: crate::domain::entities::PokemonNumber,
        name: crate::domain::entities::PokemonName,
        types: crate::domain::entities::PokemonTypes,
    ) -> Result<crate::domain::entities::Pokemon, super::InsertError> {
        let mut lock = match self.connection.lock() {
            Ok(lock) => lock,
            _ => return Err(InsertError::Unknown),
        };
        let transaction = match lock.transaction() {
            Ok(t) => t,
            _ => return Err(InsertError::Unknown),
        };
        match transaction.execute(
            "insert into pokemons(number, name) values (?, ?)",
            params![u16::from(number.clone()), String::from(name.clone())],
        ) {
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(_, Some(msg)))
                if msg == "UNIQUE constraint failed: pokemons.number" =>
            {
                return Err(InsertError::Conflict)
            }

            Err(_) => return Err(InsertError::Unknown),
        }
        for r#type in Vec::<String>::from(types.clone()) {
            if let Err(_) = transaction.execute(
                "insert into types (pokemon_number, name) values (?, ?)",
                params![u16::from(number.clone()), r#type],
            ) {
                return Err(InsertError::Unknown);
            }
        }
        match transaction.commit() {
            Ok(_) => Ok(Pokemon::new(number, name, types)),
            _ => Err(InsertError::Unknown),
        }
    }

    fn fetch_all(&self) -> Result<Vec<crate::domain::entities::Pokemon>, super::FetchAllError> {
        let lock = match self.connection.lock() {
            Ok(lock) => lock,
            _ => return Err(FetchAllError::Unknown),
        };
        let rows = match Self::fetch_pokemon_rows(&lock, None) {
            Ok(rows) => rows,
            _ => return Err(FetchAllError::Unknown),
        };
        let mut pokemons = vec![];
        for row in rows {
            let types = match Self::fetch_type_rows(&lock, row.0) {
                Ok(types) => types,
                _ => return Err(FetchAllError::Unknown),
            };
            let pokemon = match (
                PokemonNumber::try_from(row.0),
                PokemonName::try_from(row.1),
                PokemonTypes::try_from(types),
            ) {
                (Ok(number), Ok(name), Ok(types)) => Pokemon::new(number, name, types),
                _ => return Err(FetchAllError::Unknown),
            };
            pokemons.push(pokemon);
        }
        Ok(pokemons)
    }

    fn fetch_one(
        &self,
        number: crate::domain::entities::PokemonNumber,
    ) -> Result<crate::domain::entities::Pokemon, super::FetchOneError> {
        let lock = match self.connection.lock() {
            Ok(lock) => lock,
            _ => return Err(FetchOneError::Unknown),
        };
        let mut rows = match Self::fetch_pokemon_rows(&lock, Some(u16::from(number.clone()))) {
            Ok(rows) => rows,
            _ => return Err(FetchOneError::Unknown),
        };
        if rows.is_empty() {
            return Err(FetchOneError::NotFound);
        }
        let types = match Self::fetch_type_rows(&lock, u16::from(number)) {
            Ok(types) => types,
            _ => return Err(FetchOneError::Unknown),
        };
        let row = rows.remove(0);
        match (
            PokemonNumber::try_from(row.0),
            PokemonName::try_from(row.1),
            PokemonTypes::try_from(types),
        ) {
            (Ok(number), Ok(name), Ok(types)) => Ok(Pokemon::new(number, name, types)),
            _ => Err(FetchOneError::Unknown),
        }
    }

    fn delete(
        &self,
        number: crate::domain::entities::PokemonNumber,
    ) -> Result<(), super::DeleteError> {
        let lock = match self.connection.lock() {
            Ok(lock) => lock,
            _ => return Err(DeleteError::Unknown),
        };
        match lock.execute(
            "delete from pokemons where number = ?",
            params![u16::from(number)],
        ) {
            Ok(0) => Err(DeleteError::NotFound),
            Ok(_) => Ok(()),
            _ => Err(DeleteError::Unknown),
        }
    }
}
