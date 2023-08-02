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
            false => match self.do_put(key, value) {
                None => Ok(()),
                Some(_) => panic!("User for key {:?} should not exist", key),
            },
            true => Err(RepositoryError::ValueAlreadyExists),
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

#[cfg(test)]
mod tests {
    use super::*;

    const DB_KEY: &str = "test_key";

    fn init() -> InMemoryRepository<String> {
        InMemoryRepository::<String>::default()
    }

    #[test]
    fn should_not_exist() {
        let db = init();
        assert!(!db.exists(DB_KEY).unwrap());
        assert_eq!(db.get(DB_KEY).unwrap(), None);
    }

    #[test]
    fn should_insert() {
        let mut db = init();
        let value: String = String::from("test value");

        assert!(!db.exists(DB_KEY).unwrap());
        assert_eq!(db.insert(DB_KEY, &value), Ok(()));
        assert!(db.exists(DB_KEY).unwrap());
        assert_eq!(db.get(DB_KEY).unwrap(), Some(value));
    }

    #[test]
    fn should_not_insert_twice() {
        let mut db = init();
        let value: String = String::from("test value");

        assert!(!db.exists(DB_KEY).unwrap());
        assert_eq!(db.insert(DB_KEY, &value), Ok(()));
        assert_eq!(
            db.insert(DB_KEY, &value),
            Err(RepositoryError::ValueAlreadyExists)
        );
    }

    #[test]
    fn should_put_nonexistent_value() {
        let mut db = init();
        let value: String = String::from("test value");

        assert!(!db.exists(DB_KEY).unwrap());
        assert_eq!(db.put(DB_KEY, &value), Ok(()));
        assert!(db.exists(DB_KEY).unwrap());
        assert_eq!(db.get(DB_KEY).unwrap(), Some(value));
    }

    #[test]
    fn should_update_existent_value_with_put() {
        let mut db = init();
        let value: String = String::from("test value");
        let updated_value: String = String::from("updated value");

        assert!(!db.exists(DB_KEY).unwrap());
        assert_eq!(db.insert(DB_KEY, &value), Ok(()));
        assert_eq!(db.put(DB_KEY, &updated_value), Ok(()));
        assert_eq!(db.get(DB_KEY).unwrap(), Some(updated_value));
    }
}
