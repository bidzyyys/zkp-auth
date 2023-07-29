pub mod error;

use crate::repository;
use crate::repository::{in_memory::InMemoryRepository, ChallengeDTO, UserDTO};

use error::AuthActorError;

use zkp::chaum_pedersen::{ChaumPedersenProtocol, Context};

pub struct AuthChallenge {
    pub auth_id: String,
    pub c: i64,
}

pub struct SessionDetails {
    pub session_id: String,
}

pub struct AuthActor {
    user_repository: InMemoryRepository<UserDTO>,
    challenge_repository: InMemoryRepository<ChallengeDTO>,
    zkp: ChaumPedersenProtocol,
}

impl AuthActor {
    pub fn new(ctx: Context) -> Self {
        Self {
            user_repository: InMemoryRepository::<UserDTO>::default(),
            challenge_repository: InMemoryRepository::<ChallengeDTO>::default(),
            zkp: ChaumPedersenProtocol::new(ctx),
        }
    }

    pub fn register(&mut self, username: String, y1: i64, y2: i64) -> Result<(), AuthActorError> {
        self.user_repository
            .insert(&(username.clone()), &UserDTO::new(username, y1, y2))
            .map_err(|e| e.into())
    }

    pub fn create_authentication_challenge(
        &mut self,
        username: String,
        r1: i64,
        r2: i64,
    ) -> Result<AuthChallenge, AuthActorError> {
        if !self.user_repository.exists(&username)? {
            return Err(AuthActorError::UserNotFound);
        }
        let auth_id = username.clone();
        let c = self.zkp.create_auth_challenge();

        self.challenge_repository
            .put(&auth_id, &ChallengeDTO::new(username, r1, r2, c))
            .map_err(<repository::error::RepositoryError as Into<AuthActorError>>::into)?;

        Ok(AuthChallenge { auth_id, c })
    }
    pub fn verify_authentication(
        &self,
        auth_id: String,
        s: i64,
    ) -> Result<SessionDetails, AuthActorError> {
        let UserDTO {
            username: _,
            y1,
            y2,
        } = match self.user_repository.get(&auth_id)? {
            None => return Err(AuthActorError::UserNotFound),
            Some(user) => user,
        };

        let ChallengeDTO {
            username: _,
            r1,
            r2,
            c,
        } = match self.challenge_repository.get(&auth_id)? {
            None => return Err(AuthActorError::AuthChallengeNotFound),
            Some(challenge) => challenge,
        };

        match self.zkp.verify_auth_challenge(y1, y2, r1, r2, c, s) {
            false => Err(AuthActorError::AuthChallengeFailed),
            true => Ok(SessionDetails {
                session_id: "test session".into(),
            }),
        }
    }
}
