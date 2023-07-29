use crate::repository::error::RepositoryError;

pub enum AuthActorError {
    AuthChallengeNotFound,
    AuthChallengeFailed,
    UserAlreadyRegistered,
    UserNotFound,
}

impl From<RepositoryError> for AuthActorError {
    fn from(val: RepositoryError) -> Self {
        match val {
            RepositoryError::ValueAlreadyExists => AuthActorError::UserAlreadyRegistered,
        }
    }
}
