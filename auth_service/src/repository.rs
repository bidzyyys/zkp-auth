pub mod error;
pub mod in_memory;

type DBResult<T> = Result<T, error::RepositoryError>;

#[derive(Clone, Default)]
pub struct UserDTO {
    pub username: String,
    pub y1: i64,
    pub y2: i64,
}

impl UserDTO {
    pub fn new(username: String, y1: i64, y2: i64) -> Self {
        Self { username, y1, y2 }
    }
}

#[derive(Clone, Default)]
pub struct ChallengeDTO {
    pub username: String,
    pub r1: i64,
    pub r2: i64,
    pub c: i64,
}

impl ChallengeDTO {
    pub fn new(username: String, r1: i64, r2: i64, c: i64) -> Self {
        Self {
            username,
            r1,
            r2,
            c,
        }
    }
}
