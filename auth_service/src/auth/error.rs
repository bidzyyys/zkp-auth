use crate::repository::error::RepositoryError;
use zkp::ZKPError;

pub enum AuthActorError {
    AuthChallengeNotFound,
    AuthChallengeFailed,
    UserAlreadyRegistered,
    UserNotFound,
    ZKPMathError,
}

impl From<RepositoryError> for AuthActorError {
    fn from(val: RepositoryError) -> Self {
        match val {
            RepositoryError::ValueAlreadyExists => AuthActorError::UserAlreadyRegistered,
        }
    }
}

impl From<ZKPError> for AuthActorError {
    fn from(_val: ZKPError) -> Self {
        AuthActorError::ZKPMathError
    }
}
