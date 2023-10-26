use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::home;
use crate::routes::login;
use crate::routes::{confirm, health_check, publish_newsletter, subscribe};
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::{web, web::Data, App, HttpServer};
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Pool, Postgres};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    pub port: u16,
    pub server: Server,
}

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_configuration_pool(&configuration.database);

        let sender = configuration
            .email_client
            .sender()
            .expect("Invalid sender email");

        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender,
            configuration.email_client.authorization_token,
            timeout,
        )
        .expect("Failed to parse email server url");

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        // A tcp listener for listening on port
        let lst = TcpListener::bind(address)?;
        let port = lst.local_addr().unwrap().port();
        let server = run(
            lst,
            connection_pool,
            email_client,
            configuration.application.base_url,
            HmacSecret(configuration.application.hmac_secret),
        )?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> std::io::Result<()> {
        self.server.await
    }
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: HmacSecret,
) -> Result<Server, std::io::Error> {
    let message_framework = {
        let message_store =
            CookieMessageStore::builder(Key::from(hmac_secret.0.expose_secret().as_bytes()))
                .build();
        FlashMessagesFramework::builder(message_store).build()
    };

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default()) // Use this middleware
            .wrap(message_framework.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/subscription", web::post().to(subscribe))
            .route("/subscription/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            .service(home)
            .service(login::login_form)
            .service(login::login)
            .app_data(Data::new(db_pool.clone()))
            .app_data(Data::new(email_client.clone()))
            .app_data(Data::new(base_url.clone()))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub fn get_configuration_pool(db: &DatabaseSettings) -> Pool<Postgres> {
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(db.with_db())
}
