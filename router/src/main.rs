use actix_web::{middleware, post, web, web::Data, App, HttpResponse, HttpServer};

use std::env;
use std::sync::Mutex;

use auth::{LoginData, RegisterCalculateRequest, RegisterData};
use zkp::chaum_pedersen;

use tonic::transport::Channel;

use zkp_auth::auth_client::AuthClient;

pub mod zkp_auth {
    tonic::include_proto!("zkp_auth");
}

mod auth;

const AUTH_SERVICE_URI_ENV: &str = "AUTH_SERVICE_URI";
const HTTP_PORT_ENV: &str = "HTTP_PORT";
const ZKP_G_ENV: &str = "ZKP_G";
const ZKP_H_ENV: &str = "ZKP_H";
const ZKP_Q_ENV: &str = "ZKP_Q";

const LOG_TARGET: &str = "router";

struct AppState {
    auth_client: Mutex<AuthClient<Channel>>,
    zkp: chaum_pedersen::ChaumPedersenProtocol,
}

#[post("/register/calculate")]
async fn register_calculate(
    app_state: Data<AppState>,
    data: web::Json<RegisterCalculateRequest>,
) -> HttpResponse {
    log::info!("Handling register calculate request: {:?}", data);

    match auth::register_calculate(&app_state.zkp, &data.into_inner()) {
        Ok(register_data) => register_data.into(),
        Err(e) => e.into(),
    }
}

#[post("/register")]
async fn register(app_state: Data<AppState>, data: web::Json<RegisterData>) -> HttpResponse {
    log::info!("Handling register request: {:?}", data);

    match auth::register(
        &mut app_state
            .auth_client
            .lock()
            .expect("Auth client must be available in `register` handler"),
        data.into_inner(),
    )
    .await
    {
        Ok(register_data) => register_data.into(),
        Err(e) => e.into(),
    }
}

#[post("/login")]
async fn login(app_state: Data<AppState>, data: web::Json<LoginData>) -> HttpResponse {
    log::info!("Handling login request: {:?}", data);

    match auth::login(
        &mut app_state
            .auth_client
            .lock()
            .expect("Auth client must be available in `login` handler"),
        &app_state.zkp,
        data.into_inner(),
    )
    .await
    {
        Ok(session_data) => session_data.into(),
        Err(e) => e.into(),
    }
}

fn read_env_var(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("Missing env variable: {:?}", name))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let auth_service_address: String = read_env_var(AUTH_SERVICE_URI_ENV)
        .parse()
        .expect(format!("Invalid value set for {:?}", AUTH_SERVICE_URI_ENV).as_str());

    let http_port: u16 = read_env_var(HTTP_PORT_ENV)
        .parse()
        .expect(format!("Invalid value set for {:?}", AUTH_SERVICE_URI_ENV).as_str());

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
        "Connecting to auth_service on address: {:?}",
        auth_service_address
    );

    let auth_client: AuthClient<Channel> = AuthClient::connect(auth_service_address)
        .await
        .expect("Failed to connect to auth_service");

    HttpServer::new(move || {
        // Create thread-local auth_client
        let app_state = AppState {
            auth_client: Mutex::new(auth_client.clone()),
            zkp: chaum_pedersen::ChaumPedersenProtocol::new(chaum_pedersen::Context::new(
                zkp_g, zkp_h, zkp_q,
            )),
        };
        let json_config = web::JsonConfig::default()
            .limit(4096)
            .error_handler(|err, _req| {
                // create custom error response
                actix_web::error::InternalError::from_response(
                    err,
                    HttpResponse::Conflict().finish(),
                )
                .into()
            });

        App::new()
            .app_data(Data::new(app_state)) // add thread-local auth_client
            .app_data(json_config)
            // enable logger
            .wrap(middleware::Logger::default())
            // register `register_calculate` handler
            .service(register_calculate)
            // register `register_calculate` handler
            .service(register)
            // register `login` handler
            .service(login)
    })
    .bind(("0.0.0.0", http_port))
    .map_err(Box::new)?
    .run()
    .await?;

    Ok(())
}
