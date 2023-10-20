use actix_web::{get, http::header::ContentType, web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: Option<String>,
}

#[get("/login")]
pub async fn login_form(query: web::Query<QueryParams>) -> HttpResponse {
    let error_html = match query.0.error {
        None => "".into(),
        Some(error_message) => format!(
            "<p><i>{}</i></p>",
            // PREVENTION FROM XSS (CROSS SITE SCRIPTING)
            // OWASP Advises to HTML Entity Encode the UNTRUSTED DATA
            htmlescape::encode_minimal(&error_message)
        ),
    };

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
