use std::collections::hash_map::HashMap;

use crate::repository::{error::RepositoryError, DBResult};

#[derive(Default)]
pub struct InMemoryRepository<T: Clone> {
    items: HashMap<String, T>,
}

impl<T: Clone> InMemoryRepository<T> {
    pub fn insert(&mut self, key: &str, value: &T) -> DBResult<()> {
        self.do_insert(key, value)
    }

    pub fn put(&mut self, key: &str, value: &T) -> DBResult<()> {
        self.do_put(key, value);
        Ok(())
    }

    pub fn get(&self, key: &str) -> DBResult<Option<T>> {
        Ok(self.do_get(key))
    }

    pub fn exists(&self, key: &str) -> DBResult<bool> {
        Ok(self.do_exists(key))
    }

    fn do_insert(&mut self, key: &str, value: &T) -> DBResult<()> {
        match self.do_exists(key) {
            true => match self.do_put(key, value) {
                None => Ok(()),
                Some(_) => panic!("User for key {:?} should not exist", key),
            },
            false => Err(RepositoryError::ValueAlreadyExists),
        }
    }

    fn do_put(&mut self, key: &str, value: &T) -> Option<T> {
        self.items.insert(key.into(), (*value).clone())
    }

    fn do_get(&self, key: &str) -> Option<T> {
        self.items.get(key).map(|dto| (*dto).clone())
    }

    fn do_exists(&self, key: &str) -> bool {
        self.items.contains_key(key)
    }
}
