use tracing_subscriber::fmt::{self};
use tracing_subscriber::layer::SubscriberExt;
use std::io::stdout;
use std::fs::OpenOptions;
use tracing::{info, warn, error, debug, trace};


fn init_log() {

    let log_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open("app.log") {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Cannot open file app.log: {}", e);
                return;
            }
        };

    let file_layer = fmt::layer()
        .with_writer(log_file)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_level(true);

    let console_layer = fmt::layer()
        .with_writer(|| stdout())
        .with_file(true)
        .with_line_number(true)
        .with_ansi(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_level(true);

    let subscriber = tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer);

    tracing::subscriber::set_global_default(subscriber).expect("设置 subscriber 失败");

    info!("This is an info message.");
    warn!("This is a warning message.");

    error!("This is an error message.");
    debug!("This is a debug message.");
    trace!("This is a trace message.");
}

fn main() {
    init_log();
}
