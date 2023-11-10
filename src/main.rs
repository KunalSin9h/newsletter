use newsletter::configuration::get_configuration;
use newsletter::idempotency::run_idempotency_worker;
use newsletter::issue_delivery_workers::run_worker_until_stopped;
use newsletter::startup::Application;
use newsletter::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("newsletter".into(), "info".into());
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let app = Application::build(configuration.clone()).await?;
    let app_run = tokio::spawn(app.run_until_stopped());
    let worker_run = tokio::spawn(run_worker_until_stopped(configuration.clone()));

    let idempotency_worker = tokio::spawn(run_idempotency_worker(configuration.clone()));

    println!("Started server at post {}", configuration.application.port);

    tokio::select! {
        // TODO consume error
        _ = app_run => {
            println!("Application (API) exited");
        },
        _ = worker_run => {
            println!("Worker exited");
        },
        _ = idempotency_worker => {
            println!("Idempotency worker exited");
        },
    };

    Ok(())
}
