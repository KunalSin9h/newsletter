use actix_web::{get, http::header::ContentType, HttpResponse};
use actix_web_flash_messages::{IncomingFlashMessages, Level};

#[get("/newsletters")]
pub async fn issue_page(
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
        .body(get_issue_page_html(message_str)))
}

fn get_issue_page_html(message: String) -> String {
    let idempotency_key = uuid::Uuid::new_v4().to_string();

    format!(
        r#"
        <html>
            <head>
                <title>New newsletter issue</title>
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
                <form action="/admin/newsletters" method="post">
                    <label>Title
                        <input
                            type="text"
                            placeholder="Title"
                            name="title"
                        >
                    </label>

                    <label>Text
                        <input
                            type="text"
                            placeholder="Text content"
                            name="text"
                        >
                    </label>

                    <label>HTML
                        <input
                            type="text"
                            placeholder="Html content"
                            name="html"
                        >
                    </label>

                    <input 
                        hidden
                        type="text"
                        name="idempotency_key"
                        value="{idempotency_key}"
                    >

                    <button type="submit">Send</button>
                </form>
            </body>
        </html>
    "#
    )
}
