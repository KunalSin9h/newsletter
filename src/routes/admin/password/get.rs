use actix_web::{get, http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

use crate::session_state::TypedSession;
use crate::utils::{e500, see_other};

#[get("/admin/password")]
pub async fn change_password_form(
    session: TypedSession,
    flash_message: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }

    let mut message_str = String::new();
    for m in flash_message.iter() {
        message_str.push_str(format!("<p><i>{}</i></p>", m.content()).as_str());
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(get_change_password_html(message_str)))
}

fn get_change_password_html(message: String) -> String {
    format!(
        r#"<!DOCTYPE html>
    <html lang="en">
        <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Change Password</title>
            <style>
                .error {{
                    color: red;
                    font-weight: bold;
                }}
            </style>
        </head>
        <body>
            <span class="error">{message}</span>
            <form action="/admin/password" method="post">
                <label>Current password
                    <input
                        type="password"
                        placeholder="Enter current password"
                        name="current_password"
                    >
                </label>
                <br>
                <label>New password
                    <input
                    type="password"
                    placeholder="Enter new password"
                    name="new_password"
                    >
                </label>
                <br>
                <label>Confirm new password
                    <input
                        type="password"
                        placeholder="Type the new password again"
                        name="new_password_check"
                    >
                </label>
                <br>
                <button type="submit">Change password</button>
            </form>
            <p><a href="/admin/dashboard">&lt;- Back</a></p>
        </body>
    </html>"#
    )
}
