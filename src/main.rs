use koru::api::RestApi;
use koru::application::app::Application;
use koru::configuration::get_configuration;
use koru::infrastructure::event_bus::EventBusImpl;
use koru::infrastructure::store::StoreImpl;
use koru::utils::telemetry::{get_subscriber, init_subscriber};
use koru::worker::Worker;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging
    let subscriber = get_subscriber("koru".into(), "sqlx=error,info".into(), std::io::stdout);
    init_subscriber(subscriber);
    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");
    println!("{:?}", configuration);

    let store = Arc::new(StoreImpl::build(&configuration.database).await?);
    let (event_bus, event_listener) = EventBusImpl::build(&configuration.event_bus).await?;
    // the None for argon_memory wil make it use the default 16MB
    let app = Application::build(&configuration.application, store.clone(), event_bus, None)?;

    let api = RestApi::build(&configuration.api, app).await?;
    let worker = Worker::build(&configuration.application, event_listener, store).await?;

    // Start
    let worker = tokio::spawn(worker.run());
    let app = tokio::spawn(api.run());
    tokio::select! {
        o = app => report_exit("API", o),
        o = worker =>  report_exit("Worker", o),
    }
    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}
