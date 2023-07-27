use actix_web::HttpResponse;

use tonic::{transport::Channel, Request};

use crate::{
    zkp_auth::{AuthenticationAnswerRequest, AuthenticationChallengeRequest, RegisterRequest},
    AuthClient,
};

mod crypto;
mod error;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct RegisterData {
    pub username: String,
    pub y1: i64,
    pub y2: i64,
}

impl From<RegisterData> for RegisterRequest {
    fn from(val: RegisterData) -> Self {
        RegisterRequest {
            user: val.username,
            y1: val.y1,
            y2: val.y2,
        }
    }
}

impl From<RegisterData> for HttpResponse {
    fn from(val: RegisterData) -> Self {
        HttpResponse::Created()
            .body(serde_json::to_string(&val).expect("`RegisterData` is serializable to json"))
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LoginData {
    pub username: String,
    pub r1: i64,
    pub r2: i64,
}

impl From<LoginData> for AuthenticationChallengeRequest {
    fn from(val: LoginData) -> Self {
        AuthenticationChallengeRequest {
            user: val.username,
            r1: val.r1,
            r2: val.r2,
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthChallengeData {
    pub auth_id: String,
    pub s: i64,
}

impl From<AuthChallengeData> for AuthenticationAnswerRequest {
    fn from(val: AuthChallengeData) -> Self {
        AuthenticationAnswerRequest {
            auth_id: val.auth_id,
            s: val.s,
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionData {
    pub session_id: String,
}

impl From<SessionData> for HttpResponse {
    fn from(val: SessionData) -> Self {
        HttpResponse::Ok()
            .body(serde_json::to_string(&val).expect("`SessionData` is serializable to json"))
    }
}

pub async fn register(
    auth_client: &mut AuthClient<Channel>,
    register_data: RegisterData,
) -> Result<RegisterData, error::AuthClientError> {
    auth_client
        .register(Request::new(register_data.clone().into()))
        .await
        .map_or_else(
            |_status| Err(error::AuthClientError::ConnectionFailed),
            |_response| Ok(register_data),
        )
}

pub async fn login(
    auth_client: &mut AuthClient<Channel>,
    login_data: LoginData,
) -> Result<SessionData, error::AuthClientError> {
    let req: AuthenticationChallengeRequest = login_data.into();
    let (auth_id, c) = auth_client
        .create_authentication_challenge(Request::new(req))
        .await
        .map_or_else(
            |_status| Err(error::AuthClientError::ConnectionFailed),
            |response| {
                let response = response.into_inner();
                Ok((response.auth_id, response.c))
            },
        )?;

    let auth_challenge_data = AuthChallengeData {
        auth_id,
        s: crypto::calculate_challenge(c),
    };

    auth_client
        .verify_authentication(Request::new(auth_challenge_data.into()))
        .await
        .map_or_else(
            |_status| Err(error::AuthClientError::ConnectionFailed),
            |response| {
                Ok(SessionData {
                    session_id: response.into_inner().session_id,
                })
            },
        )
}
