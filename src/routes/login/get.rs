use actix_web::{get, http::header::ContentType, HttpResponse};
use actix_web_flash_messages::{IncomingFlashMessages, Level};

#[get("/login")]
pub async fn login_form(flash_message: IncomingFlashMessages) -> HttpResponse {
    let mut message_str = String::new();

    for m in flash_message.iter() {
        let mut class = "error";
        if m.level() == Level::Info {
            class = "info"
        }

        message_str.push_str(format!("<p class='{}'><i>{}</i></p>", class, m.content()).as_str());
    }

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(get_login_html(message_str))
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
                .info {{
                    color: green;
                    font-weight: bold;
                }}
            </style>
        </head>
        <body>
            {error_message}
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
