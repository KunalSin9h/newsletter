use actix_web::{post, HttpResponse};
use actix_web_flash_messages::FlashMessage;

use crate::{session_state::TypedSession, utils::see_other};

#[post("/logout")]
pub async fn admin_logout(session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    session.log_out();
    FlashMessage::info("You have successfully logged out.").send();
    Ok(see_other("/login"))
}
