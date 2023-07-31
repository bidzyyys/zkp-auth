use actix_web::HttpResponse;

use tonic::Code;

use zkp::ZKPError;

pub enum AuthClientError {
    AuthenticationFailure,
    ConnectionFailed,
    InternalServerError,
    ServerMathError,
    UserAlreadyRegistered,
    UserNotFound,
    UnexpectedResponse,
}

impl From<ZKPError> for AuthClientError {
    fn from(_val: ZKPError) -> Self {
        AuthClientError::ServerMathError
    }
}

impl From<Code> for AuthClientError {
    fn from(val: Code) -> Self {
        match val {
            Code::AlreadyExists => AuthClientError::UserAlreadyRegistered,
            Code::NotFound => AuthClientError::UserNotFound,
            Code::ResourceExhausted => AuthClientError::ServerMathError,
            Code::Unauthenticated => AuthClientError::AuthenticationFailure,
            _ => AuthClientError::InternalServerError,
        }
    }
}

impl From<AuthClientError> for HttpResponse {
    fn from(val: AuthClientError) -> Self {
        match val {
            AuthClientError::AuthenticationFailure => HttpResponse::Forbidden().into(),
            AuthClientError::ConnectionFailed => HttpResponse::InternalServerError().into(),
            AuthClientError::InternalServerError => HttpResponse::InternalServerError().into(),
            AuthClientError::ServerMathError => HttpResponse::InsufficientStorage().into(),
            AuthClientError::UserAlreadyRegistered => HttpResponse::NotAcceptable().into(),
            AuthClientError::UserNotFound => HttpResponse::NotAcceptable().into(),
            AuthClientError::UnexpectedResponse => HttpResponse::InternalServerError().into(),
        }
    }
}
