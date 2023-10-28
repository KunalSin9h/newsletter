use std::future::{Ready, ready};

use actix_session::{Session, SessionExt, SessionInsertError, SessionGetError};
use actix_web::{FromRequest, HttpRequest};

pub struct TypedSession(Session);

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";

    pub fn renew(&self) {
        self.0.renew(); 
    }

    pub fn insert_user_id(&self, user_id: uuid::Uuid) -> Result<(), SessionInsertError> {
        self.0.insert(Self::USER_ID_KEY, user_id)
    }

    pub fn get_user_id(&self) -> Result<Option<uuid::Uuid>, SessionGetError> {
        self.0.get(Self::USER_ID_KEY)
    }
}

// Anyone that implements FromRequest is called Extractor
// and we are making TypedSession an extractor
impl FromRequest for TypedSession {
    // We return the same error as returned 
    // by the implementation of 'impl FromRequest for Session'
    type Error = <Session as FromRequest>::Error;

    // Rust does not yet support the `async` syntax in traits.
    // From request expects a `Future` as return type to allow for extractors
    // that need to perform asynchronous operations (e.g. a HTTP call)
    // We do not have a `Future`, because we don't perform any I/O,
    // so we wrap `TypedSession` into `Ready` to convert it into a `Future` that
    // resolves to the wrapped value the first time it's polled by the executor.
    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}
