use crate::locations::get_default_storage_dir;
use chrono::Utc;
use std::path::PathBuf;
use tracing_appender;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, Registry};

pub(crate) fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_file = get_log_location();
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .unwrap();

    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy();
    // The subscriber should be an append-only file

    let formatter = BunyanFormattingLayer::new("godata".into(), non_blocking);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatter);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    return guard;
}

fn get_log_location() -> PathBuf {
    let storage_dir = get_default_storage_dir().unwrap();
    let log_dir = storage_dir.join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();
    let timestamp = Utc::now().format("%Y-%m-%d-%H-%M-%S");
    let log_file = log_dir.join(format!("godata-{}.log", timestamp));
    clean_logfiles(&log_dir);
    log_file
}

fn clean_logfiles(log_dir: &PathBuf) {
    // Logfiles from more than 30 days ago are deleted
    let files = std::fs::read_dir(log_dir).unwrap();
    for file in files {
        let file = file.unwrap();
        let metadata = file.metadata().unwrap();
        let modified = metadata.modified().unwrap();
        // convert the modified time to a DateTime<Utc>
        let modified: chrono::DateTime<Utc> = chrono::DateTime::from(modified);
        let duration = Utc::now().signed_duration_since(modified);
        if duration.num_days() > 30 {
            std::fs::remove_file(file.path()).unwrap();
        }
    }
}
