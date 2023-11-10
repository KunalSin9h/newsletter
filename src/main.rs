use newsletter::configuration::get_configuration;
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

    println!("Started server at post {}", configuration.application.port);

    tokio::select! {
        // TODO consume error
        _ = app_run => {
            println!("Application (API) exited");
        },
        _ = worker_run => {
            println!("Worker exited");
        },
    };

    Ok(())
}
