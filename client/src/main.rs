use zkp_auth::auth_client::AuthClient;

pub mod zkp_auth {
    tonic::include_proto!("zkp_auth");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _client = AuthClient::connect("http://[::1]:8080").await?;
    println!("Hello, world!");
    Ok(())
}
