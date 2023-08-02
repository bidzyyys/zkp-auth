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
const ZKP_G_ENV: &str = "ZKP_G";
const ZKP_H_ENV: &str = "ZKP_H";
const ZKP_Q_ENV: &str = "ZKP_Q";

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
        log::info!("Handling register request: {:?}", request);

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
        log::info!(
            "Handling create authentication challenge request: {:?}",
            request
        );

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
        log::info!("Handling verify authentication request: {:?}", request);

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

fn read_env_var(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("Missing env variable: {:?}", name))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let server_address = read_env_var(SERVER_LISTENING_ADDR_ENV)
        .parse()
        .expect(format!("Invalid value set for {:?}", SERVER_LISTENING_ADDR_ENV).as_str());

    let zkp_g = read_env_var(ZKP_G_ENV)
        .parse()
        .expect(format!("Invalid value set for {:?}", ZKP_G_ENV).as_str());
    let zkp_h = read_env_var(ZKP_H_ENV)
        .parse()
        .expect(format!("Invalid value set for {:?}", ZKP_H_ENV).as_str());
    let zkp_q = read_env_var(ZKP_Q_ENV)
        .parse()
        .expect(format!("Invalid value set for {:?}", ZKP_Q_ENV).as_str());

    log::info!(
        target: LOG_TARGET,
        "Starting auth_service on address: {:?}",
        server_address
    );

    let zkp_ctx = chaum_pedersen::Context::new(zkp_g, zkp_h, zkp_q);
    let auth_actor = Arc::new(Mutex::new(AuthActor::new(zkp_ctx)));

    let auth_service = AuthService::new(auth_actor);

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(server_address)
        .await?;
    Ok(())
}
