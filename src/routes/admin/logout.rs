use actix_web::{post, HttpResponse};
use actix_web_flash_messages::FlashMessage;

use crate::{
    session_state::TypedSession,
    utils::{e500, see_other},
};

#[post("/admin/logout")]
pub async fn admin_logout(session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        Ok(see_other("/login"))
    } else {
        session.log_out();
        FlashMessage::info("You have successfully logged out.").send();
        Ok(see_other("/login"))
    }
}
