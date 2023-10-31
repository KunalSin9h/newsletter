use actix_web::{get, http::header::ContentType, HttpResponse};
use actix_web_flash_messages::{IncomingFlashMessages, Level};

#[get("/admin/password")]
pub async fn change_password_form(
    flash_message: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut message_str = String::new();
    for m in flash_message.iter() {
        let mut class = "error";
        if m.level() == Level::Info {
            class = "info"
        }

        message_str.push_str(format!("<p class='{}'><i>{}</i></p>", class, m.content()).as_str());
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
                .info {{
                    color: green;
                    font-weight: bold;
                }}
            </style>
        </head>
        <body>
            {message}
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
