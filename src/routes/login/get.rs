use actix_web::{get, http::header::ContentType, HttpResponse};
use actix_web_flash_messages::{IncomingFlashMessages, Level};

#[get("/login")]
pub async fn login_form(flash_message: IncomingFlashMessages) -> HttpResponse {
    let mut error_html = String::new();

    for m in flash_message.iter().filter(|m| m.level() == Level::Error) {
        error_html.push_str(format!("<p><i>{}</i></p>\n", m.content()).as_str());
    }

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(get_login_html(error_html))
}

fn get_login_html(error_message: String) -> String {
    format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
        <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Login</title>
            <style>
                .error {{
                    color: red;
                    font-weight: bold;
                }}
            </style>
        </head>
        <body>
            <span class="error">{error_message}</span>
            <form action="/login" method="post">
                <label>Username
                    <input
                        type="text"
                        placeholder="Enter Username"
                        name="username"
                    >
                </label>
                <label>Password
                    <input
                        type="password"
                        placeholder="Enter Password"
                        name="password"
                    >
                </label>
                <button type="submit">Login</button>
            </form>
        </body>
    </html>
    "#
    )
}
