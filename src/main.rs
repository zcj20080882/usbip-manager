

#[allow(unused_imports)]
use tracing::{info, warn, error, debug, trace};

slint::include_modules!();

mod log;
mod usbipd;

fn main() -> Result<(), slint::PlatformError> {
    log::init_log();
    let ui = MainView::new()?;
    let ui_handle = ui.as_weak();
    ui.run()
}
