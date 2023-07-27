use actix_web::HttpResponse;

pub enum AuthClientError {
    ConnectionFailed,
    UnexpectedResponse,
    UserAlreadyRegistered,
}

impl From<AuthClientError> for HttpResponse {
    fn from(val: AuthClientError) -> Self {
        match val {
            AuthClientError::ConnectionFailed => HttpResponse::InternalServerError().into(),
            AuthClientError::UnexpectedResponse => HttpResponse::InternalServerError().into(),
            AuthClientError::UserAlreadyRegistered => HttpResponse::Forbidden().into(),
        }
    }
}
