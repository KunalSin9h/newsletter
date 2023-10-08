use super::{error_chain_printer, StoreTokenError};
use actix_web::http::StatusCode;

pub enum SubscribeError {
    ValidationError(String),
    PoolError(sqlx::Error),
    InsertSubscriberError(sqlx::Error),
    TransactionCommitError(sqlx::Error),
    StoreTokenError(StoreTokenError),
    SendEmailError(reqwest::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_printer(self, f)
    }
}

impl std::fmt::Display for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValidationError(e) => write!(f, "{}", e),
            Self::PoolError(_) => write!(f, "Failed to connect to Postgres Database"),
            Self::InsertSubscriberError(_) => {
                write!(f, "Failed to inset new subscriber to database")
            }
            Self::TransactionCommitError(_) => write!(f, "Failed to commit transition"),
            Self::StoreTokenError(_) => {
                write!(f, "Failed to store confirmation token for a new subscriber")
            }
            Self::SendEmailError(_) => {
                write!(f, "Failed to send confirmation token")
            }
        }
    }
}

impl std::error::Error for SubscribeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ValidationError(_) => None,
            Self::PoolError(e) => Some(e),
            Self::InsertSubscriberError(e) => Some(e),
            Self::TransactionCommitError(e) => Some(e),
            Self::StoreTokenError(e) => Some(e),
            Self::SendEmailError(e) => Some(e),
        }
    }
}

impl actix_web::ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<StoreTokenError> for SubscribeError {
    fn from(value: StoreTokenError) -> Self {
        Self::StoreTokenError(value)
    }
}

impl From<reqwest::Error> for SubscribeError {
    fn from(value: reqwest::Error) -> Self {
        Self::SendEmailError(value)
    }
}

impl From<String> for SubscribeError {
    fn from(value: String) -> Self {
        Self::ValidationError(value)
    }
}
