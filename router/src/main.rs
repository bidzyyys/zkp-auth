use actix_web::{
    middleware,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer,
};

use std::env;
use std::sync::Mutex;

use tonic::transport::Channel;

use zkp_auth::auth_client::AuthClient;

pub mod zkp_auth {
    tonic::include_proto!("zkp_auth");
}

mod auth;

const AUTH_SERVICE_URI_ENV: &str = "AUTH_SERVICE_URI";
const HTTP_PORT_ENV: &str = "HTTP_PORT";

const LOG_TARGET: &str = "router";

async fn register(auth_client: Data<Mutex<AuthClient<Channel>>>, req: HttpRequest) -> HttpResponse {
    log::info!("Handling request: {:?}", req);

    let register_data = auth::RegisterData {
        username: "bidzyyys".into(),
        y1: 1,
        y2: 2,
    };

    match auth::register(
        &mut auth_client
            .lock()
            .expect("Auth client must be available in `register` handler"),
        register_data,
    )
    .await
    {
        Ok(register_data) => register_data.into(),
        Err(e) => e.into(),
    }
}

async fn login(auth_client: Data<Mutex<AuthClient<Channel>>>, req: HttpRequest) -> HttpResponse {
    log::info!("Handling request: {:?}", req);
    let login_data = auth::LoginData {
        username: "bidzyyys".into(),
        r1: 1,
        r2: 8,
    };

    match auth::login(
        &mut auth_client
            .lock()
            .expect("Auth client must be available in `login` handler"),
        login_data,
    )
    .await
    {
        Ok(session_data) => session_data.into(),
        Err(e) => e.into(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let auth_service_address: String = env::var(AUTH_SERVICE_URI_ENV)
        .expect(format!("Missing env variable: {:?}", AUTH_SERVICE_URI_ENV).as_str())
        .parse()
        .expect(format!("Invalid value set for {:?}", AUTH_SERVICE_URI_ENV).as_str());

    let http_port = env::var(HTTP_PORT_ENV)
        .expect(format!("Missing env variable: {:?}", HTTP_PORT_ENV).as_str())
        .parse()
        .expect(format!("Invalid value set for {:?}", AUTH_SERVICE_URI_ENV).as_str());

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
        let auth_client = Data::new(Mutex::new(auth_client.clone()));
        App::new()
            .app_data(Data::new(auth_client)) // add thread-local auth_client
            // enable logger
            .wrap(middleware::Logger::default())
            // register `register` handler
            .service(web::resource("/register").to(register))
            // register `login` handler
            .service(web::resource("/login").to(login))
    })
    .bind(("0.0.0.0", http_port))
    .map_err(Box::new)?
    .run()
    .await?;

    Ok(())
}
