

#[allow(unused_imports)]
use tracing::{info, warn, error, debug, trace};
#[allow(unused_imports)]
use usbipd::{USBIPD, UsbDevice, commands, runner};

mod log;
mod usbipd;

fn main()  {
    log::init_log();

}
