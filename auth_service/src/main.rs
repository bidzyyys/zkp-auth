use std::env;
use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};
use zkp::chaum_pedersen;
use zkp_auth::{
    auth_server::{Auth, AuthServer},
    AuthenticationAnswerRequest, AuthenticationAnswerResponse, AuthenticationChallengeRequest,
    AuthenticationChallengeResponse, RegisterRequest, RegisterResponse,
};

use auth::{error::AuthActorError, AuthActor};
mod auth;
mod repository;

pub mod zkp_auth {
    tonic::include_proto!("zkp_auth");
}

const SERVER_LISTENING_ADDR_ENV: &str = "SERVER_LISTENING_ADDR";
const LOG_TARGET: &str = "auth_service";

type Actor = Arc<Mutex<AuthActor>>;

pub struct AuthService {
    auth_actor: Actor,
}

impl AuthService {
    pub fn new(auth_actor: Actor) -> Self {
        Self { auth_actor }
    }
}

#[tonic::async_trait]
impl Auth for AuthService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let request = request.into_inner();
        let actor = &mut self
            .auth_actor
            .lock()
            .expect("Auth Actor must be available");
        match actor.register(request.user, request.y1, request.y2) {
            Ok(_) => Ok(Response::new(RegisterResponse {})),
            Err(e) => {
                let (code, msg) = match e {
                    AuthActorError::UserAlreadyRegistered => {
                        (tonic::Code::AlreadyExists, "UserAlreadyRegistered")
                    }
                    _ => (tonic::Code::Internal, "Unexpected server error"),
                };

                Err(Status::new(code, msg))
            }
        }
    }

    async fn create_authentication_challenge(
        &self,
        request: Request<AuthenticationChallengeRequest>,
    ) -> Result<Response<AuthenticationChallengeResponse>, Status> {
        let request = request.into_inner();
        let actor = &mut self
            .auth_actor
            .lock()
            .expect("Auth Actor must be available");
        match actor.create_authentication_challenge(request.user, request.r1, request.r2) {
            Ok(auth::AuthChallenge { auth_id, c }) => {
                Ok(Response::new(AuthenticationChallengeResponse {
                    auth_id,
                    c,
                }))
            }
            Err(e) => {
                let (code, msg) = match e {
                    AuthActorError::UserNotFound => (tonic::Code::NotFound, "User not found"),
                    _ => (tonic::Code::Internal, "Unexpected server error"),
                };

                Err(Status::new(code, msg))
            }
        }
    }

    async fn verify_authentication(
        &self,
        request: Request<AuthenticationAnswerRequest>,
    ) -> Result<Response<AuthenticationAnswerResponse>, Status> {
        let request = request.into_inner();
        let actor = &mut self
            .auth_actor
            .lock()
            .expect("Auth Actor must be available");
        match actor.verify_authentication(request.auth_id, request.s) {
            Ok(auth::SessionDetails { session_id }) => {
                Ok(Response::new(AuthenticationAnswerResponse { session_id }))
            }
            Err(e) => {
                let (code, msg) = match e {
                    AuthActorError::UserNotFound => (tonic::Code::NotFound, "User not found"),
                    AuthActorError::AuthChallengeNotFound => {
                        (tonic::Code::NotFound, "Challenge not found")
                    }
                    AuthActorError::AuthChallengeFailed => (
                        tonic::Code::Unauthenticated,
                        "Negative challenge verification",
                    ),
                    AuthActorError::ZKPMathError => (
                        tonic::Code::ResourceExhausted,
                        "Arithmetic overflow during challenge verification",
                    ),
                    _ => (tonic::Code::Internal, "Unexpected server error"),
                };

                Err(Status::new(code, msg))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let server_address = env::var(SERVER_LISTENING_ADDR_ENV)
        .expect(format!("Missing env variable: {:?}", SERVER_LISTENING_ADDR_ENV).as_str())
        .parse()
        .expect(format!("Invalid value set for {:?}", SERVER_LISTENING_ADDR_ENV).as_str());

    log::info!(
        target: LOG_TARGET,
        "Starting auth_service on address: {:?}",
        server_address
    );

    let zkp_ctx = chaum_pedersen::Context::new(1, 1, 1);
    let auth_actor = Arc::new(Mutex::new(AuthActor::new(zkp_ctx)));

    let auth_service = AuthService::new(auth_actor);

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(server_address)
        .await?;
    Ok(())
}
