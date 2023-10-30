use actix_web::{post, web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::{
    session_state::TypedSession,
    utils::{e500, see_other}, routes::get_username, authentication::{validate_credential, AuthError, Credentials},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

#[post("/admin/password")]
pub async fn change_password(
    form: web::Form<FormData>,
    session: TypedSession,
    pool: web::Data<PgPool>
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = if let Some(user_id) = session.get_user_id().map_err(e500)? {
        user_id
    } else {
         return Ok(see_other("/login"));
    };
    
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error(
            "You entered two different new passwords - the field values must match.",
        )
        .send();
        return Ok(see_other("/admin/password"));
    }

    let username = get_username(user_id, &pool).await.map_err(e500)?;

    let credential = Credentials {
        username,
        password: form.0.current_password,
    };

    if let Err(e) = validate_credential(credential, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::UnexpectedError(_) => Err(e500(e).into()),
        }
    }

    let new_password_length = form.0.new_password.expose_secret().len();

    if new_password_length < 12 || new_password_length > 128 {
        FlashMessage::error("New password is invalid, it should be between 12 and 128 characters.").send();
        return Ok(see_other("/admin/password"));
    }

    todo!()
}
