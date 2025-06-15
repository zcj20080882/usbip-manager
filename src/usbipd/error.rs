pub use std::error::Error as StdError;
use std::fmt::{self};

#[derive(Debug)]
pub enum ErrorCode {
    #[allow(dead_code)]
    Unknown,
    #[allow(dead_code)]
    Timeout,
    #[allow(dead_code)]
    PermissionDenied,
    #[allow(dead_code)]
    WslNotFound,
    #[allow(dead_code)]
    WslDistributionNotFound,
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
    #[allow(dead_code)]
    ParseError,
    #[allow(dead_code)]
    CommandNotFound,
}

#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
}

pub type USBIPDResult<T> = Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::Unknown => write!(f, "Unknown error"),
            ErrorCode::Timeout => write!(f, "Operation timed out"),
            ErrorCode::PermissionDenied => write!(f, "Permission denied"),
            ErrorCode::WslNotFound => write!(f, "WSL not found"),
            ErrorCode::WslLowVersion => write!(f, "WSL version is too low"),
            ErrorCode::WslNotRunning => write!(f, "WSL is not running"),
            ErrorCode::UsbIPDNotFound => write!(f, "usbipd not found"),
            ErrorCode::UsbIPDLowVersion => write!(f, "usbipd version is too low"),
            ErrorCode::NotConnected => write!(f, "Device is not connected"),
            ErrorCode::NotBound => write!(f, "Device is not bound"),
            ErrorCode::NotAttached => write!(f, "Device is not attached"),
            ErrorCode::BindFailed => write!(f, "Failed to bind device"),
            ErrorCode::AttachFailed => write!(f, "Failed to attach device"),
            ErrorCode::DetachFailed => write!(f, "Failed to detach device"),
            ErrorCode::UnbindFailed => write!(f, "Failed to unbind device"),
            ErrorCode::InternalError => write!(f, "Internal error occurred"),
            ErrorCode::PowerShellError => write!(f, "PowerShell error occurred"),
            ErrorCode::ParseError => write!(f, "Failed to parse"),
            ErrorCode::CommandNotFound => write!(f, "command not found"),
            ErrorCode::WslDistributionNotFound => write!(f, "WSL distribution not found"),
        }
    }
}
