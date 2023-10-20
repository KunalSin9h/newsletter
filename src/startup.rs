use actix_web::dev::Server;
use actix_web::{web, web::Data, App, HttpServer};
use secrecy::Secret;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Pool, Postgres};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use uuid::Uuid;

use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::home;
use crate::routes::login;
use crate::routes::{confirm, health_check, publish_newsletter, subscribe};

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
            HmacSecret(Secret::new(Uuid::new_v4().to_string())),
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
    secret: HmacSecret,
) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default()) // Use this middleware
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
            .app_data(Data::new(secret.clone()))
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
