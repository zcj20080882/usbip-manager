mod usbipd;

use tracing_subscriber::field::debug;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::layer::SubscriberExt;
use std::io::stdout;
use std::fs::OpenOptions;
use std::thread::sleep;
use std::time::Duration;
#[allow(unused_imports)]
use tracing::{info, warn, error, debug, trace};
#[allow(unused_imports)]
use usbipd::{USBIPD, UsbDevice, commands, runner};

use crate::usbipd::wsl;


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

fn main()  {
    init_log();
    let usbipd = USBIPD::new().expect("Failed to create USBIPD instance");
    info!("USBIPD instance created successfully");
    // let wsl_distribution = usbipd.get_default_or_first_running_wsl_distribution().expect("Failed to get WSL distribution");
    // debug!("WSL distribution: {}", wsl_distribution);
    // let wsl_distribution = Some(wsl_distribution.as_str());
    // let out = usbipd.run_wsl(wsl_distribution,  vec!["ls", "-l"]);
    // debug!("{:#?}", out);
    let devices = usbipd.get_all_devices().expect("Failed to get all USB devices");
    info!("Found {} USB devices", devices.len());
    for mut device in devices {
        if device.hardware_id.as_ref().map(|s| s.eq_ignore_ascii_case("1a2c:7fff")).unwrap_or(false) {
            debug!("Binding device {:#?}", device.hardware_id);
            match device.bind() {
                Ok(result) => {
                    info!("Binding successful: {}", result);
                }
                Err(e) => {
                    error!("Failed to bind device: {}", e);
                }
            }
            if let Err(e) = device.update() {
                error!("Failed to update device: {}", e);
            }
            info!("After binding, device info: {:#?}",device);
            sleep(Duration::from_secs(1));
            if let Err(e) = device.attach(None) {
                error!("Failed to attach device: {}", e);
            }
            if let Err(e) = device.update() {
                error!("Failed to update device: {}", e);
            }
            info!("After attaching, device info: {:#?}",device);
            sleep(Duration::from_secs(1));
            if let Err(e) = device.detach() {
                error!("Failed to detach device: {}", e);
            }
            if let Err(e) = device.update() {
                error!("Failed to update device: {}", e);
            }
            info!("After Detaching, device info: {:#?}",device);
            sleep(Duration::from_secs(1));
            if let Err(e) = device.unbind() {
                error!("Failed to unbind device: {}", e);
            }
            if let Err(e) = device.update() {
                error!("Failed to update device: {}", e);
            }
            info!("After unbinding, device info: {:#?}",device);
        }
    }

}
