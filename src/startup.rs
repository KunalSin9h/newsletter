use crate::authentication::reject_anonymous_user;
use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::login;
use crate::routes::{admin, home};
use crate::routes::{confirm, health_check, subscribe};
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::{web, web::Data, App, HttpServer};
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use actix_web_lab::middleware::from_fn;
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
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
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
            configuration.redis_uri,
        )
        .await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> std::io::Result<()> {
        self.server.await
    }
}

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: HmacSecret,
    redis_uri: Secret<String>,
) -> Result<Server, anyhow::Error> {
    let secret_key = Key::from(hmac_secret.0.expose_secret().as_bytes());
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;

    let message_framework = {
        let message_store = CookieMessageStore::builder(secret_key.clone()).build();
        FlashMessagesFramework::builder(message_store).build()
    };

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default()) // Use this middleware
            .wrap(message_framework.clone())
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .route("/health_check", web::get().to(health_check))
            .route("/subscription", web::post().to(subscribe))
            .route("/subscription/confirm", web::get().to(confirm))
            .service(home)
            .service(login::login_form)
            .service(login::login)
            .service(
                web::scope("/admin")
                    .wrap(from_fn(reject_anonymous_user)) // Auth middleware
                    .service(admin::admin_dashboard)
                    .service(admin::change_password_form)
                    .service(admin::change_password)
                    .service(admin::admin_logout)
                    .service(admin::issue_page)
                    .service(admin::newsletter_issue),
            )
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
