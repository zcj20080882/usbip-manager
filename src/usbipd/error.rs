
#[derive(Debug)]
pub enum Error {
    #[allow(dead_code)]
    Unknown,
    #[allow(dead_code)]
    Timeout,
    #[allow(dead_code)]
    PermissionDenied,
    #[allow(dead_code)]
    WslNotFound,
    #[allow(dead_code)]
    WslLowVersion,
    #[allow(dead_code)]
    WslNotRunning,
    #[allow(dead_code)]
    UsbIPDNotFound,
    #[allow(dead_code)]
    UsbIPDLowVersion,
    #[allow(dead_code)]
    NotConnected,
    #[allow(dead_code)]
    NotBound,
    #[allow(dead_code)]
    NotAttached,
    #[allow(dead_code)]
    BindFailed,
    #[allow(dead_code)]
    AttachFailed,
    #[allow(dead_code)]
    DetachFailed,
    #[allow(dead_code)]
    UnbindFailed,
    #[allow(dead_code)]
    InternalError,
    #[allow(dead_code)]
    PowerShellError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Unknown => write!(f, "Unknown error"),
            Error::Timeout => write!(f, "Operation timed out"),
            Error::PermissionDenied => write!(f, "Permission denied"),
            Error::WslNotFound => write!(f, "WSL not found"),
            Error::WslLowVersion => write!(f, "WSL version is too low"),
            Error::WslNotRunning => write!(f, "WSL is not running"),
            Error::UsbIPDNotFound => write!(f, "usbipd not found"),
            Error::UsbIPDLowVersion => write!(f, "usbipd version is too low"),
            Error::NotConnected => write!(f, "Device is not connected"),
            Error::NotBound => write!(f, "Device is not bound"),
            Error::NotAttached => write!(f, "Device is not attached"),
            Error::BindFailed => write!(f, "Failed to bind device"),
            Error::AttachFailed => write!(f, "Failed to attach device"),
            Error::DetachFailed => write!(f, "Failed to detach device"),
            Error::UnbindFailed => write!(f, "Failed to unbind device"),
            Error::InternalError => write!(f, "Internal error occurred"),
            Error::PowerShellError => write!(f, "PowerShell error occurred"),
        }
    }
}
