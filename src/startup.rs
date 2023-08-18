use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::{PgPool, Postgres, Pool};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::configuration::{Settings, DatabaseSettings};
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};

pub struct Application {
    pub port: u16,
    pub server: Server,
}

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
        let lst = TcpListener::bind(&address)?;
        let port = lst.local_addr().unwrap().port();
        let server =  run(lst, connection_pool, email_client)?;

        Ok(Self{port, server})
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
) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default()) // Use this middleware
            .route("/health_check", web::get().to(health_check))
            .route("/subscription", web::post().to(subscribe))
            .app_data(connection.clone())
            .app_data(email_client.clone())
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