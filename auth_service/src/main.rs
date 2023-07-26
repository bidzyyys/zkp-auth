use std::env;
use tonic::{transport::Server, Request, Response, Status};
use zkp_auth::{
    auth_server::{Auth, AuthServer},
    AuthenticationAnswerRequest, AuthenticationAnswerResponse, AuthenticationChallengeRequest,
    AuthenticationChallengeResponse, RegisterRequest, RegisterResponse,
};

pub mod zkp_auth {
    tonic::include_proto!("zkp_auth");
}

const SERVER_LISTENING_ADDR_ENV: &str = "SERVER_LISTENING_ADDR";
const LOG_TARGET: &str = "auth_service";

#[derive(Debug, Default)]
pub struct AuthService {}

#[tonic::async_trait]
impl Auth for AuthService {
    async fn register(
        &self,
        _request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        unimplemented!();
    }

    async fn create_authentication_challenge(
        &self,
        _request: Request<AuthenticationChallengeRequest>,
    ) -> Result<Response<AuthenticationChallengeResponse>, Status> {
        unimplemented!();
    }

    async fn verify_authentication(
        &self,
        _request: Request<AuthenticationAnswerRequest>,
    ) -> Result<Response<AuthenticationAnswerResponse>, Status> {
        unimplemented!();
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

    let auth_service = AuthService::default();

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(server_address)
        .await?;
    Ok(())
}
