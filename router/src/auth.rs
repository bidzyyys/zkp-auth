use actix_web::HttpResponse;

use tonic::{transport::Channel, Code, Request};

use crate::{
    zkp_auth::{AuthenticationAnswerRequest, AuthenticationChallengeRequest, RegisterRequest},
    AuthClient,
};

use error::AuthClientError;

use zkp::chaum_pedersen::ChaumPedersenProtocol;

mod error;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RegisterCalculateRequest {
    user: String,
    x: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RegisterCalculateResponse {
    user: String,
    y1: i64,
    y2: i64,
}

impl From<RegisterCalculateResponse> for HttpResponse {
    fn from(val: RegisterCalculateResponse) -> Self {
        HttpResponse::Created().body(
            serde_json::to_string(&val)
                .expect("`RegisterCalculateResponse` is serializable to json"),
        )
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
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

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct LoginData {
    pub user: String,
    pub x: i64,
    pub k: i64,
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

pub fn register_calculate(
    zkp: &ChaumPedersenProtocol,
    data: &RegisterCalculateRequest,
) -> Result<RegisterCalculateResponse, AuthClientError> {
    let (y1, y2) = zkp.calculate_registration_data(data.x)?;
    Ok(RegisterCalculateResponse {
        user: data.user.clone(),
        y1,
        y2,
    })
}

pub async fn register(
    auth_client: &mut AuthClient<Channel>,
    register_data: RegisterData,
) -> Result<RegisterData, AuthClientError> {
    auth_client
        .register(Request::new(register_data.clone().into()))
        .await
        .map_or_else(
            |status| Err(status.code().into()),
            |_response| Ok(register_data),
        )
}

pub async fn login(
    auth_client: &mut AuthClient<Channel>,
    zkp: &ChaumPedersenProtocol,
    login_data: LoginData,
) -> Result<SessionData, AuthClientError> {
    let (r1, r2) = zkp.calculate_registration_data(login_data.k)?;
    let req = AuthenticationChallengeRequest {
        user: login_data.user.clone(),
        r1,
        r2,
    };

    let (auth_id, c) = auth_client
        .create_authentication_challenge(Request::new(req))
        .await
        .map_or_else(
            |status| Err(<Code as Into<AuthClientError>>::into(status.code())),
            |response| {
                let response = response.into_inner();
                Ok((response.auth_id, response.c))
            },
        )?;

    let auth_challenge_data = AuthChallengeData {
        auth_id,
        s: zkp.calculate_challenge(login_data.k, c, login_data.x)?,
    };

    auth_client
        .verify_authentication(Request::new(auth_challenge_data.into()))
        .await
        .map_or_else(
            |status| Err(status.code().into()),
            |response| {
                Ok(SessionData {
                    session_id: response.into_inner().session_id,
                })
            },
        )
}
