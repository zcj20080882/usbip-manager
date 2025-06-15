mod usbipd;


use tracing_subscriber::fmt::{self};
use tracing_subscriber::layer::SubscriberExt;
use std::io::stdout;
use std::fs::OpenOptions;
use tracing::{info, warn, error, debug, trace};
use usbipd::{USBIPDError, UsbDevice, commands, runner};


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
}

#[tokio::main]
async fn main()  {
    init_log();
    let devices = commands::get_all_devices().await;
    match devices {
        Ok(devices) => {
            info!("Found {} USB devices:", devices.len());
            for device in devices {
                info!("{:?}", device);
            }
        },
        Err(e) => {
            error!("Error retrieving USB devices: {}", e);
        }
    }
    let bind_result = commands::bind_device("0403:6001", false).await;
    match bind_result {
        Ok(success) => {
            if success {
                info!("Device bound successfully.");
            } else {
                warn!("Device binding was not successful.");
            }
        },
        Err(e) => {
            error!("Error binding device: {}", e);
        }
    }
    let unbind_result = commands::unbind_device(Some("0403:6001")).await;
    match unbind_result {
        Ok(success) => {
            if success {
                info!("Device unbound successfully.");
            } else {
                warn!("Device unbinding was not successful.");
            }
        },
        Err(e) => {
            error!("Error unbinding device: {}", e);
        }
    }
}
